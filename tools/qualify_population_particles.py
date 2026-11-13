"""Headless real population export with particles and immutable dependency checks."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from launch_environment import client_environment


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    source = args.zone.read_bytes()
    zone = json.loads(source)
    assert zone['particles'], 'Provide at least one saved particle placement'
    library = out/'library'
    shutil.copytree(args.library.resolve(), library)
    env = client_environment(dict(os.environ, THEIA_PARTICLE_EDITOR_DATA_DIR=str(out/'missing')))
    command = [str(args.binaries.resolve()/'theia-zone-editor'), '--headless', '--zone', str(args.zone.resolve()),
        '--population', '--network-test', '--population-creature', str(args.creature.resolve()),
        '--content-id', 'theia.test.population-particles', '--revision', '1']
    if args.appearance:
        command += ['--creature-asset', str(args.appearance.resolve())]
    def export(name):
        result = subprocess.run(command+['--export',str(out/name)],env=env,capture_output=True,text=True,timeout=120)
        (out/(name+'.log')).write_text(result.stdout+result.stderr)
        return result
    result = export('refused')
    assert result.returncode and 'local effect is missing' in result.stderr and not (out/'refused').exists(),result.stderr
    env['THEIA_PARTICLE_EDITOR_DATA_DIR']=str(library)
    result = export('package')
    assert result.returncode==0,result.stderr
    report = json.loads(result.stdout.splitlines()[-1])['population_export']
    assert report['world']['particles']['placements']==len(zone['particles'])
    package = out/'package'
    catalog = json.loads((package/'world/particles/catalog.json').read_text())
    assert len(catalog['placements'])==len(zone['particles'])
    copied = {path.relative_to(package):path.read_bytes() for path in (package/'world/particles').rglob('*') if path.is_file()}
    # Source changes cannot alter the frozen output; an output change must fail admission.
    for path in (library/'library').glob('*.ron'):
        path.write_text('Changed test-owned source')
    assert all((package/path).read_bytes()==data for path,data in copied.items())
    check = [str(args.binaries.resolve()/'theia-client'), '--authored-world', str(package/'world'),
        '--prediction','--classic','--population-config',str(package/'population.json'),
        '--combat-catalog',str(package/'combat.json'),'--configuration-report']
    result = subprocess.run(check,env=env,capture_output=True,text=True,timeout=30)
    (out/'preflight.log').write_text(result.stdout+result.stderr)
    assert result.returncode==0,result.stderr
    effect = next((package/'world/particles/effects').glob('*.ron'))
    original = effect.read_bytes()
    try:
        effect.write_text('Changed published effect')
        result = subprocess.run(check,env=env,capture_output=True,text=True,timeout=30)
        (out/'tampered.log').write_text(result.stdout+result.stderr)
        assert result.returncode and 'descriptor' in result.stderr,result.stderr
    finally:
        effect.write_bytes(original)
    assert args.zone.read_bytes()==source
    (out/'qualification.json').write_text(json.dumps(dict(passed=True,placements=len(catalog['placements']),
        count=report['count'],missing_refused=True,tampered_refused=True,source_unchanged=True,
        scope='Headless package/preflight integrity only; native rendering checked separately'),indent=2)+'\n')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('zone','library','creature','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--appearance',type=Path)
    parser.add_argument('--binaries',type=Path,default=Path('target/debug'))
    run(parser.parse_args())
