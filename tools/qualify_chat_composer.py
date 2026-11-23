"""Native composer assertions shared by the owned account frontend qualifier.

Messages are synthetic. Evidence records channel/name/length/outcome, not bodies.
The caller owns the window, identity provider, database and server lifecycle.
"""
import ctypes as c
import time
from qualify_world_input_os import wait


def qualify(desktop, window, state, screenshot, saved_name, bubbles=False):
    def chat():
        return state()['chat']

    def click(action):
        item = wait(lambda: next((w for w in chat()['widgets'] if w['action'] == action), None))
        desktop.click(window, item['point'], 1, 1)
        time.sleep(.15)

    def type_text(text):
        for char in text:
            desktop.key('space' if char == ' ' else char)
            time.sleep(.025)

    def send(text, channel):
        before = chat()['submitted']
        desktop.key('Return')
        wait(lambda: chat()['typing'])
        type_text(text)
        wait(lambda: chat()['draft_bytes'] == len(text.encode()))
        desktop.key('Return')
        wait(lambda: chat()['submitted'] == before+1 and not chat()['pending'] and chat()['status'] != 'Sending…')
        assert chat()['draft_bytes'] == 0 and not chat()['typing']
        assert chat()['channel'] == channel
        return chat()['reply']

    wait(lambda: 'submitted' in chat())
    wait(lambda: {w['action'] for w in chat()['widgets']} == {'channel', 'input'})
    assert chat()['status'] == ''
    click('input')
    wait(lambda: chat()['typing'])
    desktop.key_state('w', True)
    try:
        type_text('ijpo1')
        desktop.key('F10')
        time.sleep(.4)
        current = state()
        assert current['movement']['direction'] == [0., 0.], current['movement']
        assert not current['inventory_open']
        assert not current['ability_book']['open']
        assert not current['party']['open']
        assert not current['interface']['open']
        assert current['active_cast'] is None
        desktop.key('Escape')
        wait(lambda: not chat()['typing'] and chat()['draft_bytes'] == 0)
        time.sleep(.2)
        assert state()['movement']['direction'] == [0., 0.], 'Held movement resumed after cancel'
        assert state()['connected'], 'Chat Escape closed the game'
    finally:
        desktop.key_state('w', False)
    assert chat()['submitted'] == 0, 'Cancelled draft was submitted'
    assert send('hello local', 'Say') == 'Accepted'
    assert chat()['history'][-1]['channel'] == 'Say'
    assert chat()['history'][-1]['sender'] == saved_name
    assert chat()['history'][-1]['bytes'] == len('hello local')
    if bubbles:
        wait(lambda:chat()['bubbles']['visible']==1,15)
        assert chat()['bubbles']['entries'][0]['bytes']==len('hello local')
        screenshot('chat-say-bubble.png')
    click('channel')
    wait(lambda: chat()['channel_menu'])
    click('channel_area')
    wait(lambda: chat()['channel'] == 'Area')
    # Switching channel focuses the field, ready to send with Enter.
    type_text('hello area')
    wait(lambda: chat()['draft_bytes'] == len('hello area'))
    desktop.key('Return')
    wait(lambda: chat()['submitted'] == 2 and not chat()['pending'] and chat()['reply'] == 'Accepted')
    assert chat()['history'][-1]['channel'] == 'Area'
    assert chat()['history'][-1]['bytes'] == len('hello area')
    if bubbles:
        area_serial=chat()['history'][-1]['serial']
        assert all(b['serial']!=area_serial for b in chat()['bubbles']['entries'])
    screenshot('chat-say-area.png')
    if bubbles:
        wait(lambda:chat()['bubbles']['visible']==0 and not chat()['bubbles']['entries'],10)
        screenshot('chat-bubble-expired.png')
    # Start a fresh budget window, then deliberately exceed it without retries.
    time.sleep(5.1)
    for _ in range(3):
        assert send('x', 'Area') == 'Accepted'
    accepted = len(chat()['history'])
    assert send('x', 'Area') == 'RateLimited'
    assert len(chat()['history']) == accepted, 'Refusal appeared as an optimistic message'
    assert chat()['status'] == 'Too many messages — please wait'
    screenshot('chat-rate-limit.png')
    desktop.key('Return')
    wait(lambda: chat()['typing'])
    geometry=chat()['widgets']
    type_text('w'*48)
    wait(lambda: chat()['draft_bytes'] == 48)
    assert chat()['widgets'] == geometry, 'Long draft changed composer geometry'
    screenshot('chat-long-draft.png')
    # A real owned peer window keeps window managers from immediately restoring
    # the sole client's focus. Never activate an unrelated user's application.
    x = desktop.x
    x.XCreateSimpleWindow.argtypes = [c.c_void_p, c.c_ulong, c.c_int, c.c_int,
                                     c.c_uint, c.c_uint, c.c_uint, c.c_ulong, c.c_ulong]
    x.XCreateSimpleWindow.restype = c.c_ulong
    x.XMapWindow.argtypes = [c.c_void_p, c.c_ulong]
    x.XDestroyWindow.argtypes = [c.c_void_p, c.c_ulong]
    peer = x.XCreateSimpleWindow(desktop.display, desktop.root, 10, 10, 160, 80, 0, 0, 0)
    assert peer, 'Cannot create owned focus fixture'
    try:
        x.XMapWindow(desktop.display, peer)
        x.XFlush(desktop.display)
        time.sleep(.3)
        desktop.focus(peer)
        wait(lambda: not state()['cursor']['focused'])
        wait(lambda: not chat()['typing'] and chat()['draft_bytes'] == 0)
        assert chat()['status'] != 'Chat unavailable', 'Window focus was mistaken for connection loss'
        desktop.focus(window)
        wait(lambda: state()['cursor']['focused'])
    finally:
        x.XDestroyWindow(desktop.display, peer)
        x.XFlush(desktop.display)
    assert chat()['submitted'] == 6
    desktop.key_state('w', True)
    try:
        wait(lambda: state()['movement']['direction'] != [0., 0.])
    finally:
        desktop.key_state('w', False)
    wait(lambda: state()['movement']['direction'] == [0., 0.])
    return dict(say_echo=True, area_echo=True, saved_name=True, rate_refusal=True,
                no_optimistic_echo=True, typing_consumes_gameplay=True,
                escape_cancel=True, focus_loss_discards=True, held_key_release=True, long_draft_clipped=True)


