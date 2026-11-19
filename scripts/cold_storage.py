#!/usr/bin/env python3
"""Verified local tar.zst archives, explicit restoration and coordinated readers.

Only ignored paths below target are eligible. Original private inputs are never
selected. Archives are verified before retirement; extraction never uses
extractall and never follows archived links. The store lease is intentionally
exclusive, including while an archived input is consumed.
"""
import argparse
import base64
from collections import Counter
import contextlib
import errno
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import uuid

from artifact_coordinator import Lease, NAME, ordinary
from local_artifacts import ensure_untracked

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = {'.gba', '.bios', '.sav', '.ffstate', '.input', '.rec', '.recording', '.sps'}
KIND = 'ffta-cold-storage'
GIB = 1024 ** 3
BLOCK = 1024 ** 2
# Set only by the container entry point after authenticating its host driver.
HOST_DRIVER_PID = None


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(BLOCK), b''):
            h.update(block)
    return h.hexdigest()


def sync_dir(path):
    if os.name == 'posix':
        fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(fd)
        finally:
            os.close(fd)


def save(path, data):
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as f:
        tmp = Path(f.name)
        try:
            f.write((json.dumps(data, indent=2) + '\n').encode())
            f.flush()
            os.fsync(f.fileno())
            f.close()
            os.replace(tmp, path)
            sync_dir(path.parent)
        finally:
            tmp.unlink(missing_ok=True)


def relative(value):
    p = PurePosixPath(value)
    if (not value or '\\' in value or p.is_absolute() or '..' in p.parts
            or (p.parts and p.parts[0].endswith(':')) or (os.name != 'posix' and ':' in value)):
        raise ValueError(f'Unsafe archive path: {value}')
    if str(p) != value:
        raise ValueError(f'Noncanonical archive path: {value}')
    return Path(*p.parts)


def external_mount(mount, expected_uuid=None):
    """Resolve an explicitly mounted Linux volume; never fall back to its empty mountpoint."""
    mount = Path(os.path.abspath(mount))
    try:
        ordinary(mount, directory=True)
    except OSError as error:
        raise ValueError(f'External archive volume is not available at {mount}: {error}') from error
    try:
        result = subprocess.run(['findmnt', '--json', '--mountpoint', str(mount),
                                 '--output', 'TARGET,UUID'], check=True,
                                capture_output=True, text=True)
        filesystems = json.loads(result.stdout)['filesystems']
    except (OSError, subprocess.SubprocessError, KeyError, ValueError) as error:
        raise ValueError(f'External archive volume is not mounted at {mount}: {error}') from error
    if len(filesystems) != 1 or filesystems[0].get('target') != str(mount):
        raise ValueError(f'External archive volume is not mounted at {mount}')
    uuid = filesystems[0].get('uuid')
    if not uuid or (expected_uuid and uuid.lower() != expected_uuid.lower()):
        raise ValueError(f'External archive volume UUID is missing or does not match at {mount}')
    return mount, uuid


def exists(path):
    return os.path.lexists(path)


def shareable_payload_groups(entries, new_entry):
    by_hash = {}
    for entry, data in entries:
        sealed = data['state'] == 'archived' or (entry == new_entry and data['state'] == 'restored')
        if sealed and not data.get('external_payload'):
            by_hash.setdefault(data['archive_sha256'], []).append((entry, data))
    # On low-space admission, check groups with the largest potential return first.
    groups = sorted(by_hash.values(), key=lambda g: (len(g) - 1) * g[0][1]['archive_bytes'], reverse=True)
    for matches in groups:
        if len(matches) < 2:
            break
        compatible = {}
        for entry, data in matches:
            path = entry / 'payload.tar.zst'
            st = ordinary(path, allow_hardlinks=True)
            mode = stat.S_IMODE(st.st_mode)
            try:
                attributes = os.listxattr(path, follow_symlinks=False)
            except OSError as error:
                if error.errno in (errno.EOPNOTSUPP, errno.ENOSYS):
                    continue
                raise
            if (st.st_size != data['archive_bytes'] or st.st_uid != os.getuid()
                    or mode & 0o7000 or attributes or (st.st_nlink > 1 and mode & 0o222)):
                continue
            key = (st.st_dev, st.st_uid, st.st_gid, mode & ~0o222)
            compatible.setdefault(key, []).append((entry, data, st))
        for group in compatible.values():
            names = Counter((st.st_dev, st.st_ino) for _, _, st in group)
            owned = [item for item in group if item[2].st_nlink == names[item[2].st_dev, item[2].st_ino]]
            if len(owned) < 2:
                continue
            owned.sort(key=lambda item: (item[0] == new_entry, -item[2].st_nlink, item[0]))
            # Only replacing a singleton releases its allocated blocks.
            copies = [item for item in owned[1:] if item[2].st_nlink == 1
                      and (new_entry is None or item[0] == new_entry)]
            if copies:
                yield owned[0], copies


def link_verified_payload(keeper, keep_stat, path, expected):
    current = ordinary(path)
    kept = ordinary(keeper, allow_hardlinks=True)
    if (not os.path.samestat(current, expected) or current.st_ctime_ns != expected.st_ctime_ns
            or not os.path.samestat(kept, keep_stat) or kept.st_ctime_ns != keep_stat.st_ctime_ns):
        raise ValueError('Local archive changed during compaction')
    if kept.st_mode & 0o222:
        ordinary(keeper)
        with keeper.open('rb') as f:
            os.fchmod(f.fileno(), stat.S_IMODE(kept.st_mode) & ~0o222)
            os.fsync(f.fileno())
    pending = path.with_name('.payload-link-' + uuid.uuid4().hex)
    try:
        os.link(keeper, pending)
    except OSError as error:
        if error.errno in (errno.EOPNOTSUPP, errno.EXDEV, errno.EMLINK, errno.EPERM, errno.ENOSYS):
            return False
        raise
    try:
        os.replace(pending, path)
        sync_dir(path.parent)
    finally:
        pending.unlink(missing_ok=True)
    return True


