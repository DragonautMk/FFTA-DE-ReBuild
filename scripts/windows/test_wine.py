#!/usr/bin/env python3
"""Run the Windows container test gate with UTF-8 and real filesystem aliases.

The caller owns the container's Wine prefix. This is not a host-prefix runner;
host validation must use validation_unit.py and its shared-prefix lease.
"""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def run(command):
    if not command:
        raise ValueError('Supply the foreground Windows test command')
    if os.name != 'posix':
        raise ValueError('This test helper requires a Linux container')
    env = dict(os.environ, LANG='C.UTF-8', LC_ALL='C.UTF-8')
    if not env.get('WINEPREFIX'):
        raise ValueError('The container must provide its owned WINEPREFIX')
    server = shutil.which(env.get('WINESERVER', 'wineserver'))
    if not server:
        raise ValueError('The container must provide a matching WINESERVER')
    fixture = Path(tempfile.mkdtemp(prefix='ffta-wine-tests-')).resolve()
    external = fixture / 'external'
    inputs = fixture / 'inputs'
    external.mkdir()
    inputs.mkdir()
    source = external / 'outside.py'
    source.write_bytes(b'PUBLIC SYNTHETIC FIXTURE: DO NOT READ OR RUN\n')
    (inputs / 'linked.py').symlink_to(source)
    (inputs / 'linked-dir').symlink_to(external, target_is_directory=True)
    os.link(source, inputs / 'shared.py')
    # Wine's CreateSymbolicLinkW can report success without creating a link.
    # Supply genuine aliases to the existing negative test; never skip it.
    assert (inputs / 'linked.py').is_symlink()
    assert (inputs / 'linked-dir').is_symlink()
    assert (inputs / 'shared.py').stat().st_nlink == 2
    assert os.path.samefile(source, inputs / 'shared.py')
    env['FFTA_DEBUG_CONTENT_LINK_FIXTURE'] = 'Z:' + str(inputs).replace('/', '\\')
    print(f'Wine tests: UTF-8 locale; real link fixture: {fixture}', flush=True)
    result = subprocess.run(command, env=env)
    # Keep the fixture available until every Wine consumer has finished, also
    # after a nonzero test result. Interrupted/failed runs retain diagnostics.
    subprocess.run([server, '-w'], env=env, check=True, timeout=120)
    if result.returncode == 0:
        shutil.rmtree(fixture)
    else:
        print(f'Wine tests failed (exit {result.returncode}); fixture retained: {fixture}',
              file=sys.stderr, flush=True)
    return result.returncode


if __name__ == '__main__':
    try:
        raise SystemExit(run(sys.argv[1:]))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise SystemExit(f'Wine test setup or execution failed: {error}')
