"""Directory leases shared by Python and the isolated Rust runtime (directory-v1).

Atomic mkdir coordinates across native Windows, Wine, Python and container bind
mounts without assuming their OS file-lock implementations interoperate. A dead
owner leaves a protective lease; there is no automatic stale-lock reclamation.
"""
import json
import os
from pathlib import Path
import stat
import uuid

NAME = '.debug-artifacts-lease'
PROTOCOL = 'directory-v1'


def ordinary(path, *, directory=False):
    path = Path(os.path.abspath(path))
    for parent in reversed(path.parents):
        s = parent.lstat()
        if not stat.S_ISDIR(s.st_mode) or getattr(s, 'st_file_attributes', 0) & 0x400:
            raise ValueError(f'Nonordinary ancestor refused: {parent}')
    s = path.lstat()
    if (getattr(s, 'st_file_attributes', 0) & 0x400
            or (not stat.S_ISDIR(s.st_mode) if directory else not stat.S_ISREG(s.st_mode))
            or (not directory and s.st_nlink != 1)):
        raise ValueError(f'Nonordinary/shared path refused: {path}')
    return s


class Lease:
    def __init__(self, root, role):
        self.root = Path(os.path.abspath(root))
        self.path = self.root / NAME
        self.owner = {'schema': 1, 'protocol': PROTOCOL, 'token': uuid.uuid4().hex,
                      'pid': os.getpid(), 'role': role}
        self.held = False

    def __enter__(self):
        ordinary(self.root, directory=True)
        try:
            self.path.mkdir()
        except FileExistsError as error:
            raise ValueError(f'Active or interrupted artifact lease: {self.path}; inspect its owner, never reclaim automatically') from error
        # A failed owner write intentionally leaves diagnostic protection.
        with (self.path / 'owner.json').open('x', encoding='utf-8') as f:
            json.dump(self.owner, f, separators=(',', ':'))
            f.flush()
            os.fsync(f.fileno())
        self.held = True
        return self

    def check(self, root=None):
        if root is not None and Path(os.path.abspath(root)) != self.root:
            raise ValueError('Lease belongs to a different root')
        ordinary(self.root, directory=True)
        ordinary(self.path, directory=True)
        owner = self.path / 'owner.json'
        ordinary(owner)
        with owner.open('rb') as f:
            data = f.read(4097)
        if not self.held or len(data) > 4096 or json.loads(data) != self.owner:
            raise ValueError('Artifact lease ownership changed; diagnostic lease retained')

    def __exit__(self, *_):
        self.check()
        (self.path / 'owner.json').unlink()
        self.path.rmdir()
        self.held = False