def compact_local_payloads(store, entries, *, apply=False, new_entry=None, goal_bytes=None):
    """Share sealed local archives under the catalog lease, preserving independent restores."""
    store.check()
    report = {'mode': 'apply' if apply else 'preview', 'supported': os.name == 'posix' and hasattr(os, 'listxattr'),
              'candidates': 0, 'candidate_bytes': 0, 'shared': 0, 'released_bytes': 0, 'errors': []}
    if not report['supported']:
        return report
    for (keeper, keep_data, keep_stat), copies in shareable_payload_groups(entries, new_entry):
        keep_path = keeper / 'payload.tar.zst'
        checking = keep_data
        try:
            if apply:
                store.load(keeper, keep_data)
            for entry, checking, st in copies:
                amount = st.st_blocks * 512
                report['candidates'] += 1
                report['candidate_bytes'] += amount
                if not apply:
                    continue
                store.load(entry, checking)
                store.check()
                if link_verified_payload(keep_path, keep_stat, entry / 'payload.tar.zst', st):
                    report['shared'] += 1
                    report['released_bytes'] += amount
                    if goal_bytes is not None and report['released_bytes'] >= goal_bytes:
                        return report
                keep_stat = keep_path.stat()
        except (OSError, ValueError) as error:
            store.check()
            if new_entry is not None:
                raise  # The new publication still has its original working input.
            report['errors'].append({'source': checking['source'], 'error': str(error)})
    return report


def scan(source, *, hashes=True, private=False):
    """Inventory links themselves and require every hard-link name to be owned."""
    ordinary(source.parent, directory=True)
    if source.is_symlink():
        raise ValueError('An archive root cannot be a symbolic link')
    rows = {}
    identities = {}

    def visit(p, name):
        s = p.lstat()
        if getattr(s, 'st_file_attributes', 0) & 0x400:
            raise ValueError(f'Windows reparse point refused: {p}')
        if p.name == NAME:
            raise ValueError(f'Active or interrupted artifact lease: {p}')
        if not private and p.suffix.lower() in PRIVATE:
            raise ValueError(f'Private input remains available outside cold storage: {p}')
        attrs = {}
        if hasattr(os, 'listxattr'):
            for key in os.listxattr(p, follow_symlinks=False):
                if not key.startswith('user.'):
                    raise ValueError(f'Privileged/ACL attributes require separate preservation: {p}')
                attrs[key] = base64.b64encode(os.getxattr(p, key, follow_symlinks=False)).decode('ascii')
        if os.name == 'posix' and (s.st_uid != os.getuid() or s.st_gid not in {*os.getgroups(), os.getgid()}):
            raise ValueError(f'Ownership cannot be restored by this user: {p}')
        if stat.S_IMODE(s.st_mode) & 0o7000:
            raise ValueError(f'Special permission bits require separate preservation: {p}')
        row = {'mode': stat.S_IMODE(s.st_mode), 'mtime_ns': s.st_mtime_ns,
               'uid': s.st_uid, 'gid': s.st_gid, 'identity': [s.st_dev, s.st_ino], 'xattrs': attrs}
        if stat.S_ISLNK(s.st_mode):
            row.update(type='symlink', target=os.readlink(p))
        elif stat.S_ISDIR(s.st_mode):
            row['type'] = 'dir'
        elif stat.S_ISREG(s.st_mode):
            row.update(type='file', size=s.st_size, links=s.st_nlink,
                       allocated=getattr(s, 'st_blocks', 0) * 512)
            if hashes:
                row['sha256'] = digest(p)
            identities.setdefault((s.st_dev, s.st_ino), []).append(name)
        else:
            raise ValueError(f'Special file refused: {p}')
        rows[name] = row
        if row['type'] == 'dir':
            for child in sorted(p.iterdir()):
                visit(child, child.relative_to(source).as_posix())

    visit(source, '.')
    for names in identities.values():
        if any(rows[n]['links'] != len(names) for n in names):
            raise ValueError(f'External hard link refused: {source / names[0]}')
        for name in names[1:]:
            rows[name]['link_to'] = names[0]
    return rows


def logical(rows):
    return {name: {k: v for k, v in row.items() if k not in {'identity', 'allocated'}}
            for name, row in rows.items()}


def size(rows, field='size'):
    return sum(row.get(field, 0) for row in rows.values() if not row.get('link_to'))


def capacity(path, additional, floor=5 * GIB):
    if additional < 0 or floor < 0:
        raise ValueError('Negative space budget')
    free = shutil.disk_usage(path).free
    if free < additional + floor:
        raise ValueError(f'Insufficient space: need {(additional + floor) / GIB:.2f} GiB, '
                         f'have {free / GIB:.2f} GiB; includes uncompressed/staging size and floor')


def remove_owned(path):
    # shutil.rmtree does not follow a symlink encountered inside the tree.
    ordinary(path.parent, directory=True)
    if path.is_symlink() or path.is_file():
        path.unlink()
    else:
        ordinary(path, directory=True)
        shutil.rmtree(path)
    sync_dir(path.parent)


def check_manifest(rows):
    if '.' not in rows or rows['.']['type'] not in {'file', 'dir'}:
        raise ValueError('Invalid archive root')
    for name, row in rows.items():
        p = relative(name)
        if row['type'] not in {'dir', 'file', 'symlink'}:
            raise ValueError('Unsupported archive member')
        if row['type'] == 'file' and (not isinstance(row['size'], int) or row['size'] < 0):
            raise ValueError('Invalid file size')
        if not isinstance(row['mode'], int) or row['mode'] & ~0o777:
            raise ValueError('Invalid archived permissions')
        for key, value in row.get('xattrs', {}).items():
            if not key.startswith('user.') or not hasattr(os, 'setxattr'):
                raise ValueError('Unsupported archived extended attributes')
            base64.b64decode(value, validate=True)
        for ancestor in p.parents:
            if rows.get(ancestor.as_posix(), {}).get('type') != 'dir':
                raise ValueError('Archive member has a non-directory ancestor')
        if row.get('link_to'):
            target = rows.get(row['link_to'], {})
            if target.get('type') != 'file' or target.get('link_to') or row['link_to'] == name:
                raise ValueError('Invalid hard-link target')
            if (row['size'], row['sha256']) != (target['size'], target['sha256']):
                raise ValueError('Inconsistent hard-link contents')


