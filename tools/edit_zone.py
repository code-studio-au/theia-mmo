#!/usr/bin/env python3
"""Build and open the 3440x1440 zone editor, reopening the saved draft by default."""
import argparse
import os
from pathlib import Path
import subprocess
import sys

from launch_environment import client_environment

ROOT = Path(__file__).resolve().parents[1]
AUTHORING_FLAGS = {'--zone', '--commands', '--output', '--save-path', '--export'}


def editor_arguments(arguments, draft):
    """Explicit editor authoring arguments win; opening a default draft never writes it."""
    if any(arg.split('=', 1)[0] in AUTHORING_FLAGS for arg in arguments):
        return list(arguments)
    option = '--zone' if draft.exists() else '--save-path'
    return [*arguments, option, str(draft)]


def example_arguments(arguments, draft, root=ROOT):
    """Open a saved example draft, or seed a new destination without writing it."""
    if any(arg.split('=', 1)[0] in AUTHORING_FLAGS for arg in arguments):
        raise ValueError('--example cannot be combined with explicit authoring flags; use --draft to choose its destination')
    if draft.exists():
        return [*arguments, '--zone', str(draft)]
    source = root / 'content/zones/wayfarer-hollow.base.json'
    commands = root / 'content/zones/wayfarer-hollow.commands.jsonl'
    return [*arguments, '--zone', str(source), '--commands', str(commands), '--save-path', str(draft)]


def build_environment(source, root=ROOT):
    result = dict(source)
    sdk = root / 'build/native-sdk/usr/lib/x86_64-linux-gnu'
    if sys.platform.startswith('linux') and (sdk / 'pkgconfig').is_dir():
        for name, path in [('PKG_CONFIG_PATH', sdk / 'pkgconfig'), ('LIBRARY_PATH', sdk)]:
            result[name] = str(path) + (os.pathsep + source[name] if source.get(name) else '')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog='Other arguments pass directly to the editor; explicit authoring flags bypass automatic draft selection.')
    parser.add_argument('--draft', type=Path, help='draft to reopen or save later (default: build/zone-draft.json, or build/wayfarer-hollow.zone.json with --example)')
    parser.add_argument('--example', action='store_true', help='start or reopen an editable Wayfarer Hollow example draft')
    parser.add_argument('--no-build', action='store_true', help='use the existing debug editor binary')
    parser.add_argument('--build-only', action='store_true', help='build the editor and exit')
    options, forwarded = parser.parse_known_args()
    if options.no_build and options.build_only:
        parser.error('--no-build and --build-only cannot be combined')
    draft = (options.draft or ROOT / ('build/wayfarer-hollow.zone.json' if options.example else 'build/zone-draft.json')).resolve()
    if not options.no_build:
        completed = subprocess.run(['cargo', 'build', '--locked', '-p', 'theia-zone-editor', '-j1'], cwd=ROOT, env=build_environment(os.environ))
        if completed.returncode:
            return completed.returncode
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'target'))
    if not target.is_absolute():
        target = ROOT / target
    binary = target / 'debug' / ('theia-zone-editor.exe' if os.name == 'nt' else 'theia-zone-editor')
    if not binary.is_file():
        raise ValueError(f'Missing editor binary: {binary}')
    if options.build_only:
        print(f'Editor built: {binary}')
        return 0
    arguments = example_arguments(forwarded, draft) if options.example else editor_arguments(forwarded, draft)
    if not any(arg.split('=', 1)[0] in AUTHORING_FLAGS for arg in forwarded):
        draft.parent.mkdir(parents=True, exist_ok=True)
    # Preserve caller-relative explicit paths. On Linux, exec preserves the owned PID
    # for the native input qualifier and leaves no launcher/child pair to clean up.
    os.execve(str(binary), [str(binary), *arguments], client_environment(os.environ))


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f'Zone editor: {error}', file=sys.stderr)
        raise SystemExit(1)
