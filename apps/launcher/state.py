"""Local release discovery and launcher settings, independent of the desktop UI."""
import json
import os
from pathlib import Path
import tempfile

import client_install


def data_directory():
    if os.name == 'nt':
        base = Path(os.environ.get('LOCALAPPDATA') or Path.home() / 'AppData' / 'Local')
    else:
        base = Path(os.environ.get('XDG_DATA_HOME') or Path.home() / '.local' / 'share')
    return base / 'theia-online'


def newest_release(library, current):
    """Inspect a package or one level of release directories in a local library."""
    if library is None:
        return None
    library = library.resolve(strict=True)
    if not library.is_dir():
        raise client_install.InstallError('Choose a release folder')
    single = (library / client_install.MANIFEST).is_file()
    candidates = [library] if single else (
        child for child in library.iterdir() if child.is_dir() and not child.is_symlink()
    )
    installed = client_install.version_key(current) if current else None
    newest = None
    for directory in candidates:
        try:
            manifest, total = client_install.metadata(directory)
        except (OSError, client_install.InstallError):
            if single:
                raise
            continue
        key = client_install.version_key(manifest['version'])
        if installed is not None and key <= installed:
            continue
        if newest is None or key > newest[0]:
            newest = key, directory, manifest['version'], total, False
        elif key == newest[0]:
            newest = (*newest[:4], True)
    if newest and newest[4]:
        raise client_install.InstallError('More than one package has the newest version')
    return newest[1:4] if newest else None


def read_settings(path):
    if not path.exists():
        return {}
    config = client_install.read_json(path, 16 * 1024)
    keys = {'install_root', 'release_library'}
    if not isinstance(config, dict) or set(config) - keys or any(
        not isinstance(value, str) or not Path(value).is_absolute()
        for value in config.values()
    ):
        raise client_install.InstallError('Invalid launcher preferences')
    return config


def save_settings(path, install_root, release_library):
    conf = {'install_root': str(install_root.resolve())}
    if release_library is not None:
        conf['release_library'] = str(release_library.resolve())
    path.parent.mkdir(parents=True, mode=0o700, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.settings-', dir=path.parent) as directory:
        staged = Path(directory) / 'settings.json'
        client_install.write_json(staged, conf)
        os.replace(staged, path)