def codec():
    exe = shutil.which('zstd')
    if not exe:
        raise ValueError('zstd is required; install it or use --keep-hot with a reason')
    return exe


def make_archive(source, rows, dest):
    with dest.open('xb') as out:
        with subprocess.Popen([codec(), '-q', '-3', '-T1', '-c'], stdin=subprocess.PIPE,
                              stdout=out, stderr=subprocess.PIPE) as proc:
            try:
                with tarfile.open(mode='w|', fileobj=proc.stdin, format=tarfile.PAX_FORMAT) as tar:
                    for name, row in rows.items():
                        info = tarfile.TarInfo(name)
                        info.mode = row['mode']
                        info.uid, info.gid = row['uid'], row['gid']
                        info.mtime = row['mtime_ns'] // 1000000000
                        if row['type'] == 'dir':
                            info.type = tarfile.DIRTYPE
                        elif row['type'] == 'symlink':
                            info.type, info.linkname = tarfile.SYMTYPE, row['target']
                        elif row.get('link_to'):
                            info.type, info.linkname = tarfile.LNKTYPE, row['link_to']
                        else:
                            info.size = row['size']
                        if info.isreg():
                            with (source / name if name != '.' else source).open('rb') as f:
                                tar.addfile(info, f)
                        else:
                            tar.addfile(info)
            finally:
                proc.stdin.close()
            error = proc.stderr.read(8192)
            if proc.wait() != 0:
                raise ValueError(f'Compression failed: {error.decode(errors="replace")}')
        out.flush()
        os.fsync(out.fileno())


