#!/usr/bin/env python3
"""Build the ARM64 preview with Docker/Buildx; no host SDK, Java or Rust needed."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = Path(__file__).resolve().parent
APK_NAME = 'FFTARecomp-android-arm64-preview.apk'
SKIP_DIRS = {'.git', '.gradle', '.kotlin', '.idea', '.agents', '.codex', '.aws',
             '__pycache__', 'target', 'build', 'Data', 'rom', 'ROMs', 'jniLibs', 'node_modules'}
SOURCE_SUFFIXES = {'.rs', '.toml', '.lock', '.py', '.c', '.h', '.inc', '.wgsl', '.tsv', '.json',
                   '.kt', '.kts', '.xml', '.properties', '.pro', '.png', '.webp', '.svg',
                   '.ttf', '.md', '.txt'}


def project_layout(project):
    if (project / 'scripts/build_android.py').is_file():
        return 'scripts/build_android.py', 'androidport'
    if (project / 'scripts/android/build_android.py').is_file():
        return 'scripts/android/build_android.py', 'root'
    raise ValueError(f'No supported Android builder under {project}')


def source_allowed(relative):
    if any(part in SKIP_DIRS or part.startswith('.') for part in relative.parts):
        return False
    if relative.name in {'local.properties', 'gba_bios.bin', 'shared-save-path.txt'}:
        return False
    # These files are generated from pinned source by the native builder.
    if relative.as_posix().startswith('android/app/src/main/assets/licenses/'):
        return False
    if relative.as_posix() == 'android/gradle/wrapper/gradle-wrapper.jar':
        return True
    return relative.suffix.lower() in SOURCE_SUFFIXES or relative.as_posix() == 'android/gradlew'


def stage(project, context):
    """Copy public working-tree sources only; never bind-mount or modify the checkout."""
    script, scope = project_layout(project)
    output = context / 'project'
    output.mkdir(parents=True)
    paths = [project / 'Cargo.toml', project / 'Cargo.lock', project / script]
    for directory in ['crates', 'android', 'Licenses']:
        for parent, dirs, files in os.walk(project / directory, followlinks=False):
            dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS and not d.startswith('.') and not (Path(parent) / d).is_symlink())
            paths.extend(Path(parent) / name for name in sorted(files))
    entries = {}
    for path in paths:
        relative = path.relative_to(project)
        if path.is_symlink() or not path.is_file() or not source_allowed(relative):
            continue
        destination = output / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        data = path.read_bytes()
        if relative.as_posix() == 'android/gradlew':
            data = data.replace(b'\r\n', b'\n')
        destination.write_bytes(data)
        entries[relative.as_posix()] = hashlib.sha256(data).hexdigest()
    for required in ['Cargo.toml', 'Cargo.lock', script, 'crates/gba-android/Cargo.toml',
                     'android/app/build.gradle.kts', 'android/gradlew',
                     'android/gradle/wrapper/gradle-wrapper.jar']:
        if required not in entries:
            raise ValueError(f'Missing public build input: {project / required}')
    digest = hashlib.sha256(json.dumps(entries, sort_keys=True).encode()).hexdigest()
    metadata = {'project': scope, 'source_sha256': digest, 'files': entries}
    (output / 'BUILD-SOURCE.json').write_text(json.dumps(metadata, indent=2) + '\n', encoding='utf-8')
    shutil.copyfile(ROOT / 'Dockerfile.android', context / 'Dockerfile')
    shutil.copyfile(SCRIPTS / 'export_android.py', context / 'export_android.py')
    return script, scope, metadata


def stream(command, log):
    private_paths = []
    for index, argument in enumerate(command[:-1]):
        if argument == '--secret':
            for option in command[index + 1].split(','):
                if option.startswith('src='):
                    private_paths.append(option[4:])
    with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                          text=True, encoding='utf-8', errors='replace') as process:
        for line in process.stdout:
            for path in private_paths:
                line = line.replace(path, '<local signing file>')
            print(line, end='', flush=True)
            log.write(line)
            log.flush()
        return process.wait()


def prepare_toolchain(context, log):
    builder = subprocess.check_output(['docker', 'buildx', 'inspect'], text=True)
    driver = next((line.split(':', 1)[1].strip() for line in builder.splitlines()
                   if line.startswith('Driver:')), None)
    if driver != 'docker':
        return []
    recipe = context / 'Dockerfile'
    prefix, marker, _ = recipe.read_text(encoding='utf-8').partition('# End of cached toolchain\n')
    if not marker:
        raise ValueError('Missing toolchain boundary in Dockerfile.android')
    image = 'ffta-toolchain-android:' + hashlib.sha256(prefix.encode()).hexdigest()[:20]
    if not subprocess.check_output(['docker', 'image', 'ls', '--quiet', image]).strip():
        build = ['docker', 'buildx', 'build', '--platform', 'linux/amd64', '--target',
                 'toolchain', '--file', str(recipe), '--output', 'type=image,unpack=false',
                 '--tag', image, str(context)]
        status = stream(build, log)
        if status:
            raise subprocess.CalledProcessError(status, build)
    return ['--build-arg', f'FFTA_TOOLCHAIN={image}']


def public_command(command):
    """Keep local signing-file paths out of durable build logs."""
    result = list(command)
    for index, argument in enumerate(result[:-1]):
        if argument == '--secret':
            secret_id = result[index + 1].split(',', 1)[0]
            result[index + 1] = secret_id + ',src=<local signing file>'
    return result


def main(argv=None, *, default_project=None, default_output=None, log_root=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-dir', type=Path, default=default_project or ROOT,
                        help='Android source tree; defaults to this self-contained port')
    parser.add_argument('--output', type=Path, default=default_output or ROOT / 'dist/docker-android',
                        help='APK, checksums and build/test evidence destination')
    parser.add_argument('--dry-run', action='store_true', help='Validate public inputs and show the build command without Docker')
    parser.add_argument('--keep-failed-context', action='store_true',
                        help='Keep a copy of the staged public sources if the APK build fails')
    parser.add_argument('--signing-keystore', type=Path,
                        help='Persistent preview keystore (alias: ffta-preview); sent as a BuildKit secret')
    parser.add_argument('--signing-password-file', type=Path,
                        help='Private file containing the keystore and key password')
    args = parser.parse_args(argv)
    project = args.project_dir.resolve()
    output = args.output.resolve()
    if any(ch in str(output) for ch in ',"\r\n'):
        parser.error('output must not contain commas, double quotes or newlines (Buildx CSV syntax)')
    if (args.signing_keystore is None) != (args.signing_password_file is None):
        parser.error('provide both --signing-keystore and --signing-password-file')
    signing_args = []
    if args.signing_keystore is not None:
        keystore = args.signing_keystore.resolve()
        password = args.signing_password_file.resolve()
        if any(any(ch in str(path) for ch in ',"\r\n') for path in (keystore, password)):
            parser.error('signing file paths must not contain commas, double quotes or newlines')
        if not keystore.is_file() or not password.is_file():
            parser.error('both signing files must exist; never regenerate a missing release key automatically')
        if keystore.is_relative_to(project) or password.is_relative_to(project):
            parser.error('signing files must be outside the staged Android source tree')
        if keystore == password or keystore.samefile(password):
            parser.error('keystore and password must be different files')
        if keystore.stat().st_size > 1024 * 1024:
            parser.error('preview keystore is unexpectedly large')
        signer_input = hashlib.sha256(keystore.read_bytes()).hexdigest()
        # BuildKit does not include secrets in its cache key. A fresh nonce
        # re-runs the signing step while preserving its Gradle cache mount.
        signing_args = ['--build-arg', 'FFTA_PREVIEW_SIGNING=1',
                        '--build-arg', f'FFTA_PREVIEW_SIGNER_INPUT={signer_input}',
                        '--build-arg', f'FFTA_PREVIEW_SIGNING_NONCE={secrets.token_hex(16)}',
                        '--secret', f'id=ffta_preview_keystore,src={keystore}',
                        '--secret', f'id=ffta_preview_password,src={password}']
    if not args.dry_run and not shutil.which('docker'):
        parser.error('Docker with Buildx is required')
    try:
        with tempfile.TemporaryDirectory(prefix='ffta-android-context-') as temporary:
            context = Path(temporary)
            script, scope, metadata = stage(project, context)
            if signing_args and scope != 'androidport':
                raise ValueError('persistent preview signing is supported only for AndroidPort')
            command = ['docker', 'buildx', 'build', '--platform', 'linux/amd64',
                       '--file', str(context / 'Dockerfile'), '--target', 'export',
                       '--build-arg', f'BUILD_SCRIPT={script}',
                       '--build-arg', f'CACHE_SCOPE={scope}', '--output', f'type=local,dest={output}',
                       '--progress', 'plain', str(context)]
            command[-1:-1] = signing_args
            print(f"Android project: {project}\nPublic source files: {len(metadata['files'])}\nSource SHA256: {metadata['source_sha256']}", flush=True)
            if args.dry_run:
                print(json.dumps(public_command(command), indent=2))
                return 0
            output.mkdir(parents=True, exist_ok=True)
            logs = Path(log_root or project / 'target/build-logs')
            logs.mkdir(parents=True, exist_ok=True)
            timestamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')
            with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', prefix=f'android-{timestamp}-',
                                             suffix='.log', dir=logs, delete=False) as log:
                print(f'Android Docker build log: {log.name}', flush=True)
                command[3:3] = prepare_toolchain(context, log)
                log.write(json.dumps({'command': public_command(command), 'source': metadata}) + '\n')
                status = stream(command, log)
                if status:
                    if args.keep_failed_context:
                        failed_context = logs / (Path(log.name).stem + '-context')
                        try:
                            shutil.copytree(context, failed_context)
                            log.write(f'Failed public build context retained: {failed_context}\n')
                        except OSError as error:
                            log.write(f'Could not retain failed context: {error}\n')
                    print(f'Android build failed (exit {status}). Log retained: {log.name}', file=sys.stderr)
                    return status
            artifact = output / APK_NAME
            actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
            expected = (output / 'SHA256.txt').read_text().split()[0]
            if actual != expected:
                raise ValueError('Exported APK checksum mismatch')
            latest = logs / f'android-{scope}-latest.log'
            os.replace(log.name, latest)
            print(f'Build log: {latest}', flush=True)
            print(f'APK: {artifact}\nSHA256: {actual}', flush=True)
            return 0
    except subprocess.CalledProcessError as error:
        print(f'Android Docker build: {error}', file=sys.stderr)
        return error.returncode or 1
    except (OSError, ValueError) as error:
        print(f'Android Docker build: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
