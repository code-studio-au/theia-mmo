"""Package and install trusted local Theia client builds for development.

A package is a manifest beside a payload directory. The hashes bind the copy
to that manifest. Remote publishers need signed release metadata before their
packages can be admitted.
"""
import argparse
from contextlib import contextmanager, nullcontext
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
import sys
import tempfile

MANIFEST = 'client-release.json'
ACTIVE = 'active.json'
MAX_MANIFEST_BYTES = 4 * 1024 * 1024
MAX_FILES = 20_000
MAX_TOTAL_BYTES = 50 * 1024**3
CHUNK_BYTES = 1024 * 1024
WINDOWS_RESERVED = frozenset(
    {'con', 'prn', 'aux', 'nul'} | {f'{prefix}{i}' for prefix in ('com', 'lpt') for i in range(10)}
)


class InstallError(ValueError):
    pass


def target():
    machine = platform.machine().lower()
    if machine in ('amd64', 'x86_64') and sys.platform in ('linux', 'win32'):
        return 'windows-x86_64' if sys.platform == 'win32' else 'linux-x86_64'
    raise InstallError('Unsupported client install platform')


def executable(host):
    return 'theia-client.exe' if host == 'windows-x86_64' else 'theia-client'


def version(value):
    if not isinstance(value, str) or not re.fullmatch(r'(0|[1-9][0-9]{0,7})(\.(0|[1-9][0-9]{0,7})){2,3}', value):
        raise InstallError('Use a client version such as 0.1.0.12')
    return value


def relative(value):
    if (not isinstance(value, str) or len(value) > 240
            or not re.fullmatch(r'[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*', value)):
        raise InstallError('Invalid release file path')
    for part in value.split('/'):
        if (part.startswith('.') or part.endswith('.')
                or part.split('.')[0].casefold() in WINDOWS_RESERVED):
            raise InstallError('Invalid release file path')
    return value


def regular(path):
    mode = path.lstat().st_mode
    if not stat.S_ISREG(mode) or path.is_symlink() or path.resolve() != path:
        raise InstallError('Release files must be regular files inside their package')
    return path


def read_json(path, limit):
    with regular(path).open('rb') as stream:
        raw = stream.read(limit + 1)
    if len(raw) > limit:
        raise InstallError('Release metadata exceeds its size limit')

    def unique(fields):
        value = dict(fields)
        if len(value) != len(fields):
            raise InstallError('Release metadata contains duplicate fields')
        return value

    try:
        return json.loads(raw, object_pairs_hook=unique)
    except (UnicodeError, json.JSONDecodeError, RecursionError) as error:
        raise InstallError('Invalid release metadata') from error


def metadata(directory):
    data = read_json(directory / MANIFEST, MAX_MANIFEST_BYTES)
    if (not isinstance(data, dict) or set(data) != {'schema', 'version', 'target', 'files'}
            or type(data['schema']) is not int or data['schema'] != 1):
        raise InstallError('Unknown release metadata format')
    version(data['version'])
    if data['target'] != target():
        raise InstallError('This client build targets another platform')
    files = data['files']
    if not isinstance(files, list) or not 1 <= len(files) <= MAX_FILES:
        raise InstallError('Invalid release file count')
    names = set()
    total = 0
    for row in files:
        if not isinstance(row, dict) or set(row) != {'path', 'bytes', 'sha256'}:
            raise InstallError('Invalid release file entry')
        path, size, digest = relative(row['path']), row['bytes'], row['sha256']
        key = path.casefold()
        if (key in names or type(size) is not int or not 0 <= size <= MAX_TOTAL_BYTES
                or not isinstance(digest, str) or not re.fullmatch('[a-f0-9]{64}', digest)):
            raise InstallError('Invalid or duplicate release file')
        names.add(key)
        total += size
        if total > MAX_TOTAL_BYTES:
            raise InstallError('Release exceeds its size limit')
    entry = executable(data['target'])
    if not any(row['path'] == entry and row['bytes'] > 0 for row in files):
        raise InstallError('Release is missing its client executable')
    for path in names:
        if any(parent.as_posix() in names for parent in Path(path).parents):
            raise InstallError('A release path is both a file and a directory')
    return data, total


