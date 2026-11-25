"""Discover and stage verified Debian releases of the launcher application."""
from dataclasses import dataclass
from pathlib import Path
import shutil
import subprocess
import tempfile

from client_install import InstallError, copy_file, version_key
from hosted_release import DownloadInterrupted, HostedLibrary

PACKAGE = 'theia-launcher'
SYSTEM_LAUNCHER = Path('/usr/bin') / PACKAGE
VERSION = Path(__file__).with_name('VERSION').read_text(encoding='utf-8').strip()


def launcher_package_name(release):
    return f'{PACKAGE}_{release}_amd64.deb'


@dataclass(frozen=True)
class HostedLauncherUpdate:
    library: HostedLibrary
    metadata: dict

    def download(self, destination):
        try:
            self.library.download(self.metadata, destination, lambda _: None,
                                  name=launcher_package_name(self.metadata['version']))
        except DownloadInterrupted as error:
            raise InstallError('Launcher download interrupted. Choose Update Launcher to retry.') from error
        size = self.metadata['bytes']
        digest = self.metadata['sha256']
        if copy_file(destination, None, size, lambda _: None) != (size, digest):
            raise InstallError('Launcher package failed verification. Check for updates again.')


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


def newest_hosted_launcher_release(library, current):
    metadata = library.launcher_metadata()
    if metadata is not None and version_key(metadata['version']) > version_key(current):
        return HostedLauncherUpdate(library, metadata), metadata['version']
    return None


def install_launcher_update(offer):
    source, release = offer
    with tempfile.TemporaryDirectory(prefix='theia-launcher-update-') as directory:
        if isinstance(source, HostedLauncherUpdate):
            staged = Path(directory) / launcher_package_name(release)
            source.download(staged)
        else:
            staged = Path(directory) / source.name
            shutil.copyfile(source, staged)
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
