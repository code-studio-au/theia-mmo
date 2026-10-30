#!/usr/bin/env python3
"""Launch the continental graybox or an explicit legacy map, with child cleanup."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import time
import tempfile


def play(world, players, clients, binaries, tick_hz=30, transport="renet", chunk_world=None, camera_collision=None, graybox=False, dev_speed=1, dev_spawn="8000,8000", dev_tour=False, prediction=False, saved_terrain=None, targeting=False, combat_catalog=None, feedback=None, persistence=False, account_login=False, character_stats=None, item_catalog=None, status_catalog=None, creature_profile=None, navigation=None, reward_catalog=None, progression_catalog=None, window_size=None, fullscreen=False, quest_catalog=None, dialogue_catalog=None, guide_position=None, vendor_catalog=None):
    window_size = window_size or os.environ.get('THEIA_WINDOW_SIZE', '1920x1200')
    match = re.fullmatch(r'(\d+)x(\d+)', window_size)
    if not match or not (640 <= int(match[1]) <= 7680 and 480 <= int(match[2]) <= 4320):
        raise ValueError('Expected --window-size WIDTHxHEIGHT within 640x480 to 7680x4320')
    if vendor_catalog and (not dialogue_catalog or not account_login or not item_catalog):
        raise ValueError('Vendor requires guide dialogue, account login and items')
    if bool(dialogue_catalog) != bool(guide_position):
        raise ValueError('Dialogue requires a catalog and explicit guide position')
    if dialogue_catalog:
        if not graybox or not prediction or transport != 'renet':
            raise ValueError('Dialogue requires predicted graybox and secure Renet')
        guide_values=[float(value) for value in guide_position.split(',')]
        if len(guide_values)!=2 or not all(math.isfinite(value) and 0 <= value < 16000 for value in guide_values):
            raise ValueError('Guide position must be finite graybox x,z metres')
    if bool(creature_profile) != bool(navigation):
        raise ValueError('Creature profile and navigation package must be selected together')
    if creature_profile and (not combat_catalog or not prediction or saved_terrain is None):
        raise ValueError('Creature patrol requires predicted saved-terrain combat')
    if status_catalog and (not character_stats or not combat_catalog):
        raise ValueError('Statuses require combat and character stats')
    if quest_catalog and not progression_catalog:
        raise ValueError("Quest objectives require progression")
    if progression_catalog and (not account_login or not creature_profile):
        raise ValueError("Progression requires account login and creatures")
    if reward_catalog and (not account_login or not item_catalog or not creature_profile):
        raise ValueError('Rewards require account login, items and creature combat')
    if item_catalog and not account_login:
        raise ValueError('Persistent inventory requires --account-login')
    if character_stats and not combat_catalog:
        raise ValueError('Character stats require combat')
    if account_login:
        if persistence or saved_terrain is None or not prediction or transport != 'renet' or clients != 1 or dev_tour or dev_speed != 1:
            raise ValueError('Account login requires one client, normal predicted saved terrain and Renet; excludes development persistence/tour')
        if any(not os.environ.get(key) for key in ['THEIA_DATABASE_URL','THEIA_OIDC_ISSUER','THEIA_OIDC_CLIENT_ID']):
            raise ValueError('Account login requires THEIA_DATABASE_URL, THEIA_OIDC_ISSUER and THEIA_OIDC_CLIENT_ID; see docs/AUTHENTICATED_ADMISSION.md')
    if persistence and (saved_terrain is None or not prediction or transport != 'renet' or dev_tour or dev_speed != 1):
        raise ValueError('Persistence requires saved terrain, normal-speed prediction and Renet without a tour')
    if persistence and not os.environ.get('THEIA_DATABASE_URL'):
        raise ValueError('Persistence requires a local PostgreSQL THEIA_DATABASE_URL; see docs/PERSISTENCE_FOUNDATION.md')
    if saved_terrain is not None:
        if world is not None or chunk_world is not None:
            raise ValueError('Choose saved terrain or an imported world')
        if not (saved_terrain / 'terrain.manifest.json').is_file():
            raise ValueError(f'Missing saved terrain at {saved_terrain}; bake once with cargo run --release --locked -p theia-network --example bake_terrain -- bake "{saved_terrain}"; see docs/SAVED_TERRAIN.md')
        graybox = True
    if creature_profile and not (navigation / 'navigation.manifest.json').is_file():
        raise ValueError(f'Missing baked navigation at {navigation}; run cargo run --release --locked -p theia-nav-bake -- bake "{saved_terrain}" "{navigation}" first')
    if feedback and not combat_catalog:
        raise ValueError("Spell feedback requires combat")
    if feedback:
        required_feedback = feedback
        if not required_feedback.is_file():
            raise ValueError(f"Missing spell feedback {feedback}")
    if combat_catalog is not None:
        targeting = True
    if targeting and (not graybox or transport != 'renet'):
        raise ValueError("Targeting requires the graybox world and authenticated Renet")
    if prediction and not graybox:
        raise ValueError("Prediction requires the graybox world")
    if not graybox and (dev_speed != 1 or dev_spawn != "8000,8000" or dev_tour):
        raise ValueError('Developer travel options require the graybox world')
    if (chunk_world or graybox) and transport != 'renet':
        raise ValueError('Chunk worlds require authenticated Renet readiness')
    suffix = '.exe' if os.name == 'nt' else ''
    server_binary = binaries / ('theia-server' + suffix)
    client_binary = binaries / ('theia-client' + suffix)
    required = ([saved_terrain / 'terrain.manifest.json'] if saved_terrain else []) if graybox else ([chunk_world / 'world.chunks.json', chunk_world / 'world.session.json'] if chunk_world else
                [world / 'world.manifest.json', world / 'terrain.json', world / 'terrain.glb'])
    scene = [*(['--saved-terrain', str(saved_terrain)] if saved_terrain else ['--graybox-world']), *(['--prediction'] if prediction else []), *(['--targeting'] if targeting else [])] if graybox else (['--chunk-world', str(chunk_world)] if chunk_world else ['--terrain-world', str(world)])
    if vendor_catalog:
        required.append(vendor_catalog)
        scene.extend(['--vendor-catalog',str(vendor_catalog)])
    if dialogue_catalog:
        required.append(dialogue_catalog)
        scene.extend(['--dialogue-catalog',str(dialogue_catalog),'--guide-position',guide_position])
    if combat_catalog is not None:
        required.append(combat_catalog)
        scene.extend(['--combat-catalog', str(combat_catalog)])
    if character_stats is not None:
        required.append(character_stats)
        scene.extend(['--character-stats', str(character_stats)])
    if status_catalog is not None:
        required.append(status_catalog)
        scene.extend(['--status-catalog', str(status_catalog)])
    if creature_profile is not None:
        required.extend([creature_profile, navigation / 'navigation.manifest.json'])
        scene.extend(['--creature-profile', str(creature_profile), '--navigation', str(navigation)])
    if item_catalog is not None:
        required.append(item_catalog)
        scene.extend(['--item-catalog', str(item_catalog)])
    if quest_catalog is not None:
        required.append(quest_catalog)
        scene.extend(['--quest-catalog', str(quest_catalog)])
    if progression_catalog is not None:
        required.append(progression_catalog)
        scene.extend(['--progression-catalog', str(progression_catalog)])
    if reward_catalog is not None:
        required.append(reward_catalog)
        scene.extend(['--reward-catalog', str(reward_catalog)])
    server_options = ['--dev-speed', str(dev_speed), '--dev-spawn', dev_spawn, *(['--dev-tour'] if dev_tour else [])] if graybox else []
    for path in [*required, players / ('character.glb' if graybox else 'human.glb'), server_binary, client_binary]:
        if not path.is_file():
            raise ValueError(f'Missing {path}; build with cargo build --release --locked --workspace; see docs/DEVELOPMENT_WORLD.md for asset preparation or select --binaries explicitly')
    session = Path('build/development-session')
    session.mkdir(parents=True, exist_ok=True)
    certificate = session / 'server-cert.der'
    certificate.unlink(missing_ok=True)
    tickets = tempfile.TemporaryDirectory(prefix="renet-", dir=session) if transport == "renet" else None
    processes = []
    account = None
    try:
        with (session / 'server.log').open('w') as log:
            server = subprocess.Popen([str(server_binary), '127.0.0.1:0', str(certificate),
                                       *scene, *server_options, *(['--dev-persistence'] if persistence else []), '--tick-hz', str(tick_hz),
                                       *(['--account-login'] if account_login else (['--renet-tickets', tickets.name] if tickets else []))],
                                      stdout=log, stderr=subprocess.STDOUT)
            processes.append(server)
            address = None
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                if server.poll() is not None:
                    raise RuntimeError(f'Server failed; see {session / "server.log"}')
                if account_login:
                    ready = next((json.loads(line.removeprefix('account_game_ready ')) for line in (session/'server.log').read_text().splitlines() if line.startswith('account_game_ready ')),None)
                    if ready:
                        from account_login import AccountClient
                        account = AccountClient(ready['account_address'],alive=lambda:server.poll() is None)
                        print('Sign in using your system browser, then return here to choose your character.',flush=True)
                        account.login()
                        character = account.choose_character()
                        address = account.enter(character,Path(tickets.name)/'client-1.token')
                        break
                match = re.search(r'listening=(\S+) (?:public_certificate|local_tickets)=', (session / 'server.log').read_text())
                if match and (tickets or certificate.exists()):
                    address = match[1]
                    break
                time.sleep(.05)
            if address is None:
                raise RuntimeError('Server readiness timed out')
            for index in range(clients):
                processes.append(subprocess.Popen([str(client_binary), '--window-size', window_size, *(['--fullscreen'] if fullscreen else []),
                    *(['--camera-collision', camera_collision] if camera_collision else []), *(['--spell-feedback', str(feedback)] if feedback else []), *scene,
                    '--player-assets', str(players),
                    '--renet-connect' if tickets else '--connect', address,
                    str(Path(tickets.name) / f'client-{index+1}.token') if tickets else str(certificate)],
                    env={key:value for key,value in os.environ.items() if key not in {'THEIA_DATABASE_URL','THEIA_TEST_DATABASE_URL'}}))
            if combat_catalog:
                print('Combat: click the action slot or press 1 to cast; health, cooldown and respawn come from the server.', flush=True)
            if targeting:
                print('Training target: Tab selects/cycles; Backspace clears.', flush=True)
            print('Theia development map: WASD, Shift to run, Space to jump, RMB orbit, wheel zoom, Escape to quit.', flush=True)
            while all(p.poll() is None for p in processes):
                time.sleep(.2)
            if any(p.poll() not in (None, 0) for p in processes):
                raise RuntimeError('A game process failed; inspect its output and build/development-session/server.log')
    except KeyboardInterrupt:
        pass
    finally:
        if account:
            account.close()
        for process in reversed(processes):
            if process.poll() is None:
                # Let the POSIX server finish its simulation/transport summary.
                # Windows retains terminate until native console shutdown is qualified.
                if process is processes[0] and os.name != 'nt':
                    process.send_signal(signal.SIGINT)
                else:
                    process.terminate()
                try:
                    process.wait(timeout=15 if process is processes[0] and (persistence or account_login) else 5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()

        if tickets:
            tickets.cleanup()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    worlds = parser.add_mutually_exclusive_group()
    worlds.add_argument('--world', type=Path, help='Explicit legacy monolithic world')
    worlds.add_argument('--graybox-world', action='store_true', help='Retained generated terrain baseline')
    worlds.add_argument('--saved-terrain', type=Path, help='Versioned saved height terrain directory')
    worlds.add_argument('--chunk-world', type=Path, help='Opt-in prepared chunk index/session; requires Renet')
    parser.add_argument('--fullscreen', action='store_true', help='Borderless fullscreen on the current monitor; F11 toggles in game')
    parser.add_argument('--window-size', help='Client window WIDTHxHEIGHT (default 1920x1200, or THEIA_WINDOW_SIZE)')
    parser.add_argument('--players', type=Path, help='Player assets; defaults to POLYGON for graybox')
    parser.add_argument('--prediction', action=argparse.BooleanOptionalAction, default=None, help='Movement prediction (default for graybox; --no-prediction retains server-pose interpolation)')
    parser.add_argument('--account-login', action='store_true', help='System-browser OIDC and owned character entry; requires configured loopback provider/database')
    parser.add_argument('--persistence', action='store_true', help='Opt-in trusted development character checkpoints; requires local THEIA_DATABASE_URL')
    parser.add_argument('--spell-feedback', action='store_true', help='Opt-in original spell pulses and spatial audio; implies combat')
    parser.add_argument('--combat', action='store_true', help='Opt-in authoritative mobile ability, health, death and respawn; implies targeting')
    parser.add_argument('--inventory', action='store_true', help='Opt-in persistent development backpack; requires --account-login')
    parser.add_argument('--vendors',action='store_true',help='Opt-in guide barter; implies dialogue/equipment, requires account login')
    parser.add_argument('--vendor-catalog',type=Path,help='Barter offers; requires --vendors')
    parser.add_argument('--dialogue',action='store_true',help='Opt-in noncombat informational guide; E to talk, F1–F4 or buttons to choose, Esc to close')
    parser.add_argument('--dialogue-catalog',type=Path,help='Informational dialogue profile; requires --dialogue')
    parser.add_argument('--guide-position',help='Fixed guide x,z metres; default 8004,8000, independent of player spawn')
    parser.add_argument('--quests', action='store_true', help='Opt-in development kill objective; implies progression/creatures, requires account login')
    parser.add_argument('--progression', action='store_true', help='Opt-in durable kill XP; implies creatures, requires account login')
    parser.add_argument('--rewards', action='store_true', help='Opt-in first training-kill reward; implies equipment and creatures, requires account login')
    parser.add_argument('--equipment', action='store_true', help='Opt-in development main-hand equipment; implies inventory and requires --account-login')
    parser.add_argument('--items', type=Path, help='Development item catalog; requires --inventory')
    parser.add_argument('--creatures', action='store_true', help='Opt-in server patrol on baked navigation; implies resources/combat')
    parser.add_argument('--creature-profile', type=Path, help='Patrol definition; requires --creatures')
    parser.add_argument('--navigation', type=Path, help='Offline navigation package; requires --creatures')
    parser.add_argument('--status-effects', action='store_true', help='Opt-in authoritative Scorch periodic damage; implies resources and combat')
    parser.add_argument('--statuses', type=Path, help='Status definitions; requires --status-effects')
    parser.add_argument('--resources', action='store_true', help='Opt-in transient development health/mana and cast costs; implies combat')
    parser.add_argument('--character-stats', type=Path, help='Development vitals profile; requires --resources')
    parser.add_argument('--abilities', type=Path, help='Combat catalog (default: content/gameplay/development-abilities.json); requires --combat')
    parser.add_argument('--targeting', action='store_true', help='Opt-in authoritative training target, Tab selection and Backspace clear')
    parser.add_argument('--dev-speed', type=int, choices=[1, 8, 32], default=1)
    parser.add_argument('--dev-spawn', default='8000,8000', help='Graybox server spawn x,z in metres')
    parser.add_argument('--dev-tour', action='store_true', help='Server-owned teleport circuit for streaming qualification')
    parser.add_argument('--clients', type=int, choices=range(1, 9), default=1)
    parser.add_argument('--binaries', type=Path, default=Path('target/release'),
                        help='Optimized interactive binaries; use target/debug explicitly for debugging')
    parser.add_argument('--tick-hz', type=int, choices=[20, 30, 60], default=30)
    parser.add_argument('--transport', choices=['quinn', 'renet'], default='renet')
    parser.add_argument('--camera-collision', choices=['world', 'visual'],
                        help='Override authored world camera clearance or legacy visual-mesh refinement')
    args = parser.parse_args()
    if args.vendor_catalog and not args.vendors:
        parser.error('--vendor-catalog requires --vendors')
    args.dialogue = args.dialogue or args.vendors
    vendor_catalog = (args.vendor_catalog or Path('content/gameplay/development-vendor.json')).resolve() if args.vendors else None
    if (args.dialogue_catalog or args.guide_position) and not args.dialogue:
        parser.error('--dialogue-catalog/--guide-position require --dialogue')
    dialogue_catalog=(args.dialogue_catalog or Path('content/gameplay/development-dialogue.json')).resolve() if args.dialogue else None
    guide_position=(args.guide_position or '8004,8000') if args.dialogue else None
    args.progression = args.progression or args.quests
    quest_catalog = Path('content/gameplay/development-quest.json').resolve() if args.quests else None
    args.equipment = args.equipment or args.rewards or args.vendors
    args.creatures = args.creatures or args.rewards or args.progression
    progression_catalog = Path('content/gameplay/development-progression.json').resolve() if args.progression else None
    reward_catalog = Path('content/gameplay/development-rewards.json').resolve() if args.rewards else None
    args.inventory = args.inventory or args.equipment
    if args.inventory and not args.account_login:
        parser.error('--inventory requires --account-login')
    if args.items and not args.inventory:
        parser.error('--items requires --inventory')
    item_catalog = (args.items or Path('content/gameplay/development-equipment-items.json' if args.equipment else 'content/gameplay/development-items.json')).resolve() if args.inventory else None
    if args.equipment and json.loads(item_catalog.read_text()).get('schema') != 2:
        parser.error('--equipment requires an equipment-capable item catalog')
    if args.statuses and not args.status_effects:
        parser.error('--statuses requires --status-effects')
    args.resources = args.resources or args.status_effects or args.creatures
    args.combat = args.combat or args.spell_feedback or args.resources
    if args.character_stats and not args.resources:
        parser.error('--character-stats requires --resources')
    if args.abilities and not args.combat:
        parser.error('--abilities requires --combat')
    default_catalog = 'development-status-abilities.json' if args.status_effects else ('development-resource-abilities.json' if args.resources else 'development-abilities.json')
    status_catalog = (args.statuses or Path('content/gameplay/development-statuses.json')).resolve() if args.status_effects else None
    feedback = Path('content/gameplay/development-status-feedback.json' if args.status_effects else 'content/gameplay/development-spell-feedback.json').resolve() if args.spell_feedback else None
    combat_catalog = (args.abilities or Path('content/gameplay') / default_catalog).resolve() if args.combat else None
    character_stats = (args.character_stats or Path('content/gameplay/development-character-stats.json')).resolve() if args.resources else None
    if (args.creature_profile or args.navigation) and not args.creatures:
        parser.error('--creature-profile/--navigation require --creatures')
    creature_profile = (args.creature_profile or Path('content/gameplay/development-creature-patrol.json')).resolve() if args.creatures else None
    navigation = (args.navigation or Path('build/navigation-terrain-v1')).resolve() if args.creatures else None
    graybox = args.graybox_world or args.saved_terrain is not None or (args.world is None and args.chunk_world is None)
    saved_terrain = (args.saved_terrain or Path('build/graybox-terrain-v1')).resolve() if graybox and not args.graybox_world else None
    prediction = graybox if args.prediction is None else args.prediction
    players = args.players or Path('build/synty-character-canonical-stage' if graybox else 'build/reference-scene')
    play(args.world.resolve() if args.world else None, players.resolve(), args.clients, args.binaries.resolve(), args.tick_hz, args.transport,
         args.chunk_world.resolve() if args.chunk_world else None, args.camera_collision, graybox, args.dev_speed, args.dev_spawn, args.dev_tour, prediction, saved_terrain, args.targeting, combat_catalog, feedback, args.persistence, args.account_login, character_stats, item_catalog, status_catalog, creature_profile, navigation, reward_catalog, progression_catalog, args.window_size, args.fullscreen, quest_catalog, dialogue_catalog, guide_position, vendor_catalog)
