"""Discover trusted local Debian releases of the launcher application."""
from pathlib import Path
import shutil
import subprocess
import tempfile

from client_install import InstallError, version_key

PACKAGE = 'theia-launcher'
SYSTEM_LAUNCHER = Path('/usr/bin') / PACKAGE
VERSION = Path(__file__).with_name('VERSION').read_text(encoding='utf-8').strip()


def package_matches(path, release):
    try:
        metadata = subprocess.run(
            ['/usr/bin/dpkg-deb', '--show', '--showformat=${Package}\t${Version}\t${Architecture}', str(path)],
            capture_output=True, check=False, timeout=10,
        )
    except subprocess.TimeoutExpired:
        return False
    return metadata.returncode == 0 and metadata.stdout == f'{PACKAGE}\t{release}\tamd64'.encode('ascii')


def newest_launcher_release(library, current):
    if library is None:
        return None
    directory = library / 'launcher'
    if not directory.is_dir():
        return None
    candidates = []
    installed = version_key(current)
    for path in directory.glob(f'{PACKAGE}_*_amd64.deb'):
        release = path.name.removeprefix(f'{PACKAGE}_').removesuffix('_amd64.deb')
        try:
            key = version_key(release)
        except InstallError:
            continue
        if key > installed and not path.is_symlink() and path.is_file():
            candidates.append((key, release, path))
    for _, release, path in sorted(candidates, reverse=True):
        if package_matches(path, release):
            return path, release
    return None


def install_launcher_update(offer):
    path, release = offer
    with tempfile.TemporaryDirectory(prefix='theia-launcher-update-') as directory:
        staged = Path(directory) / path.name
        shutil.copyfile(path, staged)
        if not package_matches(staged, release):
            raise InstallError('The launcher package changed. Check for updates again.')
        result = subprocess.run(
            ['/usr/bin/pkexec', '--disable-internal-agent', '/usr/bin/apt-get',
             '--assume-yes', '--no-remove', 'install', str(staged)],
            capture_output=True, text=True, errors='replace', check=False,
        )
        if result.returncode == 126:
            raise InstallError('Launcher update cancelled.')
        if result.returncode:
            lines = result.stderr.strip().splitlines()
            detail = f' {lines[-1]}' if lines else ''
            raise InstallError(f'Launcher update failed.{detail}')
    try:
        installed = subprocess.run(
            ['/usr/bin/dpkg-query', '--show', '--showformat=${Status}\t${Version}', PACKAGE],
            capture_output=True, check=False, timeout=10,
        )
    except subprocess.TimeoutExpired as error:
        raise InstallError('Timed out checking the installed launcher version.') from error
    if (installed.returncode != 0 or installed.stdout != f'install ok installed\t{release}'.encode('ascii')
            or not SYSTEM_LAUNCHER.is_file()):
        raise InstallError('The system did not confirm the offered launcher version.')
    return release
