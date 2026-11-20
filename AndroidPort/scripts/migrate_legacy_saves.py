#!/usr/bin/env python3
"""Copy 1.1.2 Android preview saves once through a locally connected ADB device."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile

OLD = 'com.fftarecomp.port'
NEW = 'com.fftarecomp.port.preview'
LOCK = 'files/.ffta-legacy-migration.lock'
MARKER = 'files/.ffta-legacy-migration-complete.json'
MAX_FILE = 65 * 1024 * 1024
SAVE_PATH = re.compile(
    r'files/(?:ffta\.sav|SaveStates/[0-9a-f]{64}/(?:slot-[1-9]\.ffstate|'
    r'ImportBackups/slot-[1-9]-[0-9a-f]{64}\.ffstate)|'
    r'SaveBackups/[0-9a-f]{64}/ffta\.sav/backup-[0-9]{20}-[0-9a-f]{64}\.sav)\Z'
)


class MigrationError(Exception):
    pass


def digest(data):
    return hashlib.sha256(data).hexdigest()


def inside_git(path):
    return any((parent / '.git').exists() for parent in (path, *path.parents))


def quiet(bridge):
    if bridge.running(OLD) or bridge.running(NEW):
        raise MigrationError('Close both previews completely before migrating saves.')


def save_paths(bridge, package):
    paths = bridge.files(package)
    selected = []
    for path in paths:
        if path == 'files/ffta.sav' or path.startswith(('files/SaveStates/', 'files/SaveBackups/')):
            if not SAVE_PATH.fullmatch(path):
                raise MigrationError('An unsupported save path was found; refusing a partial migration.')
            selected.append(path)
    return sorted(set(selected))


def snapshot(bridge, package, paths, directory):
    values = {}
    for path in paths:
        data = bridge.read(package, path)
        if len(data) > MAX_FILE or digest(data) != bridge.hash(package, path):
            raise MigrationError('A save changed or exceeded the size limit while being backed up.')
        if path.startswith('files/SaveBackups/'):
            expected = path.rsplit('-', 1)[-1].removesuffix('.sav')
            if digest(data) != expected:
                raise MigrationError('A cartridge backup does not match its recorded checksum.')
        if '/ImportBackups/' in path:
            expected = path.rsplit('-', 1)[-1].removesuffix('.ffstate')
            if digest(data) != expected:
                raise MigrationError('A state import backup does not match its recorded checksum.')
        destination = directory / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, 'wb') as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        if digest(destination.read_bytes()) != digest(data):
            raise MigrationError('A local backup failed its checksum.')
        values[path] = data
    return values


def unchanged(bridge, package, values):
    return all(bridge.hash(package, path) == digest(data) for path, data in values.items())


def migrate(bridge, backup_root):
    quiet(bridge)
    if bridge.exists(NEW, MARKER):
        return {'status': 'already_migrated', 'copied': 0}
    old_rom = bridge.hash(OLD, 'files/ffta.gba')
    if not old_rom or old_rom != bridge.hash(NEW, 'files/ffta.gba'):
        raise MigrationError('Import the same ROM in both previews before migrating saves.')
    old_paths, new_paths = save_paths(bridge, OLD), save_paths(bridge, NEW)
    if 'files/ffta.sav' not in old_paths:
        raise MigrationError('The old preview has no cartridge save to migrate.')
    backup_root = Path(backup_root)
    backup_root.mkdir(parents=True, exist_ok=True, mode=0o700)
    backup = Path(tempfile.mkdtemp(prefix='android-preview-', dir=backup_root))
    completed = []
    locked = False
    marker_written = False
    try:
        bridge.mkdir_exclusive(NEW, LOCK)
        locked = True
        quiet(bridge)
        if bridge.exists(NEW, MARKER):
            bridge.remove_tree(NEW, LOCK)
            locked = False
            return {'status': 'already_migrated', 'copied': 0}
        source = snapshot(bridge, OLD, old_paths, backup / 'old')
        previous = snapshot(bridge, NEW, new_paths, backup / 'new-before')
        manifest = {
            'schema': 1, 'created_utc': datetime.now(timezone.utc).isoformat(),
            'old_package': OLD, 'new_package': NEW, 'rom_sha256': old_rom,
            'old': {p: digest(b) for p, b in source.items()},
            'new_before': {p: digest(b) for p, b in previous.items()},
        }
        manifest_path = backup / 'manifest.json'
        manifest_path.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
        os.chmod(manifest_path, 0o600)
        if (save_paths(bridge, OLD) != old_paths or save_paths(bridge, NEW) != new_paths
                or not unchanged(bridge, OLD, source) or not unchanged(bridge, NEW, previous)):
            raise MigrationError('A save changed during backup; nothing was imported.')
        if bridge.hash(OLD, 'files/ffta.gba') != old_rom or bridge.hash(NEW, 'files/ffta.gba') != old_rom:
            raise MigrationError('A ROM changed during backup; nothing was imported.')
        changed = [path for path in old_paths if previous.get(path) != source[path]]
        # Import the normal save last so an incomplete transfer cannot start it early.
        changed.sort(key=lambda p: (p == 'files/ffta.sav', p))
        staged = {}
        for index, path in enumerate(changed):
            quiet(bridge)
            temp = f'{LOCK}/item-{index}'
            bridge.write_new(NEW, temp, source[path])
            if bridge.hash(NEW, temp) != digest(source[path]):
                raise MigrationError('A staged save failed its checksum.')
            staged[path] = temp
        quiet(bridge)
        if (save_paths(bridge, OLD) != old_paths or save_paths(bridge, NEW) != new_paths
                or not unchanged(bridge, OLD, source) or not unchanged(bridge, NEW, previous)):
            raise MigrationError('A save changed before import; nothing was replaced.')
        for path in changed:
            quiet(bridge)
            expected = digest(previous[path]) if path in previous else None
            if bridge.hash(NEW, path) != expected:
                raise MigrationError('A destination save changed during import.')
            bridge.replace(NEW, staged[path], path)
            completed.append(path)
            if bridge.hash(NEW, path) != digest(source[path]):
                raise MigrationError('An imported save failed its checksum.')
        if not unchanged(bridge, OLD, source) or not unchanged(bridge, NEW, source):
            raise MigrationError('A save changed during import.')
        quiet(bridge)
        marker = json.dumps({'schema': 1, 'completed_utc': manifest['created_utc'],
                             'old': manifest['old']}, sort_keys=True).encode() + b'\n'
        bridge.write_new(NEW, f'{LOCK}/marker', marker)
        if bridge.exists(NEW, MARKER):
            raise MigrationError('A migration marker appeared during import.')
        bridge.replace(NEW, f'{LOCK}/marker', MARKER)
        marker_written = True
        bridge.remove_tree(NEW, LOCK)
        locked = False
        return {'status': 'migrated', 'copied': len(changed), 'backup': str(backup)}
    except Exception as error:
        rollback_error = None
        try:
            if marker_written:
                raise MigrationError('Migration marker was published; inspect the saved backup.')
            for path in reversed(completed):
                quiet(bridge)
                if bridge.hash(NEW, path) != digest(source[path]):
                    raise MigrationError('An imported save changed; automatic rollback stopped.')
                if path in previous:
                    temp = f'{LOCK}/rollback-{len(completed)}'
                    bridge.write_new(NEW, temp, previous[path])
                    if bridge.hash(NEW, temp) != digest(previous[path]):
                        raise MigrationError('Rollback backup failed its checksum.')
                    bridge.replace(NEW, temp, path)
                else:
                    bridge.remove(NEW, path)
            if locked:
                bridge.remove_tree(NEW, LOCK)
                locked = False
        except Exception as failure:
            rollback_error = failure
        if rollback_error:
            raise MigrationError(f'Import interrupted; preserve backup {backup} and lock {LOCK}: {rollback_error}') from error
        raise MigrationError(f'Import stopped; this tool rolled back its changes where needed. Backup: {backup}. {error}') from error


class Adb:
    def __init__(self, serial):
        self.prefix = ['adb'] + (['-s', serial] if serial else [])

    def call(self, *args, data=None, check=True):
        result = subprocess.run(self.prefix + list(args), input=data, capture_output=True, timeout=120)
        if check and result.returncode:
            raise MigrationError('ADB could not complete an operation; check the connected device and app permissions.')
        return result

    def shell(self, *args, data=None, check=True):
        return self.call('shell', '-T', *args, data=data, check=check)

    def running(self, package):
        result = self.shell('pidof', package, check=False)
        if result.returncode not in (0, 1):
            raise MigrationError('Cannot determine whether an app is running.')
        return bool(result.stdout.strip())

    def files(self, package):
        return self.shell('run-as', package, 'find', 'files', '-type', 'f').stdout.decode().splitlines()

    def exists(self, package, path):
        result = self.shell('run-as', package, 'test', '-e', path, check=False)
        if result.returncode not in (0, 1):
            raise MigrationError('Cannot inspect an app file.')
        return result.returncode == 0

    def hash(self, package, path):
        if not self.exists(package, path):
            return None
        output = self.shell('run-as', package, 'sha256sum', path).stdout.decode().split()
        if not output or not re.fullmatch(r'[0-9a-f]{64}', output[0]):
            raise MigrationError('Cannot verify an app file checksum.')
        return output[0]

    def read(self, package, path):
        size = self.shell('run-as', package, 'stat', '-c', '%s', path).stdout.decode().strip()
        if not size.isdigit() or int(size) > MAX_FILE:
            raise MigrationError('A save exceeds the size limit.')
        data = self.call('exec-out', 'run-as', package, 'cat', path).stdout
        if len(data) != int(size):
            raise MigrationError('A save changed while being read.')
        return data

    def mkdir_exclusive(self, package, path):
        if path != LOCK:
            raise MigrationError('Refusing to create an unexpected lock.')
        self.shell('run-as', package, 'mkdir', path)

    def write_new(self, package, path, data):
        if not re.fullmatch(re.escape(LOCK) + r'/(?:item-[0-9]+|rollback-[0-9]+|marker|probe)', path):
            raise MigrationError('Refusing an unexpected staging path.')
        command = f'umask 077; set -C; cat > {shlex.quote(path)}'
        self.shell('run-as', package, 'sh', '-c', shlex.quote(command), data=data)

    def replace(self, package, source, destination):
        if not source.startswith(LOCK + '/') or (destination != MARKER and not SAVE_PATH.fullmatch(destination)):
            raise MigrationError('Refusing an unexpected import path.')
        parts = destination.split('/')[:-1]
        for index in range(1, len(parts) + 1):
            parent = '/'.join(parts[:index])
            result = self.shell('run-as', package, 'test', '-L', parent, check=False)
            if result.returncode == 0:
                raise MigrationError('A destination directory is a symbolic link.')
            if result.returncode != 1:
                raise MigrationError('Cannot verify a destination directory.')
        self.shell('run-as', package, 'mkdir', '-p', str(Path(destination).parent).replace('\\', '/'))
        self.shell('run-as', package, 'mv', '-f', source, destination)

    def remove(self, package, path):
        if not SAVE_PATH.fullmatch(path):
            raise MigrationError('Refusing to remove an unexpected save.')
        self.shell('run-as', package, 'rm', path)

    def remove_tree(self, package, path):
        if path != LOCK:
            raise MigrationError('Refusing to remove an unexpected directory.')
        self.shell('run-as', package, 'rm', '-r', path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--serial', help='ADB device serial; required when multiple devices are connected')
    parser.add_argument('--backup-root', type=Path, required=True,
                        help='Private directory outside the source tree for verified save backups')
    args = parser.parse_args()
    backup_root = args.backup_root.resolve()
    port_root = Path(__file__).resolve().parent.parent
    if ((port_root / 'Cargo.toml').is_file() and (port_root / 'android').is_dir()
            and backup_root.is_relative_to(port_root)):
        parser.error('the backup directory must be outside the Android source tree')
    if inside_git(backup_root):
        parser.error('the backup directory must be outside Git repositories')
    try:
        bridge = Adb(args.serial)
        if bridge.call('get-state').stdout.strip() != b'device':
            raise MigrationError('Connect and authorize exactly one Android device through ADB.')
        result = migrate(bridge, backup_root)
        if result['status'] == 'already_migrated':
            print('The legacy saves were already migrated; no files were changed.')
        else:
            print(f"Migrated {result['copied']} save files. Verified private backup: {result['backup']}")
        return 0
    except (MigrationError, OSError, subprocess.TimeoutExpired, UnicodeError) as error:
        print(f'Android save migration: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