def write_json(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())


def copy_file(source, destination, limit, advance):
    """Read at most the declared bytes and hash the bytes written to staging."""
    digest = hashlib.sha256()
    size = 0
    with regular(source).open('rb') as reader:
        output = destination.open('xb') if destination else nullcontext()
        with output as writer:
            while chunk := reader.read(min(CHUNK_BYTES, limit - size + 1)):
                if size + len(chunk) > limit:
                    raise InstallError('Release file grew or exceeds its declared size')
                if writer:
                    writer.write(chunk)
                digest.update(chunk)
                size += len(chunk)
                advance(len(chunk))
            if writer:
                writer.flush()
                os.fsync(writer.fileno())
    return size, digest.hexdigest()


def export(source, output, release_version):
    """Freeze a prepared client tree into a new local release directory."""
    release_version = version(release_version)
    source = source.resolve(strict=True)
    output = output.resolve()
    if not source.is_dir() or output.exists() or output.is_relative_to(source):
        raise InstallError('Choose a new package directory outside the client tree')
    host = target()
    regular(source / executable(host))
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.export-', dir=output.parent) as folder:
        stage = Path(folder) / 'package'
        payload = stage / 'payload'
        payload.mkdir(parents=True)
        rows, total = [], 0
        for path in sorted(source.rglob('*')):
            if path.is_dir() and not path.is_symlink():
                continue
            if len(rows) >= MAX_FILES:
                raise InstallError('Release exceeds its file count limit')
            name = relative(path.relative_to(source).as_posix())
            copy = payload / name
            copy.parent.mkdir(parents=True, exist_ok=True)
            length, digest = copy_file(path, copy, MAX_TOTAL_BYTES - total, lambda _: None)
            total += length
            rows.append({'path': name, 'bytes': length, 'sha256': digest})
        write_json(stage / MANIFEST, {'schema': 1, 'version': release_version, 'target': host, 'files': rows})
        metadata(stage)
        stage.rename(output)
    return output


def verify_installed_tree(directory, manifest):
    payload = directory / 'payload'
    names = {entry['path'] for entry in manifest['files']}
    directories = {parent.as_posix() for name in names for parent in Path(name).parents}
    for path in payload.rglob('*'):
        name = path.relative_to(payload).as_posix()
        if path.is_symlink() or not (name in directories if path.is_dir() else name in names):
            raise InstallError('Unexpected installed payload entry')


def verify(directory, manifest, total, output=None, progress=None, cancelled=None):
    written = 0

    def check_cancelled():
        if cancelled and cancelled():
            raise InstallError('Update cancelled')

    def advance(count):
        nonlocal written
        check_cancelled()
        written += count
        if progress:
            progress(written, total)

    advance(0)
    if output is None:
        verify_installed_tree(directory, manifest)
    for row in manifest['files']:
        check_cancelled()
        name = row['path']
        dest = output / 'payload' / name if output else None
        if dest:
            dest.parent.mkdir(parents=True, exist_ok=True)
        size, digest = copy_file(directory / 'payload' / name, dest, row['bytes'], advance)
        if size != row['bytes'] or digest != row['sha256']:
            raise InstallError(f'Release file did not match its manifest: {name}')
    check_cancelled()


