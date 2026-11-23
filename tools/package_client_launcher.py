"""Build the separate Linux launcher package from the committed application and skin."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from client_install import version

ROOT = Path(__file__).resolve().parent.parent
APP = ROOT / 'apps/launcher'
sys.path.insert(0, str(APP))
from artwork import ASSET_SOURCES


def package(output, release):
    release = version(release)
    output = output.resolve()
    if output.exists() or output.suffix != '.deb':
        raise ValueError('Choose a new .deb output file')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.launcher-', dir=output.parent) as temporary:
        staging = Path(temporary) / 'package'
        application = staging / 'usr/lib/theia-launcher'
        assets = application / 'assets'
        assets.mkdir(parents=True)
        for filename in ('launcher.py', 'state.py', 'launcher.css', 'chrome.py', 'artwork.py'):
            shutil.copyfile(APP / filename, application / filename)
        shutil.copyfile(ROOT / 'tools/client_install.py', application / 'client_install.py')
        icon = ROOT / ASSET_SOURCES['icon']
        for name, source in ASSET_SOURCES.items():
            shutil.copyfile(ROOT / source, assets / f'{name}.png')
        for source, destination in [
            (APP / 'theia-launcher', 'usr/bin/theia-launcher'),
            (APP / 'theia-launcher.desktop', 'usr/share/applications/theia-launcher.desktop'),
            (icon, 'usr/share/pixmaps/theia-launcher.png'),
        ]:
            target = staging / destination
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        (staging / 'usr/bin/theia-launcher').chmod(0o755)
        control = staging / 'DEBIAN'
        control.mkdir()
        (control / 'control').write_text(
            'Package: theia-launcher\n'
            f'Version: {release}\n'
            'Section: games\n'
            'Priority: optional\n'
            'Architecture: amd64\n'
            'Maintainer: Theia Online\n'
            'Depends: python3 (>= 3.10), python3-gi, python3-gi-cairo, gir1.2-gtk-3.0\n'
            'Description: Theia Online client installer and launcher\n'
            ' Install and update the game client separately from this application.\n',
            encoding='utf-8',
        )
        archive = Path(temporary) / output.name
        subprocess.run(['dpkg-deb', '--root-owner-group', '--build', str(staging), str(archive)],
                       check=True)
        archive.rename(output)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--version', default='0.1.1')
    args = parser.parse_args()
    print(package(args.output, args.version))


if __name__ == '__main__':
    main()
