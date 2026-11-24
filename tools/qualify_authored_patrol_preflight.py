"""Real client configuration checks against a bound authored navigation package."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from launch_environment import client_environment

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--client', type=Path, default=Path('target/debug/theia-client'))
parser.add_argument('--world', type=Path, required=True)
parser.add_argument('--navigation', type=Path, required=True)
parser.add_argument('--profile', type=Path, required=True)
parser.add_argument('--route', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
profile = json.loads(args.profile.read_text())
profile['route'] = args.route
matching = out / 'matching-creature.json'
matching.write_text(json.dumps(profile, indent=2) + '\n')
missing = out / 'missing-route.json'
profile['route'] = 'theia.test.absent-route'
missing.write_text(json.dumps(profile, indent=2) + '\n')
manifest = json.loads((args.navigation / 'navigation.manifest.json').read_text())
wrong = out / 'foreign-navigation'
wrong.mkdir()
foreign = dict(manifest, source=[17] * 32)
(wrong / 'navigation.manifest.json').write_text(json.dumps(foreign))
broken = out / 'corrupt-navigation'
shutil.copytree(args.navigation, broken)
# Modify exactly one owned copied tile; retain its expected manifest hash.
tile_id = manifest['tiles'][0]['id']
tile = broken / f'tile_{tile_id[0]}_{tile_id[1]}.nav.json'
tile.write_bytes(tile.read_bytes() + b'\n')
env = client_environment(os.environ)
results = []
for name, definition, navigation, expected in [
        ('matching', matching, args.navigation, None),
        ('missing-route', missing, args.navigation, 'Patrol route is absent'),
        ('foreign-world', matching, wrong, 'Navigation manifest'),
        ('corrupt-tile', matching, broken, 'hash mismatch')]:
    command = [str(args.client.resolve()), '--authored-world', str(args.world.resolve()),
        '--prediction', '--classic', '--combat-catalog', 'content/gameplay/development-classic-abilities.json',
        '--status-catalog', 'content/gameplay/development-statuses.json',
        '--item-catalog', 'content/gameplay/development-equipment-items.json',
        '--character-stats', 'content/gameplay/development-character-stats.json',
        '--creature-retaliation', 'content/gameplay/development-creature-retaliation.json',
        '--player-respawn', 'content/gameplay/development-player-respawn.json',
        '--creature-profile', str(definition), '--navigation', str(navigation.resolve()), '--configuration-report']
    result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=30)
    (out / (name + '.stdout')).write_text(result.stdout)
    (out / (name + '.stderr')).write_text(result.stderr)
    if expected is None:
        assert result.returncode == 0, result.stderr
        report = json.loads(result.stdout[result.stdout.index('{'):])
        assert not report['renderer_started'] and not report['network_started'] and not report['saved_state_admitted']
    else:
        assert result.returncode != 0 and expected in result.stderr, (name, result.stderr)
        assert '"renderer_started"' not in result.stdout
    results.append(dict(case=name, passed=True, accepted=expected is None))
(out / 'qualification.json').write_text(json.dumps(dict(passed=True, checks=results,
    scope='Actual client preflight only; no authenticated gameplay or patrol simulation'), indent=2))