@contextmanager
def install_lock(root):
    root.mkdir(parents=True, mode=0o700, exist_ok=True)
    with (root / '.install.lock').open('a+b') as stream:
        if sys.platform == 'win32':
            import msvcrt
            if stream.tell() == 0:
                stream.write(b'\0')
                stream.flush()
            stream.seek(0)
            try:
                msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError as error:
                raise InstallError('Another client update is running') from error
        else:
            import fcntl
            try:
                fcntl.flock(stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise InstallError('Another client update is running') from error
        # Closing the handle releases its OS lock, including after process exit.
        yield


def active(root):
    pointer = root / ACTIVE
    if not pointer.exists():
        return {'schema': 1, 'current': None, 'previous': None}
    data = read_json(pointer, 4096)
    if (not isinstance(data, dict) or set(data) != {'schema', 'current', 'previous'}
            or type(data['schema']) is not int or data['schema'] != 1
            or data['current'] == data['previous']):
        raise InstallError('Invalid active client record')
    version(data['current'])
    if data['previous'] is not None:
        version(data['previous'])
    return data


def activate(root, current, previous):
    with tempfile.TemporaryDirectory(prefix='.activate-', dir=root) as folder:
        path = Path(folder) / ACTIVE
        write_json(path, {'schema': 1, 'current': current, 'previous': previous})
        os.replace(path, root / ACTIVE)


def installed(root, release_version):
    directory = root / 'versions' / version(release_version)
    manifest, total = metadata(directory)
    if manifest['version'] != release_version:
        raise InstallError('Installed directory does not match the client version')
    return directory, manifest, total


def install(package, root, progress=None, cancelled=None):
    """Verify before switching the active version. Leave previous files intact."""
    package, root = package.resolve(strict=True), root.resolve()
    manifest, total = metadata(package)
    desired = manifest['version']
    with install_lock(root):
        previous = active(root)
        versions = root / 'versions'
        versions.mkdir(exist_ok=True)
        dest = versions / desired
        if dest.exists():
            directory, existing, _ = installed(root, desired)
            if existing != manifest:
                raise InstallError('This version already has a different release manifest')
            verify(directory, existing, total, progress=progress, cancelled=cancelled)
        else:
            with tempfile.TemporaryDirectory(prefix='.update-', dir=versions) as folder:
                stage = Path(folder) / 'release'
                stage.mkdir()
                verify(package, manifest, total, stage, progress, cancelled)
                write_json(stage / MANIFEST, manifest)
                entry = stage / 'payload' / executable(manifest['target'])
                if sys.platform != 'win32':
                    entry.chmod(0o755)
                stage.rename(dest)
        if cancelled and cancelled():
            raise InstallError('Update cancelled')
        if desired != previous['current']:
            activate(root, desired, previous['current'])
    return desired


def rollback(root):
    root = root.resolve()
    with install_lock(root):
        old = active(root)
        if old['previous'] is None:
            raise InstallError('No previous client build is available')
        directory, manifest, total = installed(root, old['previous'])
        verify(directory, manifest, total)
        activate(root, old['previous'], old['current'])
        return old['previous']


def launch(root, args):
    root = root.resolve()
    release_version = active(root)['current']
    if release_version is None:
        raise InstallError('Install a client build first')
    directory, manifest, total = installed(root, release_version)
    verify(directory, manifest, total)
    payload = directory / 'payload'
    return subprocess.Popen([str(payload / executable(manifest['target'])), *args], cwd=payload)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    package = commands.add_parser('export', help='Create a release from a prepared local client tree')
    package.add_argument('source', type=Path)
    package.add_argument('package', type=Path)
    package.add_argument('--version', required=True)
    update = commands.add_parser('install', help='Stage and activate a verified local primary build')
    update.add_argument('package', type=Path)
    update.add_argument('root', type=Path)
    commands.add_parser('status').add_argument('root', type=Path)
    commands.add_parser('rollback').add_argument('root', type=Path)
    play = commands.add_parser('play', help='Launch the active verified game client')
    play.add_argument('root', type=Path)
    play.add_argument('game_args', nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    try:
        if args.command == 'export':
            print(export(args.source, args.package, args.version))
        elif args.command == 'install':
            percent = -10

            def report(done, total):
                nonlocal percent
                value = 100 * done // total
                if value >= percent + 10 or value == 100:
                    percent = value
                    print(f'Verifying {done:,} / {total:,} bytes ({value}%)', flush=True)

            print(f'Active client {install(args.package, args.root, report)}')
        elif args.command == 'status':
            print(json.dumps(active(args.root.resolve()), indent=2))
        elif args.command == 'rollback':
            print(f'Active client {rollback(args.root)}')
        else:
            game_args = args.game_args
            if game_args[:1] == ['--']:
                game_args = game_args[1:]
            return launch(args.root, game_args).wait()
    except (InstallError, OSError) as error:
        parser.exit(1, f'{error}\n')
    return 0


if __name__ == '__main__':
    sys.exit(main())
