#!/usr/bin/env python3
"""Verified local tar.zst archives, explicit restoration and coordinated readers.

Only ignored paths below target are eligible. Original private inputs are never
selected. Archives are verified before retirement; extraction never uses
extractall and never follows archived links. The store lease is intentionally
exclusive, including while an archived input is consumed.
"""
import argparse
import base64
import contextlib
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

from artifact_coordinator import Lease, NAME, ordinary
from local_artifacts import ensure_untracked

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = {'.gba', '.bios', '.sav', '.ffstate', '.input', '.rec', '.recording', '.sps'}
KIND = 'ffta-cold-storage'
GIB = 1024 ** 3
BLOCK = 1024 ** 2


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


def exists(path):
    return os.path.lexists(path)


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
        manifest, payload = entry / 'manifest.json', entry / 'payload.tar.zst'
        ordinary(manifest); ordinary(payload)
        if digest(manifest) != data['manifest_sha256'] or digest(payload) != data['archive_sha256']:
            raise ValueError('Cold archive/catalog checksum mismatch')
        rows = json.loads(manifest.read_text())
        check_manifest(rows)
        return rows

    def pack(self, path, *, reason, proof, floor=5 * GIB, before_retire=None):
        self.check()
        source, rel = self.source(path)
        if not reason.strip() or not proof:
            raise ValueError('Archiving needs a reason and completion/ownership evidence')
        for _, data in self.entries():
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
        read_archive(entry / 'payload.tar.zst', rows, entry / 'staging')
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
        read_archive(entry / 'payload.tar.zst', rows)
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


def host_activity(source):
    """Bounded legacy-source check; never infer host inactivity from sandbox ps."""
    if os.name != 'posix' or not Path('/proc/1/comm').exists() or Path('/proc/1/comm').read_text().strip() != 'systemd':
        raise ValueError('Historical archival requires the host PID namespace; sandbox process visibility is insufficient')
    excluded = {os.getpid()}
    pid = os.getppid()
    while pid and pid not in excluded:
        excluded.add(pid)
        try:
            status = Path(f'/proc/{pid}/status').read_text()
            pid = int(next(line.split()[1] for line in status.splitlines() if line.startswith('PPid:')))
        except (OSError, StopIteration):
            break
    needles = [str(source)]
    if source.is_relative_to(ROOT):
        needles.append(source.relative_to(ROOT).as_posix())
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


def prepare_review(store, path, completions, scope):
    source, rel = store.source(path)
    scope = Path(os.path.abspath(scope))
    if source == scope or scope not in source.parents or not scope.is_relative_to(store.root / 'target'):
        raise ValueError('Activity scope must be an ancestor below target, outside the archived payload')
    ordinary(scope, directory=True)
    if (scope / NAME).exists():
        raise ValueError('Activity scope already leased')
    ensure_untracked(source, store.root)
    evidence = []
    for p in completions:
        p = Path(os.path.abspath(p))
        if p == source or source in p.parents:
            raise ValueError('Completion evidence must remain readable outside the archive')
        evidence.append({'path': p.relative_to(store.root).as_posix(), 'sha256': completion(p)})
    if not evidence:
        raise ValueError('Completion evidence is required')
    rows = scan(source)
    return {'status': 'PASS', 'source': rel, 'scope': scope.relative_to(store.root).as_posix(),
            'inventory_sha256': inventory_digest(rows), 'logical_bytes': size(rows),
            'allocated_bytes': size(rows, 'allocated'), 'completion': evidence,
            'idle': host_activity(scope)}


def apply_review(store, review, reason):
    source, rel = store.source(review['source'])
    if review.get('status') != 'PASS' or not review.get('inventory_sha256') or not review.get('idle'):
        raise ValueError('Reviewed inventory, completion and host activity evidence required')
    scope = store.root / relative(review['scope'])
    if source == scope or scope not in source.parents or not scope.is_relative_to(store.root / 'target'):
        raise ValueError('Invalid archive activity scope')
    if not review.get('completion'):
        raise ValueError('Missing completion evidence')
    for evidence in review['completion']:
        p = store.root / relative(evidence['path'])
        if p == source or source in p.parents or completion(p) != evidence['sha256']:
            raise ValueError('Completion evidence changed or would be archived')
    with Lease(scope, 'cold-storage-historical-source'):
        idle = host_activity(scope)
        proof = dict(review, idle=idle)
        return store.pack(source, reason=reason, proof=proof, before_retire=lambda: host_activity(scope))


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
    for name in ['restore', 'retire', 'verify', 'recover']:
        p = sub.add_parser(name); p.add_argument('path')
    plan = sub.add_parser('plan')
    plan.add_argument('path'); plan.add_argument('--scope', required=True, type=Path)
    plan.add_argument('--completion', action='append', type=Path, required=True)
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
            if args.action == 'list':
                print(json.dumps([{k: d.get(k) for k in ['source', 'state', 'logical_bytes', 'archive_bytes']}
                                  for _, d in store.entries()], indent=2)); return 0
            source, rel = store.source(args.path)
            if args.action == 'plan':
                review = prepare_review(store, source, args.completion, args.scope)
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
            for entry, data in matches:
                if args.action == 'restore': store.restore(entry, data)
                elif args.action == 'retire':
                    scope = data.get('proof', {}).get('scope', data.get('proof', {}).get('unit', data['source']))
                    host_activity(store.root / relative(scope)); store.retire(entry, data)
                elif args.action == 'recover': store.recover(entry, data)
                else: read_archive(entry / 'payload.tar.zst', store.load(entry, data))
                print(f'PASS {args.action}: {data["source"]}')
        return 0
    except (OSError, ValueError, KeyError, tarfile.TarError, subprocess.SubprocessError) as error:
        print(f'Cold storage refused or retained for review: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
