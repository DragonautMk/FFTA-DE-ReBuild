#!/usr/bin/env python3
"""Validate the local Docker/host bridge before opening the archive store."""
import os
from pathlib import Path
import sys

import cold_storage


def main():
    try:
        expected = os.environ['FFTA_STORAGE_PID_NAMESPACE']
        if os.readlink('/proc/self/ns/pid') != expected:
            raise ValueError('The container must share the local host PID namespace')
        pid = int(os.environ['FFTA_STORAGE_DRIVER_PID'])
        driver = Path('/proc') / str(pid)
        if driver.stat().st_uid != os.getuid():
            raise ValueError('Docker must use the invoking host UID without remapping')
        argv = (driver / 'cmdline').read_bytes().split(b'\0')
        name = os.environ['FFTA_STORAGE_CONTAINER_NAME'].encode()
        if (Path(os.fsdecode(argv[0])).name != 'docker' or b'run' not in argv
                or not any(argv[i:i + 2] == [b'--name', name] for i in range(len(argv)))
                or os.uname().nodename.encode() != name):
            raise ValueError('The host driver does not identify this container')
        # Reading command lines alone misses a process holding an artifact open.
        # Fail closed if the container's security profile hides host details.
        (driver / 'environ').read_bytes()
        (driver / 'maps').read_bytes()
        os.readlink(driver / 'cwd')
        os.readlink(driver / 'fd/0')
        # Docker's client is not an ancestor of containerd's child. Exclude only
        # this authenticated coordinator and its ancestors, as for native use.
        cold_storage.HOST_DRIVER_PID = pid
    except (OSError, ValueError, KeyError, IndexError) as error:
        print(f'Cold storage container refused: {error}. Use the Linux Docker launcher.', file=sys.stderr)
        return 1
    return cold_storage.main()


if __name__ == '__main__':
    sys.exit(main())