def read_archive(payload, rows, destination=None):
    """Verify every member; optionally materialize only manifest-approved names."""
    check_manifest(rows)
    seen = set()
    with subprocess.Popen([codec(), '-q', '-d', '-c', str(payload)], stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE) as proc:
        try:
            with tarfile.open(mode='r|', fileobj=proc.stdout) as tar:
                for member in tar:
                    name = member.name
                    if name not in rows or name in seen:
                        raise ValueError(f'Unexpected/duplicate archive member: {name}')
                    seen.add(name)
                    row = rows[name]
                    expected = ('dir' if member.isdir() else 'symlink' if member.issym()
                                else 'hardlink' if member.islnk() else 'file' if member.isreg() else '')
                    if expected != ('hardlink' if row.get('link_to') else row['type']):
                        raise ValueError(f'Archive member type changed: {name}')
                    if (member.mode, member.uid, member.gid, member.mtime) != (
                            row['mode'], row['uid'], row['gid'], row['mtime_ns'] // 1000000000):
                        raise ValueError(f'Archive member metadata changed: {name}')
                    dest = destination / name if destination and name != '.' else destination
                    if row['type'] == 'dir' and dest:
                        dest.mkdir()
                    if row['type'] == 'symlink' or row.get('link_to'):
                        if member.linkname != row.get('link_to', row.get('target')):
                            raise ValueError(f'Archive link target changed: {name}')
                        continue  # Links are created last, never traversed during extraction.
                    if row['type'] == 'file':
                        if member.size != row['size']:
                            raise ValueError(f'Archive file size changed: {name}')
                        h = hashlib.sha256()
                        with tar.extractfile(member) as src, (dest.open('xb') if dest else contextlib.nullcontext()) as out:
                            for block in iter(lambda: src.read(BLOCK), b''):
                                h.update(block)
                                if out:
                                    out.write(block)
                            if out:
                                out.flush()
                                os.fsync(out.fileno())
                        if h.hexdigest() != row['sha256']:
                            raise ValueError(f'Archive content hash mismatch: {name}')
            # Drain the remaining zstd frame/footer so checksum errors cannot be missed.
            if any(proc.stdout.read(BLOCK)):
                raise ValueError('Unexpected trailing archive data')
            while block := proc.stdout.read(BLOCK):
                if any(block):
                    raise ValueError('Unexpected trailing archive data')
            error = proc.stderr.read(8192)
            if proc.wait() != 0:
                raise ValueError(f'Decompression failed: {error.decode(errors="replace")}')
        except BaseException:
            proc.kill()
            proc.wait()
            raise
    if seen != set(rows):
        raise ValueError('Archive is missing members')
    if destination:
        for name, row in rows.items():
            p = destination / name if name != '.' else destination
            if row.get('link_to'):
                os.link(destination / row['link_to'], p)
            elif row['type'] == 'symlink':
                os.symlink(row['target'], p)
        for name, row in reversed(list(rows.items())):
            p = destination / name if name != '.' else destination
            if os.name == 'posix':
                os.chown(p, row['uid'], row['gid'], follow_symlinks=False)
            if row['type'] != 'symlink':
                os.chmod(p, row['mode'])
            for key, value in row.get('xattrs', {}).items():
                os.setxattr(p, key, base64.b64decode(value, validate=True), follow_symlinks=False)
            os.utime(p, ns=(row['mtime_ns'], row['mtime_ns']), follow_symlinks=False)
            if row['type'] == 'dir':
                sync_dir(p)
            elif row['type'] == 'file':
                with p.open('rb') as f:
                    os.fsync(f.fileno())
        sync_dir(destination.parent)
        if logical(scan(destination)) != logical(rows):
            raise ValueError('Restored tree differs from its manifest')


class Store:
    def __init__(self, root=ROOT):
        self.root = root.resolve()
        self.base = self.root / 'target/cold-storage'
        self.lease = None

    def __enter__(self):
        (self.root / 'target').mkdir(exist_ok=True)
        ordinary(self.root / 'target', directory=True)
        self.base.mkdir(exist_ok=True)
        ordinary(self.base, directory=True)
        sync_dir(self.base.parent)
        self.lease = Lease(self.base, 'cold-storage')
        self.lease.__enter__()
        return self

    def __exit__(self, *exc):
        if self.lease:
            self.lease.__exit__(*exc)

    def check(self):
        if not self.lease:
            raise ValueError('Cold storage needs its activity lease')
        self.lease.check()

    def source(self, value):
        p = Path(os.path.abspath(value if Path(value).is_absolute() else self.root / value))
        rel = p.relative_to(self.root)
        parts = rel.parts
        if len(parts) < 2 or parts[0] != 'target':
            raise ValueError('Only explicit artifacts below target can be archived')
        if parts[1] in {'cold-storage', 'debug', 'release'} or parts[1].startswith(('x86_64-', 'aarch64-')):
            raise ValueError('Canonical build outputs and cold storage remain available')
        if rel.as_posix() in {'target/validation', 'target/validation/units', 'target/validation/wine-prefix'}:
            raise ValueError('Shared validation infrastructure remains available')
        if 'wine-prefix' in parts and parts[:2] == ('target', 'validation') and parts[2] == 'wine-prefix':
            raise ValueError('Shared Wine remains available')
        relative(rel.as_posix())
        return p, rel.as_posix()

    def entries(self):
        self.check()
        result = []
        for entry in sorted(self.base.iterdir()):
            if entry.name == NAME:
                continue
            ordinary(entry, directory=True)
            p = entry / 'entry.json'
            ordinary(p)
            data = json.loads(p.read_text())
            if data.get('kind') != KIND or data.get('schema') != 1:
                raise ValueError(f'Unknown cold storage entry: {entry}')
            source, rel = self.source(data['source'])
            if entry.name != hashlib.sha256(rel.encode()).hexdigest():
                raise ValueError('Cold storage entry identity mismatch')
            result.append((entry, data))
        return result

    def load(self, entry, data):
        self.check()
        if data['state'] == 'preparing':
            raise ValueError(f'Incomplete archive retained for review: {entry}')
        manifest, payload = entry / 'manifest.json', self.payload(entry, data)
        ordinary(manifest)
        payload_stat = ordinary(payload, allow_hardlinks=not data.get('external_payload'))
        if payload_stat.st_nlink > 1 and payload_stat.st_mode & 0o222:
            raise ValueError('Shared local archive must be read-only')
        if digest(manifest) != data['manifest_sha256'] or digest(payload) != data['archive_sha256']:
            raise ValueError('Cold archive/catalog checksum mismatch')
        rows = json.loads(manifest.read_text())
        check_manifest(rows)
        return rows

    def payload(self, entry, data):
        external = data.get('external_payload')
        if not external:
            return entry / 'payload.tar.zst'
        mount_override = os.environ.get('FFTA_COLD_STORAGE_MOUNT')
        mount, _ = external_mount(mount_override or external['mount'], external['uuid'])
        payload = mount / relative(external['path'])
        ordinary(payload)
        if payload.stat().st_dev != mount.stat().st_dev:
            raise ValueError('External archive is not on its recorded volume')
        return payload

    def offload(self, entry, data, mount, directory, *, completion_paths=()):
        """Copy one completed archive to a mounted volume, verify it, then retire its local payload."""
        self.check()
        if data.get('state') != 'archived' or data.get('external_payload'):
            raise ValueError('Only local, fully archived entries can be moved externally')
        proof = data.get('proof', {})
        local_outcomes = ('FAIL', 'MIXED', 'INTERRUPTED')
        if (proof.get('kind') == 'failed-diagnostic-preservation'
                or proof.get('original_outcome') in local_outcomes or proof.get('status') in local_outcomes):
            raise ValueError('Failed or mixed diagnostic archives stay on the internal disk')
        if not proof.get('scope'):
            raise ValueError('External offload requires an explicit completion scope')
        scope = self.root / relative(proof['scope'])
        source = self.root / relative(data['source'])
        if scope == source or scope not in source.parents or not scope.is_relative_to(self.root / 'target'):
            raise ValueError('Invalid archived completion scope')
        evidence = proof.get('completion', [])
        if completion_paths:
            evidence = []
            for value in completion_paths:
                path = Path(os.path.abspath(value if Path(value).is_absolute() else self.root / value))
                if path == scope or scope not in path.parents or path == source or source in path.parents:
                    raise ValueError('Offload completion evidence must be within the scope and outside the payload')
                evidence.append({'path': path.relative_to(self.root).as_posix(), 'sha256': offload_completion(path)})
        if not evidence:
            raise ValueError('External offload requires explicit completion evidence and scope')
        if exists(source):
            raise ValueError('Archived source exists; preserve it and review its state')

        def check_evidence():
            for record in evidence:
                if offload_completion(self.root / relative(record['path'])) != record['sha256']:
                    raise ValueError('Completion evidence changed; offload refused')
            for guard in proof.get('outcome_guards', []):
                path = scope / relative(guard['name'])
                ordinary(path)
                if digest(path) != guard['sha256']:
                    raise ValueError('Historical outcome evidence changed; offload refused')

        check_evidence()
        mount, uuid = external_mount(mount)
        directory = relative(str(directory))
        target_dir = mount / directory
        ordinary(target_dir, directory=True)
        if target_dir.stat().st_dev != mount.stat().st_dev:
            raise ValueError('Offload destination is not on the selected volume')
        destination = target_dir / (entry.name + '.tar.zst')
        local = entry / 'payload.tar.zst'
        with Lease(scope, 'cold-storage-external-offload'):
            host_activity(scope, root=self.root)
            self.load(entry, data)
            initial = ordinary(local, allow_hardlinks=True)
            with local.open('rb') as src, destination.open('xb') as dst:
                shutil.copyfileobj(src, dst, BLOCK)
                dst.flush(); os.fsync(dst.fileno())
            sync_dir(target_dir)
            ordinary(destination)
            if digest(destination) != data['archive_sha256']:
                raise ValueError('External copy checksum mismatch; local archive retained')
            activity = host_activity(scope, root=self.root)
            self.check()
            external_mount(mount, uuid)
            check_evidence()
            if exists(source):
                raise ValueError('Archived source appeared during offload; both copies retained')
            current = ordinary(local, allow_hardlinks=True)
            if ((current.st_dev, current.st_ino, current.st_size, current.st_mtime_ns)
                    != (initial.st_dev, initial.st_ino, initial.st_size, initial.st_mtime_ns)
                    or digest(local) != data['archive_sha256']):
                raise ValueError('Local archive changed during offload; both copies retained')
            target_identity = ordinary(destination)
            published = dict(data, external_payload={'mount': str(mount), 'uuid': uuid,
                                                      'path': (directory / destination.name).as_posix()},
                             offloaded_at=time.time(), offload_activity=activity,
                             offload_review={'completion': evidence,
                                             'basis': 'explicit' if completion_paths else 'archive proof'})
            save(entry / 'entry.json', published)
            self.load(entry, published)
            external_mount(mount, uuid)
            final_identity = ordinary(destination)
            if ((final_identity.st_dev, final_identity.st_ino, final_identity.st_size, final_identity.st_mtime_ns)
                    != (target_identity.st_dev, target_identity.st_ino, target_identity.st_size,
                        target_identity.st_mtime_ns)):
                raise ValueError('External archive changed during publication; local copy retained')
            local.unlink()
            sync_dir(entry)
        return published

    def pack(self, path, *, reason, proof, floor=5 * GIB, before_retire=None):
        self.check()
        source, rel = self.source(path)
        if not reason.strip() or not proof:
            raise ValueError('Archiving needs a reason and completion/ownership evidence')
        existing = self.entries()
        for _, data in existing:
            other = self.root / data['source']
            if source == other or source in other.parents or other in source.parents:
                raise ValueError(f'Already cataloged or overlapping archive: {data["source"]}')
        ensure_untracked(source, self.root)
        rows = scan(source)
        if proof.get('inventory_sha256') and inventory_digest(rows) != proof['inventory_sha256']:
            raise ValueError('Reviewed source inventory changed')
        # Worst-case streaming output plus headers; no full uncompressed tar copy.
        capacity(self.base, size(rows) + len(rows) * 2048 + BLOCK, floor)
        codec()
        entry = self.base / hashlib.sha256(rel.encode()).hexdigest()
        entry.mkdir()
        sync_dir(self.base)
        data = {'schema': 1, 'kind': KIND, 'source': rel, 'state': 'preparing',
                'reason': reason, 'proof': proof, 'created_at': time.time(),
                'logical_bytes': size(rows), 'source_allocated_bytes': size(rows, 'allocated')}
        save(entry / 'entry.json', data)
        save(entry / 'manifest.json', rows)
        # Keep small dataset/result identities readable without restoring payloads.
        for name, row in rows.items():
            if (row['type'] == 'file' and row['size'] <= 16 * BLOCK
                    and (Path(name).name == 'manifest.toml'
                         or Path(name).name.endswith(('result.json', 'summary.json')))):
                dest = entry / 'evidence' / ('root-result.json' if name == '.' else name)
                dest.parent.mkdir(parents=True, exist_ok=True)
                with (source if name == '.' else source / name).open('rb') as src, dest.open('xb') as out:
                    shutil.copyfileobj(src, out, BLOCK)
                    out.flush(); os.fsync(out.fileno())
                if digest(dest) != row['sha256']:
                    raise ValueError('Readable evidence changed during archival')
        make_archive(source, rows, entry / 'payload.pending.tar.zst')
        read_archive(entry / 'payload.pending.tar.zst', rows)
        if scan(source) != rows:
            raise ValueError('Source changed while being archived; originals retained')
        os.replace(entry / 'payload.pending.tar.zst', entry / 'payload.tar.zst')
        sync_dir(entry)
        data.update(state='restored', manifest_sha256=digest(entry / 'manifest.json'),
                    archive_sha256=digest(entry / 'payload.tar.zst'),
                    archive_bytes=(entry / 'payload.tar.zst').stat().st_size,
                    archive_allocated_bytes=(entry / 'payload.tar.zst').stat().st_blocks * 512
                    if hasattr((entry / 'payload.tar.zst').stat(), 'st_blocks') else 0)
        save(entry / 'entry.json', data)
        peers = [(e, d) for e, d in existing if d.get('archive_sha256') == data['archive_sha256']]
        if peers:
            shared = compact_local_payloads(self, [*peers, (entry, data)], apply=True, new_entry=entry)
            if shared['shared']:
                data['archive_added_bytes'] = 0
        if before_retire:
            data['idle_recheck'] = before_retire()
            save(entry / 'entry.json', data)
        self.retire(entry, data, rows=rows)
        return data

    def retire(self, entry, data, *, rows=None):
        self.check()
        rows = self.load(entry, data) if rows is None else rows
        source = self.root / data['source']
        if exists(entry / 'retiring'):
            raise ValueError('Interrupted retirement needs explicit recovery')
        if not exists(source):
            if data['state'] != 'archived':
                raise ValueError('Unattested missing source; recovery required')
            return
        ensure_untracked(source, self.root)
        if logical(scan(source)) != logical(rows):
            raise ValueError('Restored input changed; retain it and its original archive')
        data['state'] = 'retiring'
        save(entry / 'entry.json', data)
        os.rename(source, entry / 'retiring')
        sync_dir(source.parent); sync_dir(entry)
        remove_owned(entry / 'retiring')
        data.update(state='archived', retired_at=time.time())
        save(entry / 'entry.json', data)

    def restore(self, entry, data, *, floor=5 * GIB):
        self.check()
        rows = self.load(entry, data)
        source = self.root / data['source']
        if data['state'] in {'retiring', 'restoring'} or exists(entry / 'retiring') or exists(entry / 'staging'):
            raise ValueError('Interrupted archive operation needs explicit recovery')
        if exists(source):
            if data['state'] != 'restored' or logical(scan(source)) != logical(rows):
                raise ValueError('Restore destination exists or differs; refusing overwrite')
            return False
        if data['state'] != 'archived':
            raise ValueError('Missing restored input; explicit recovery required')
        ordinary(source.parent, directory=True)
        capacity(source.parent, size(rows) + len(rows) * 4096, floor)
        data['state'] = 'restoring'
        save(entry / 'entry.json', data)
        read_archive(self.payload(entry, data), rows, entry / 'staging')
        if exists(source):
            raise ValueError('Restore destination appeared; refusing overwrite')
        os.rename(entry / 'staging', source)
        sync_dir(source.parent); sync_dir(entry)
        data.update(state='restored', restored_at=time.time())
        save(entry / 'entry.json', data)
        return True

    def matching(self, paths):
        paths = [Path(os.path.abspath(p)) for p in paths]
        return [(entry, data) for entry, data in self.entries()
                if any(p == self.root / data['source'] or p in (self.root / data['source']).parents
                       or self.root / data['source'] in p.parents for p in paths)]

    def recover(self, entry, data):
        """Recover interrupted publication without overwriting or guessing owners."""
        rows = self.load(entry, data)
        read_archive(self.payload(entry, data), rows)
        source = self.root / data['source']
        if data['state'] == 'retiring':
            if exists(source) and exists(entry / 'retiring'):
                raise ValueError('Conflicting original and retiring paths')
            if exists(source):
                if logical(scan(source)) != logical(rows):
                    raise ValueError('Original changed; recovery refused')
                data['state'] = 'restored'
            else:
                residue = entry / 'retiring'
                if exists(residue):
                    # A partial rmtree may have changed directories/link counts.
                    pending = [residue]
                    while pending:
                        p = pending.pop(); rel = p.relative_to(residue).as_posix()
                        row = rows.get(rel); s = p.lstat()
                        kind = ('symlink' if stat.S_ISLNK(s.st_mode) else 'file' if stat.S_ISREG(s.st_mode)
                                else 'dir' if stat.S_ISDIR(s.st_mode) else 'special')
                        if row is None or kind != row['type']:
                            raise ValueError('Unknown retirement residue')
                        if kind == 'symlink' and os.readlink(p) != row['target']:
                            raise ValueError('Changed retirement link')
                        if kind == 'file' and digest(p) != row['sha256']:
                            raise ValueError('Changed retirement residue')
                        if kind == 'dir':
                            pending.extend(p.iterdir())
                    remove_owned(residue)
                data['state'] = 'archived'
        elif data['state'] == 'restoring':
            if exists(source):
                if exists(entry / 'staging') or logical(scan(source)) != logical(rows):
                    raise ValueError('Conflicting restored destination')
                data['state'] = 'restored'
            else:
                # Interrupted extraction is owned scratch, never the sole source.
                if exists(entry / 'staging'):
                    remove_owned(entry / 'staging')
                data['state'] = 'archived'
        else:
            raise ValueError('No recoverable publication in progress; preparing archives need review')
        save(entry / 'entry.json', data)
        return data




def inventory_digest(rows):
    return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def host_activity(source, *, root=ROOT):
    """Bounded legacy-source check; never infer host inactivity from sandbox ps."""
    if os.name != 'posix' or not Path('/proc/1/comm').exists() or Path('/proc/1/comm').read_text().strip() != 'systemd':
        raise ValueError('Historical archival requires the host PID namespace; sandbox process visibility is insufficient')
    excluded = {os.getpid()}
    for pid in (os.getppid(), HOST_DRIVER_PID):
        while pid and pid not in excluded:
            excluded.add(pid)
            try:
                status = Path(f'/proc/{pid}/status').read_text()
                pid = int(next(line.split()[1] for line in status.splitlines() if line.startswith('PPid:')))
            except (OSError, StopIteration):
                break
    needles = [str(source)]
    if source.is_relative_to(root):
        needles.append(source.relative_to(root).as_posix())
    busy, denied = [], []
    def match(value):
        return any(needle in value.replace(chr(92), '/') for needle in needles)
    for process in Path('/proc').iterdir():
        if not process.name.isdigit() or int(process.name) in excluded:
            continue
        try:
            if match((process / 'cmdline').read_bytes().decode(errors='replace')):
                busy.append([int(process.name), 'command']); continue
            if process.stat().st_uid != os.getuid():
                continue
            if match((process / 'environ').read_bytes().decode(errors='replace')) or match(os.readlink(process / 'cwd')):
                busy.append([int(process.name), 'environment/cwd']); continue
            for fd in (process / 'fd').iterdir():
                try:
                    if match(os.readlink(fd)):
                        busy.append([int(process.name), 'open-file']); break
                except FileNotFoundError:
                    continue
            if match((process / 'maps').read_text()):
                busy.append([int(process.name), 'mapped-file'])
        except (FileNotFoundError, ProcessLookupError):
            continue
        except PermissionError:
            denied.append(int(process.name))
    if busy:
        raise ValueError(f'Selected source has live consumers: {busy}')
    return {'checked_at': time.time(), 'source': str(source), 'host_pid_namespace':
            os.readlink('/proc/self/ns/pid'), 'selected_users': busy,
            'unreadable_processes': denied,
            'limit': 'Unreadable process details remain unknown; no claim of global inactivity'}


def completion(path):
    ordinary(path)
    data = json.loads(path.read_text())
    if not isinstance(data, dict) or not (data.get('status') == 'PASS' or data.get('closed') is True
                                         or ('normal_exit' in data and data['normal_exit'] == 0)):
        raise ValueError(f'Explicit completed/PASS evidence required: {path}')
    return digest(path)


def offload_completion(path):
    """Accept existing PASS receipts and explicit zero-exit historical result formats."""
    ordinary(path)
    if path.stat().st_size > 2 * BLOCK:
        raise ValueError(f'Completion evidence is too large: {path}')
    data = json.loads(path.read_text())

    def passed(value):
        if isinstance(value, list):
            return bool(value) and all(passed(item) for item in value)
        if not isinstance(value, dict):
            return False
        if 'status' in value and value['status'] != 'PASS':
            return False
        if any(str(value.get(key, '')).upper().startswith(
                ('FAIL', 'ERROR', 'PARTIAL', 'INTERRUPTED', 'SUSPENDED', 'WAITING', 'RUNNING'))
               for key in ('outcome', 'state')):
            return False
        if any(key in value and value[key] is not True for key in ('protected_unchanged', 'test_result')):
            return False
        if value.get('error') or value.get('errors'):
            return False
        if any(key in value and (type(value[key]) is not int or value[key] != 0)
               for key in ('exit', 'exit_code', 'returncode', 'normal_exit')):
            return False
        if any(key in value and isinstance(value[key], list) and not passed(value[key])
               for key in ('results', 'tests', 'cases')):
            return False
        return (value.get('status') == 'PASS' or value.get('closed') is True
                or any(type(value.get(key)) is int and value[key] == 0
                       for key in ('exit', 'exit_code', 'returncode', 'normal_exit')))

    if not passed(data):
        raise ValueError(f'Explicit completed/PASS evidence required: {path}')
    return digest(path)


def diagnostic_evidence(path):
    ordinary(path)
    with path.open('rb') as f:
        content = f.read(2 * BLOCK + 1)
    if len(content) > 2 * BLOCK:
        raise ValueError(f'Oversized diagnostic receipt: {path}')
    value = json.loads(content)
    if (not isinstance(value, dict) or value.get('leases_retained')
            or str(value.get('status', '')).upper() in ('ACTIVE', 'RUNNING', 'SUSPENDED', 'WAITING')):
        raise ValueError(f'Closed diagnostic receipt required: {path}')
    if not (
            value.get('closed') is True or value.get('status') == 'FAIL'
            or (type(value.get('session_exit')) is int and type(value.get('verify_exit')) is int)):
        raise ValueError(f'Closed diagnostic receipt required: {path}')
    return hashlib.sha256(content).hexdigest()


def check_diagnostic_scope(scope, *, held=None):
    if held:
        held.check(scope)
    def onerror(error):
        raise error
    for parent, dirs, files in os.walk(scope, onerror=onerror, followlinks=False):
        if NAME in dirs or NAME in files:
            mark = Path(parent) / NAME
            if not held or mark != held.path:
                raise ValueError(f'Existing activity lease inside diagnostic scope: {mark}')


def prepare_review(store, path, completions, scope, *, local_diagnostic=None):
    source, rel = store.source(path)
    scope = Path(os.path.abspath(scope))
    if source == scope or scope not in source.parents or not scope.is_relative_to(store.root / 'target'):
        raise ValueError('Activity scope must be an ancestor below target, outside the archived payload')
    ordinary(scope, directory=True)
    if (scope / NAME).exists():
        raise ValueError('Activity scope already leased')
    if local_diagnostic:
        if local_diagnostic not in ('FAIL', 'MIXED'):
            raise ValueError('Expected an explicitly reviewed FAIL or MIXED outcome')
        check_diagnostic_scope(scope)
    ensure_untracked(source, store.root)
    evidence = []
    for p in completions:
        p = Path(os.path.abspath(p))
        if p == source or source in p.parents:
            raise ValueError('Completion evidence must remain readable outside the archive')
        if local_diagnostic and scope not in p.parents:
            raise ValueError('Diagnostic evidence must belong to the selected scope')
        sha = diagnostic_evidence(p) if local_diagnostic else completion(p)
        evidence.append({'path': p.relative_to(store.root).as_posix(), 'sha256': sha})
    if not evidence:
        raise ValueError('Completion evidence is required')
    rows = scan(source)
    review = {'source': rel, 'scope': scope.relative_to(store.root).as_posix(),
            'inventory_sha256': inventory_digest(rows), 'logical_bytes': size(rows),
            'allocated_bytes': size(rows, 'allocated'), 'idle': host_activity(scope, root=store.root)}
    if local_diagnostic:
        return dict(review, kind='failed-diagnostic-preservation', status=local_diagnostic,
                    original_outcome=local_diagnostic, outcome_evidence=evidence)
    return dict(review, status='PASS', completion=evidence)


def apply_review(store, review, reason):
    source, rel = store.source(review['source'])
    diagnostic = review.get('kind') == 'failed-diagnostic-preservation'
    valid = (review.get('status') in ('FAIL', 'MIXED') and review['status'] == review.get('original_outcome')
             if diagnostic else review.get('status') == 'PASS')
    if not valid or not review.get('inventory_sha256') or not review.get('idle'):
        raise ValueError('Reviewed inventory, completion and host activity evidence required')
    scope = store.root / relative(review['scope'])
    if source == scope or scope not in source.parents or not scope.is_relative_to(store.root / 'target'):
        raise ValueError('Invalid archive activity scope')
    records = review.get('outcome_evidence' if diagnostic else 'completion')
    if not records:
        raise ValueError('Missing outcome evidence')
    with Lease(scope, 'cold-storage-historical-source') as lease:
        def recheck():
            if diagnostic:
                check_diagnostic_scope(scope, held=lease)
            for evidence in records:
                p = store.root / relative(evidence['path'])
                if p == source or source in p.parents or (diagnostic and scope not in p.parents):
                    raise ValueError('Evidence lies within the payload or outside the diagnostic scope')
                sha = diagnostic_evidence(p) if diagnostic else completion(p)
                if sha != evidence['sha256']:
                    raise ValueError('Completion evidence changed or would be archived')
            return host_activity(scope, root=store.root)
        idle = recheck()
        proof = dict(review, idle=idle)
        return store.pack(source, reason=reason, proof=proof, before_retire=recheck)


class Inputs:
    """Hold declared archived inputs through a foreground consumer."""
    def __init__(self, root, paths, *, floor=5 * GIB, archive_floor=5 * GIB):
        self.root, self.paths, self.floor = root, paths, floor
        self.archive_floor = archive_floor
        self.store = None
        self.restored = []
        self.selected = []
        self.safe_to_release = True

    def open(self):
        if not (self.root / 'target/cold-storage').exists():
            return
        self.store = Store(self.root).__enter__()
        self.selected = self.store.matching(self.paths)
        if not self.selected:
            self.close(success=False)
            return
        for entry, data in self.selected:
            if self.store.restore(entry, data, floor=self.floor):
                self.restored.append((entry, data))

    def close(self, *, success):
        if self.store is None:
            return
        store = self.store
        if not self.safe_to_release:
            # A timed-out consumer may still have descendants using these paths.
            self.store = None
            return  # The directory lease intentionally stays on disk.
        try:
            if success:
                for entry, data in self.restored:
                    store.retire(entry, data)
        finally:
            self.store = None
            store.__exit__(None, None, None)

    def archive_outputs(self, work, *, reason, proof):
        archived, skipped = [], []
        try:
            scan(work, hashes=False)
            candidates = [work]
        except ValueError:
            candidates = sorted(work.iterdir())
        own_store = self.store is None
        store = self.store or Store(self.root).__enter__()
        try:
            for source in candidates:
                try:
                    scan(source, hashes=False)
                except (ValueError, OSError) as error:
                    skipped.append({'path': str(source.relative_to(self.root)), 'reason': str(error)})
                    continue
                data = store.pack(source, reason=reason, proof=proof, floor=self.archive_floor)
                archived.append({k: data[k] for k in ['source', 'archive_sha256', 'archive_bytes',
                                                     'logical_bytes', 'source_allocated_bytes']})
        finally:
            if own_store:
                store.__exit__(None, None, None)
        return archived, skipped


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    sub = parser.add_subparsers(dest='action', required=True)
    sub.add_parser('list')
    compact = sub.add_parser('compact', help='share byte-identical local archive payloads')
    compact.add_argument('--apply', action='store_true', help='apply verified sharing (default: preview)')
    batch = sub.add_parser('batch', help='discover and compress completed historical artifacts automatically')
    batch.add_argument('--apply', action='store_true', help='compress eligible paths; default is a preview')
    batch.add_argument('--completed-only', action='store_true', help='require recognized successful receipts; skip idle-history fallback')
    batch.add_argument('--min-mib', type=int, default=1, help='minimum allocated MiB per archive (default: 1)')
    batch.add_argument('--exclude', action='append', default=[], help='additional repository-relative path glob to preserve')
    for name in ['restore', 'retire', 'verify', 'recover']:
        p = sub.add_parser(name); p.add_argument('path')
    offload = sub.add_parser('offload', help='move one completed archive to a mounted external volume')
    offload.add_argument('path'); offload.add_argument('--mount', required=True, type=Path)
    offload.add_argument('--directory', required=True, help='existing directory relative to the volume root')
    offload.add_argument('--completion', action='append', default=[], type=Path,
                         help='reviewed completed result within the scope (repeatable)')
    plan = sub.add_parser('plan')
    plan.add_argument('path'); plan.add_argument('--scope', required=True, type=Path)
    plan.add_argument('--completion', '--evidence', action='append', type=Path, required=True)
    plan.add_argument('--local-diagnostic', choices=('FAIL', 'MIXED'),
                      help='preserve an explicitly reviewed closed diagnostic with its original outcome locally')
    plan.add_argument('--output', type=Path, required=True)
    run = sub.add_parser('run')
    run.add_argument('--input', action='append', type=Path, required=True)
    run.add_argument('--timeout', type=float, default=900)
    run.add_argument('command', nargs=argparse.REMAINDER)
    pack = sub.add_parser('pack')
    pack.add_argument('path'); pack.add_argument('--reason', required=True)
    pack.add_argument('--review', type=Path, required=True,
                      help='reviewed completion/idle evidence; no automatic historical selection')
    args = parser.parse_args()
    try:
        if args.action == 'run':
            command = args.command[1:] if args.command[:1] == ['--'] else args.command
            if not command or args.timeout <= 0:
                raise ValueError('A foreground command and positive timeout are required')
            inputs = Inputs(args.root.resolve(), [p.absolute() for p in args.input])
            try:
                inputs.open()
                inputs.safe_to_release = False
                result = subprocess.run(command, timeout=args.timeout)
                inputs.safe_to_release = True
                inputs.close(success=result.returncode == 0)
                return result.returncode
            except BaseException:
                inputs.close(success=False)
                raise
        with Store(args.root) as store:
            if args.action == 'compact':
                report = compact_local_payloads(store, store.entries(), apply=args.apply)
                print(json.dumps(report, indent=2))
                return 1 if report['errors'] else 0
            if args.action == 'batch':
                import cold_storage_batch
                return cold_storage_batch.run(store, args)
            if args.action == 'list':
                print(json.dumps([{**{k: d.get(k) for k in ['source', 'state', 'logical_bytes', 'archive_bytes']},
                                   'storage': 'external' if d.get('external_payload') else 'local'}
                                  for _, d in store.entries()], indent=2)); return 0
            source, rel = store.source(args.path)
            if args.action == 'plan':
                review = prepare_review(store, source, args.completion, args.scope, local_diagnostic=args.local_diagnostic)
                output = args.output.absolute()
                if output == source or source in output.parents:
                    raise ValueError('Review output must stay outside the payload')
                ordinary(output.parent, directory=True)
                if exists(output):
                    raise ValueError('Review output already exists')
                save(output, review)
                print(json.dumps(review, indent=2)); return 0
            if args.action == 'pack':
                review = json.loads(args.review.read_text())
                if review.get('source') != rel:
                    raise ValueError('Review belongs to another source')
                print(json.dumps(apply_review(store, review, args.reason), indent=2))
                return 0
            matches = store.matching([source])
            if not matches:
                raise ValueError('No matching archive')
            if args.action == 'offload':
                exact = [(entry, data) for entry, data in matches if data['source'] == rel]
                if len(exact) != 1:
                    raise ValueError('Offload requires the exact source path of one archive')
                entry, data = exact[0]
                print(json.dumps(store.offload(entry, data, args.mount, args.directory,
                                               completion_paths=args.completion), indent=2))
                return 0
            for entry, data in matches:
                if args.action == 'restore': store.restore(entry, data)
                elif args.action == 'retire':
                    scope = data.get('proof', {}).get('scope', data.get('proof', {}).get('unit', data['source']))
                    host_activity(store.root / relative(scope), root=store.root); store.retire(entry, data)
                elif args.action == 'recover': store.recover(entry, data)
                else: read_archive(store.payload(entry, data), store.load(entry, data))
                print(f'PASS {args.action}: {data["source"]}')
        return 0
    except (OSError, ValueError, KeyError, tarfile.TarError, subprocess.SubprocessError) as error:
        print(f'Cold storage refused or retained for review: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
