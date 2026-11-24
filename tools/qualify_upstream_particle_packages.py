#!/usr/bin/env python3
"""Export each bundled upstream example and check actual client admission."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess


def run(output, editor, client):
    output.mkdir(parents=True, exist_ok=False)
    source = Path('third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples')
    commands = output/'commands.jsonl'
    base = Path('content/zones/greybox.commands.jsonl').read_text().rstrip()
    place = {'command':'apply', 'expected_revision':1, 'edits':[{'op':'put_particle',
        'particle':{'id':10,'name':'Example','effect':'vfx.example',
                    'position':[2,1.3,-2], 'yaw_degrees':0, 'scale':1}}]}
    commands.write_text(base+'\n'+json.dumps(place)+'\n')
    results = []
    for example in sorted(source.glob('*.ron')):
        folder = output/example.stem
        library = folder/'data/library'
        library.mkdir(parents=True)
        shutil.copyfile(example, library/'vfx.example.ron')
        world = folder/'world'
        env = dict(os.environ, THEIA_PARTICLE_EDITOR_DATA_DIR=str(library.parent.resolve()))
        export = subprocess.run([str(editor),'--commands',str(commands),'--export',str(world),
                                 '--content-id','theia.test.upstream-particles','--revision','1'],
                                text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                env=env, timeout=60)
        (folder/'export.log').write_text(export.stdout)
        entry = {'example':example.name, 'exported':export.returncode == 0}
        if export.returncode == 0:
            admission = subprocess.run([str(client),'--terrain-world',str(world)], text=True,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                       env=env, timeout=60)
            (folder/'admission.log').write_text(admission.stdout)
            entry['admitted'] = admission.returncode != 0 and 'World walking requires' in admission.stdout
            entry['admission_message'] = admission.stdout.strip().splitlines()[-1]
            entry['export_counts'] = json.loads((world/'export.json').read_text()).get('particles')
        else:
            entry['admitted'] = False
            entry['export_message'] = export.stdout.strip().splitlines()[-1]
        results.append(entry)
    assert len(results) == 7
    report = {'scope':'Export and pre-launch admission only; no rendering or visual equivalence claim',
              'passed':all(row['exported'] and row['admitted'] for row in results), 'examples':results}
    (output/'qualification.json').write_text(json.dumps(report,indent=2)+'\n')
    if not report['passed']:
        raise RuntimeError('At least one upstream example did not export and pass client admission')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--editor', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--client', type=Path, default=Path('target/debug/theia-client'))
    args = parser.parse_args()
    print(json.dumps(run(args.output.resolve(), args.editor.resolve(), args.client.resolve()),indent=2))