def run(args):
    """Focused repeatable account/chat run without redoing unrelated menu checks."""
    import json
    import os
    from pathlib import Path
    import subprocess
    import sys
    import urllib.parse
    from edit_zone import build_environment
    from private_identity import Identity
    from private_postgres import Cluster
    from qualify_world_input_os import Desktop, owned_window, capture_ready
    from gpu_validation import check_gpu_diagnostics

    os.environ['DISPLAY'] = args.display
    out = args.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    with Identity(out/'provider', Path('build/identity-sdk'), token_seconds=1800, interactive=True, users=2 if args.peer_check or args.switch_account else 1) as identity, \
            Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
        game = out/'game'
        game.mkdir()
        session = out/'session'
        env = {k: v for k, v in build_environment(os.environ).items() if not k.startswith('THEIA_')}
        env.update(DISPLAY=args.display, XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                   VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                   THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                   THEIA_ACCOUNT_SESSION_SECONDS='1800', THEIA_OIDC_ISSUER=identity.issuer,
                   THEIA_OIDC_CLIENT_ID=identity.client_id,
                   THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                   BROWSER='/bin/false', THEIA_WORLD_INPUT_OS_QUALIFY=str(game))
        if args.software_vulkan:env['VK_ICD_FILENAMES']='/usr/share/vulkan/icd.d/lvp_icd.json'
        command = [sys.executable, 'tools/account_population_playtest.py', '--package', str(args.package.resolve()),
                   '--players', str(args.players.resolve()), '--binaries', str(Path('target/release').resolve()),
                   '--output', str(session), '--parties', '--chat-area', '7', '--window-size', '1920x1080',
                   '--fantasy-ui', '/home/steven/Downloads/Synty Models/HUDs',
                   '--ui-preferences', str(out/'interface.json'),
                   '--gameplay-manifest', str(args.package.resolve()/'world/gameplay/gameplay.manifest.json')]
        desktop = None
        with (out/'launcher.log').open('w') as log:
            process = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=log, stderr=log)
            try:
                def alive():
                    assert process.poll() is None, 'Owned launcher exited'

                def state():
                    alive()
                    return json.loads((game/'input-os.json').read_text())

                def menu():
                    alive()
                    return json.loads((session/'runtime/frontend-evidence.json').read_text())

                first = wait(state, 120)
                window = wait(lambda: owned_window(first['pid']), 60)
                desktop = Desktop(args.display)
                desktop.focus(window)
                wait(lambda: menu()['page'] == 'Login')

                def click(label):
                    x, y, width, height = menu()['widgets'][label]
                    desktop.click(window, [x+width/2, y+height/2], 1, 1)
                    time.sleep(.2)

                def type_field(label, value):
                    click(label)
                    desktop.key_state('Control_L', True)
                    desktop.key('a')
                    desktop.key_state('Control_L', False)
                    desktop.key('BackSpace')
                    for char in value:
                        shifted = char.isupper() or char == '_'
                        if shifted: desktop.key_state('Shift_L', True)
                        desktop.key({'-': 'minus', '_': 'minus', ' ': 'space'}.get(char, char.lower()))
                        if shifted: desktop.key_state('Shift_L', False)
                        time.sleep(.025)

                def screenshot(name):
                    (game/'capture.request').write_text(name)
                    wait(lambda: (alive() or capture_ready(game/'input-os.png')), 30)
                    (game/'input-os.png').rename(out/name)
                    (out/(name+'.json')).write_text(json.dumps(state(), indent=2))

                type_field('Username', identity.users[0][0])
                type_field('Account(Password)', identity.users[0][1])
                click('Login')
                wait(lambda: menu()['page'] == 'Characters', 60)
                assert menu()['character_count'] == 0
                click('New')
                wait(lambda: menu()['page'] == 'Create' and menu()['preview_ready'], 45)
                refocus_check = getattr(args, 'creation_focus', False)
                type_field('Name', 'Zed' if refocus_check else 'Heroarden')
                click('Create')
                wait(lambda: menu()['page'] == 'Characters' and menu()['preview_ready'], 45)
                saved_name = json.loads((session/'runtime/frontend-state.json').read_text())['characters'][0]['name']
                if refocus_check:
                    click('New')
                    wait(lambda: menu()['page'] == 'Create' and menu()['preview_ready'], 45)
                    click('Category(Face)')
                    click('Appearance(1, 2)')
                    wait(lambda: menu()['appearance']['face'] == 2 and menu()['preview_ready'], 45)
                    for _ in range(3):
                        click('Zoom(true)')
                    type_field('Name', 'steve')
                    screenshot('name-after-customising.png')
                    click('Create')
                    wait(lambda: menu()['page'] == 'Characters' and menu()['character_count'] == 2
                         and menu()['preview_ready'], 45)
                    roster = json.loads((session/'runtime/frontend-state.json').read_text())['characters']
                    assert [row['name'] for row in roster] == ['steve', 'Zed'], roster
                    assert roster[0]['appearance']['face'] == 2, roster
                    saved_name = 'steve'
                    screenshot('new-name-selected.png')
                if args.switch_account:
                    def roster():
                        return json.loads((session/'runtime/frontend-state.json').read_text())['characters']
                    original = roster()
                    click('SignOut')
                    wait(lambda: menu()['page'] == 'Login' and menu()['character_count'] == 0
                         and not menu()['preview_visible'], 30)
                    screenshot('account-signed-out.png')
                    type_field('Username', identity.users[1][0])
                    type_field('Account(Password)', identity.users[1][1])
                    click('Login')
                    wait(lambda: menu()['page'] == 'Characters', 60)
                    assert roster() == [] and not menu()['preview_visible']
                    screenshot('account-second-empty.png')
                    click('New')
                    wait(lambda: menu()['page'] == 'Create' and menu()['preview_ready'], 45)
                    type_field('Name', 'Herobirch')
                    click('Create')
                    wait(lambda: menu()['page'] == 'Characters' and menu()['preview_ready'], 45)
                    assert len(roster()) == 1 and roster()[0]['id'] != original[0]['id']
                    click('SignOut')
                    wait(lambda: menu()['page'] == 'Login' and menu()['character_count'] == 0, 30)
                    type_field('Username', identity.users[0][0])
                    type_field('Account(Password)', identity.users[0][1])
                    click('Login')
                    wait(lambda: menu()['page'] == 'Characters' and menu()['preview_ready'], 60)
                    assert roster() == original, 'Account switch mixed character rosters'
                    screenshot('account-first-restored.png')
                click('Enter')
                wait(lambda: state()['connected'] and state()['loading']['phase'] == 'Playing', 120)
                print('Native account world entry passed', flush=True)
                result = qualify(desktop, window, state, screenshot, saved_name, args.bubbles)
                if args.peer_check:
                    from chat_peer_os import qualify as qualify_peer
                    result['peer_check'] = qualify_peer(desktop, window, state, screenshot, session,
                                                       env, identity.users[1], out)
                result['adapter'] = state()['adapter']
                result['say_bubbles'] = args.bubbles
                result['account_switch'] = args.switch_account
                if refocus_check:
                    result['creation_focus'] = True
                screenshot('chat-complete.png')
                print(json.dumps(result), flush=True)
            finally:
                if desktop: desktop.close()
                if process.poll() is None:
                    process.stdin.close()
                    try: process.wait(timeout=25)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
        result['gpu_diagnostics'] = check_gpu_diagnostics((out/'launcher.log').read_text(), allow_surface_resize=True)
        for path in out.rglob('*'):
            if path.is_file() and path.suffix in ('.json', '.log'):
                assert all(password not in path.read_text(errors='replace') for _,password in identity.users), 'Credential leaked to evidence'
    (out/'qualification.json').write_text(json.dumps(result, indent=2))


if __name__ == '__main__':
    import argparse
    from pathlib import Path
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--package', type=Path, default=Path('build/modular-hero-v1/native-final-v2/fresh/package'))
    parser.add_argument('--players', type=Path, default=Path('build/modular-hero-v1/art-v3'))
    parser.add_argument('--display', default=':0')
    parser.add_argument('--software-vulkan', action='store_true')
    parser.add_argument('--bubbles', action='store_true')
    extra = parser.add_mutually_exclusive_group()
    extra.add_argument('--peer-check', action='store_true', help='Two native accounts, remote Say/Area and retirement')
    extra.add_argument('--creation-focus', action='store_true',
                       help='Refocus the name field after cosmetic changes and enter with the exact saved name')
    extra.add_argument('--switch-account', action='store_true', help='Sign out and switch accounts twice in one game window')
    run(parser.parse_args())
