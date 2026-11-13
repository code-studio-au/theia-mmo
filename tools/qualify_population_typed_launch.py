"""Owned OIDC/SQL and native typed-population admission, without OS input automation."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
import urllib.parse
from edit_zone import build_environment
from gpu_validation import check_gpu_diagnostics
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import wait, capture_ready


def run(args):
    out = args.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    package, binaries = args.population.resolve(), args.binaries.resolve()
    profiles = [
        ('--combat-catalog', '--abilities', 'development-classic-abilities.json'),
        ('--character-stats', '--character-stats', 'development-character-stats.json'),
        ('--status-catalog', '--statuses', 'development-statuses.json'),
        ('--item-catalog', '--items', 'development-equipment-items.json'),
        ('--creature-retaliation', '--creature-retaliation', 'development-creature-retaliation.json'),
        ('--player-respawn', '--player-respawn', 'development-player-respawn.json'),
        ('--melee-weapons', '--melee-weapons', 'development-melee-weapons.json'),
        ('--physical-definitions', '--physical-definitions', 'development-physical-damage-definitions.json'),
        ('--physical-stats', '--physical-stats', 'development-physical-damage-stats.json'),
        ('--white-melee', '--white-melee', 'development-white-melee.json'),
        ('--ability-starting', '--ability-starting', 'development-ability-starting.json'),
        ('--ability-damage', '--ability-damage', 'development-ability-damage.json'),
        ('--combat-membership', '--combat-membership', 'development-combat-membership.json'),
        ('--creature-credit', '--creature-credit', 'development-creature-credit-solo.json'),
    ]
    common = ['--authored-world', str(package/'world'), '--population-config',
              str(package/'population.json'), '--classic', '--public-avatar-life']
    raw, launch_profiles = [], []
    for native_flag, launcher_flag, filename in profiles:
        path = str((Path('content/gameplay')/filename).resolve())
        raw.extend([native_flag, path])
        launch_profiles.extend([launcher_flag, path])
    base_env = build_environment(os.environ)
    headless = dict(base_env)
    for key in ('DISPLAY', 'WAYLAND_DISPLAY'):
        headless.pop(key, None)
    checked = subprocess.run([str(binaries/'theia-client'), *common, '--prediction',
                              *raw, '--configuration-report'], env=headless,
                             capture_output=True, text=True, timeout=30, check=True)
    configuration = json.loads(checked.stdout[checked.stdout.index('{'):])
    assert configuration['population']['count'] == 5
    assert all(configuration['typed_combat'].values())
    assert not configuration['renderer_started'] and not configuration['network_started']
    (out/'configuration.json').write_text(json.dumps(configuration, indent=2)+'\n')
    results = []
    with Identity(out/'provider', Path('build/identity-sdk'), token_seconds=600, interactive=True) as identity, \
            Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
        original_bag = None
        for resumed in (False, True):
            if resumed:
                cluster.restart()
            folder = out/('resumed' if resumed else 'fresh')
            folder.mkdir()
            env = {k: v for k, v in base_env.items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='600', THEIA_OIDC_ISSUER=identity.issuer,
                THEIA_OIDC_CLIENT_ID=identity.client_id,
                THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_WORLD_INPUT_OS_QUALIFY=str(folder))
            if args.combat and not resumed:
                env['THEIA_POPULATION_TYPED_QUALIFY'] = str(folder)
            previous_tickets = set(Path('build/development-session').glob('renet-*'))
            with (folder/'launcher.log').open('w') as log:
                launcher = subprocess.Popen([sys.executable, 'tools/play.py', *common,
                    '--account-login', '--equipment', '--status-effects', *launch_profiles,
                    '--binaries', str(binaries), '--players', str(args.players.resolve()),
                    '--creature-asset', str(package/'creature.glb'), '--window-size', '1280x720'],
                    env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True)
                try:
                    launcher.stdin.write('1\n' if resumed else 'PoolArden\n')
                    launcher.stdin.flush()
                    def state():
                        assert launcher.poll() is None, 'Owned typed population launcher exited'
                        return json.loads((folder/'input-os.json').read_text())
                    def ready():
                        value = state()
                        combat = value.get('combat') or {}
                        return value if (value['connected'] and len(value.get('combat_targets') or []) == 5
                            and value.get('inventory_bag') and combat.get('membership_events', 0) > 0
                            and combat.get('player_health') == 100 and value.get('public_avatar_life')
                            and (value.get('creature_visuals') or {}).get('players') == 5
                            and (value.get('minimap') or {}).get('known_pixels', 0) > 0) else None
                    observed = wait(ready, 90)
                    # Asset entities can be admitted before their GPU pipelines
                    # finish. Allow the same bounded warmup on fresh and resumed runs.
                    admitted_at = time.monotonic()
                    observed = wait(lambda: ready() if time.monotonic()-admitted_at >= 10 else None, 20)
                    assert observed['combat']['in_combat'] is False
                    combat_proof = None
                    if args.combat and not resumed:
                        (folder/'combat.start').write_text('start\n')
                        def completed():
                            state()
                            value = json.loads((folder/'typed-combat.json').read_text())
                            assert value['passed'], value
                            return value
                        combat_proof = wait(completed, 40)
                        observed = ready()
                        assert observed and observed['main_hand_item'] == 'theia.dev.training-wand'
                        assert combat_proof['player_health'] == 100 and combat_proof['remaining'] < 100
                        assert combat_proof['inventory_revision'] == 1

                    assert len(observed['public_avatar_life']['actors']) == 1
                    if resumed:
                        assert observed['inventory_bag'] == original_bag
                    else:
                        original_bag = observed['inventory_bag']
                    (folder/'capture.request').write_text('capture\n')
                    wait(lambda: capture_ready(folder/'input-os.png'), 15)
                    results.append(dict(resumed=resumed, health=100, creatures=5,
                        inventory_revision=observed['inventory_bag']['revision'],
                        membership_events=observed['combat']['membership_events'], combat=combat_proof))
                finally:
                    if launcher.poll() is None:
                        launcher.send_signal(signal.SIGINT)
                        try:
                            launcher.wait(timeout=30)
                        except subprocess.TimeoutExpired:
                            launcher.kill()
                            launcher.wait(timeout=10)
                    launcher.stdin.close()
                    source = Path('build/development-session/server.log')
                    if source.exists():
                        shutil.copy2(source, folder/'server.log')
                assert launcher.returncode == 0
            assert not (set(Path('build/development-session').glob('renet-*')) - previous_tickets)
            server = (folder/'server.log').read_text()
            saved = next(json.loads(line.removeprefix('account_persistence_summary '))
                         for line in server.splitlines() if line.startswith('account_persistence_summary '))
            assert not saved['failed'] and saved['worker']['inventory_ensures'] == 1
            if args.combat and not resumed:
                assert saved['worker']['inventory_committed'] == 1
                transport = next(json.loads(line.removeprefix('transport_summary '))
                                 for line in server.splitlines() if line.startswith('transport_summary '))
                assert transport['ability_damage']['arcane']['health_lost'] > 0
                assert transport['ability_damage']['arcane']['attempts'] <= 3
                assert transport['rejected_inputs'] == 0
            assert saved['worker']['incompatible_gameplay_profiles'] == 0
            account = next(json.loads(line.removeprefix('account_summary '))
                           for line in server.splitlines() if line.startswith('account_summary '))
            assert account['verified_logins'] == 1
            for file in folder.glob('*.log'):
                text = file.read_text()
                assert identity.password not in text and cluster.password not in text
                assert check_gpu_diagnostics(text) == 0
    assert not any((out/'cluster'/name).exists() for name in ('password', 'pgpass'))
    result = dict(passed=True, runs=results, database_restart=True, private_services_closed=True,
        mouse_or_keyboard_automation=False, scope=('Normal typed population cast, committed equipment, combat membership and inventory restart; no OS input, NPC retaliation or reward-grant claim' if args.combat else 'Normal typed population profile admission, owned OIDC/SQL inventory and native UI; no combat execution, NPC retaliation or reward-grant claim'))
    (out/'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--combat', action='store_true', help='Equip and cast through normal network commands without OS input')
    parser.add_argument('--population', type=Path, required=True)
    parser.add_argument('--players', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--output', type=Path, required=True)
    print(json.dumps(run(parser.parse_args()), indent=2))
