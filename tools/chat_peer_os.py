"""Two native accounts on one owned server, using only ordinary UI/input paths."""
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time

from account_login import AccountClient
from game_frontend import run as run_frontend
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import wait, owned_window, capture_ready


def qualify(desktop, first_window, first_state, first_capture, session, env, user, output):
    folder = output/'peer'
    folder.mkdir(mode=0o700)
    game, runtime = folder/'game', folder/'runtime'
    game.mkdir(); runtime.mkdir(mode=0o700)
    ready = next(json.loads(line.removeprefix('account_game_ready '))
                 for line in (session/'runtime/server.log').read_text().splitlines()
                 if line.startswith('account_game_ready '))
    first_pid = first_state()['pid']
    command = [part.decode() for part in Path(f'/proc/{first_pid}/cmdline').read_bytes().split(b'\0') if part]
    assert '--game-front-end' in command and '--renet-connect' in command
    first_age = int(subprocess.check_output(['ps', '-p', str(first_pid), '-o', 'etimes='], text=True))
    assert first_age > 60, 'Late-join regression requires an established client older than 60 seconds'
    command[command.index('--game-front-end')+1] = str(runtime)
    index = command.index('--renet-connect')
    ticket = runtime/'peer.token'
    command[index+1:index+3] = ['127.0.0.1:1', str(ticket)]
    if '--ui-preferences' in command:
        command[command.index('--ui-preferences')+1] = str(folder/'interface.json')
    peer_env = client_environment(env)
    peer_env['THEIA_WORLD_INPUT_OS_QUALIFY'] = str(game)
    # The native credential adapter reads the configured provider in its bridge
    # process. This owned qualifier has one bridge thread, so scope that single
    # public issuer setting and restore it after joining the thread.
    previous_issuer = os.environ.get('THEIA_OIDC_ISSUER')
    os.environ['THEIA_OIDC_ISSUER'] = env['THEIA_OIDC_ISSUER']
    stopped = threading.Event()
    bridge_errors = []
    peer = thread = account = None
    try:
        with (folder/'client.log').open('w') as log:
            peer = subprocess.Popen(command, env=peer_env, stdout=log, stderr=log)
            account = AccountClient(ready['account_address'], alive=lambda: peer.poll() is None,
                                    cancelled=stopped.is_set)
            def bridge():
                try:
                    run_frontend(account, runtime, ticket,
                                 lambda: stopped.is_set() or peer.poll() is not None)
                except KeyboardInterrupt:
                    pass
                except Exception as error:
                    bridge_errors.append(type(error).__name__)
            thread = threading.Thread(target=bridge, daemon=True)
            thread.start()
            def state():
                assert peer.poll() is None and not bridge_errors, 'Owned peer or account bridge failed'
                return json.loads((game/'input-os.json').read_text())
            def menu():
                state()
                return json.loads((runtime/'frontend-evidence.json').read_text())
            window = wait(lambda: owned_window(state()['pid']), 60)
            desktop.focus(window)
            wait(lambda: menu()['page'] == 'Login', 30)
            def click(label):
                x,y,w,h = menu()['widgets'][label]
                desktop.click(window, [x+w/2,y+h/2], 1, 1)
                time.sleep(.2)
            def field(label, value):
                click(label)
                desktop.key_state('Control_L', True); desktop.key('a'); desktop.key_state('Control_L', False)
                desktop.key('BackSpace')
                for char in value:
                    shifted = char.isupper() or char == '_'
                    if shifted: desktop.key_state('Shift_L', True)
                    desktop.key({'-':'minus', '_':'minus', ' ':'space'}.get(char, char.lower()))
                    if shifted: desktop.key_state('Shift_L', False)
                    time.sleep(.025)
            field('Username', user[0]); field('Account(Password)', user[1])
            click('Login')
            wait(lambda: menu()['page'] == 'Characters', 60)
            assert menu()['character_count'] == 0
            click('New')
            wait(lambda: menu()['page'] == 'Create' and menu()['preview_ready'], 45)
            click('Appearance(0, 1)')
            wait(lambda: menu()['preview_ready'], 30)
            field('Name', 'Herobirch')
            click('Create')
            wait(lambda: menu()['page'] == 'Characters' and menu()['preview_ready'], 45)
            click('Enter')
            wait(lambda: state()['connected'] and state()['loading']['phase'] == 'Playing', 120)
            wait(lambda: first_state()['snapshot_players'] == 2 and state()['snapshot_players'] == 2, 20)
            assert first_state()['actor'] != state()['actor']
            wait(lambda: all({h['appearance']['body'] for h in read()['modular_heroes']} >= {0, 1}
                             for read in (first_state, state)), 30)
            thread.join(timeout=10)
            assert not thread.is_alive() and not bridge_errors
            # Normal authoritative turning gives the peer a distinct camera view.
            desktop.key_state('a', True)
            time.sleep(.45)
            desktop.key_state('a', False)
            wait(lambda: state()['movement']['direction'] == [0.,0.])
            time.sleep(1)
            states = [first_state, state]
            windows = [first_window, window]
            actors = [read()['actor'] for read in states]
            def capture(name):
                (game/'capture.request').write_text(name)
                wait(lambda: capture_ready(game/'input-os.png'), 30)
                (game/'input-os.png').rename(folder/name)
                (folder/(name+'.json')).write_text(json.dumps(state(), indent=2))
            def send(index, channel, text):
                read, target = states[index], windows[index]
                desktop.focus(target)
                wait(lambda: read()['cursor']['focused'])
                chat = read()['chat']
                if chat['channel'] != channel:
                    button = next(w for w in chat['widgets'] if w['action'] == 'channel')
                    desktop.click(target, button['point'], 1, 1)
                    wait(lambda: read()['chat']['channel'] == channel and read()['chat']['typing'])
                else:
                    desktop.key('Return')
                    wait(lambda: read()['chat']['typing'])
                for char in text:
                    desktop.key('space' if char == ' ' else char)
                    time.sleep(.01)
                wait(lambda: read()['chat']['draft_bytes'] == len(text))
                before = read()['chat']['submitted']
                sent_at = time.monotonic()
                desktop.key('Return')
                wait(lambda: read()['chat']['submitted'] == before+1 and not read()['chat']['pending'])
                assert read()['chat']['reply'] == 'Accepted'
                # Verify actual server delivery to both authenticated clients.
                wait(lambda: all(s()['chat']['history'][-1]['bytes'] == len(text)
                                 and s()['chat']['history'][-1]['channel'] == channel
                                 and s()['chat']['history'][-1]['sender'] == ('Heroarden' if index == 0 else 'Herobirch')
                                 for s in states), 10)
                return read()['chat']['history'][-1]['serial'], sent_at
            def shown(index):
                return all(any(b['actor'] == actors[index] and b['serial'] == s()['chat']['history'][-1]['serial']
                               for b in s()['chat']['bubbles']['entries'])
                           and s()['chat']['bubbles']['visible'] > 0 for s in states)
            send(1, 'Say', 'hello from birch')
            wait(lambda: shown(1), 10)
            desktop.focus(first_window); first_capture('chat-remote-say.png')
            desktop.focus(window); capture('chat-self-say.png')
            send(1, 'Area', 'area from birch')
            assert all(not any(b['serial'] == s()['chat']['history'][-1]['serial']
                               for b in s()['chat']['bubbles']['entries']) for s in states)
            send(0, 'Say', 'hello from arden')
            wait(lambda: shown(0), 10)
            desktop.focus(window); capture('chat-remote-reply.png')
            wait(lambda: all(not s()['chat']['bubbles']['entries'] for s in states), 10)
            # Long unbroken text must wrap rather than expanding the bubble.
            _, sent_at = send(1, 'Say', 'w'*200)
            wait(lambda: shown(1), 10)
            desktop.focus(first_window); first_capture('chat-remote-long.png')
            # Actor retirement removes even an unexpired bubble; history stays.
            retired = actors[1]
            count = len(first_state()['chat']['history'])
            peer.send_signal(signal.SIGINT)
            peer.wait(timeout=15)
            wait(lambda: first_state()['snapshot_players'] == 1, 20)
            wait(lambda: not any(b['actor'] == retired for b in first_state()['chat']['bubbles']['entries']), 5)
            retirement_seconds = time.monotonic() - sent_at
            assert retirement_seconds < 6, 'Retirement check reached natural bubble expiry'
            assert len(first_state()['chat']['history']) == count
            first_capture('chat-peer-retired.png')
            result = dict(native_accounts=2, first_client_age_before_peer_seconds=first_age,
                          distinct_saved_bodies=True, bidirectional_say=True, area_delivery_without_bubble=True,
                          long_text_bytes=200, actor_retirement=True, retirement_seconds=retirement_seconds,
                          history_preserved=True,
                          adapter=first_state()['adapter'])
    finally:
        stopped.set()
        if peer is not None and peer.poll() is None:
            peer.terminate()
            try: peer.wait(timeout=15)
            except subprocess.TimeoutExpired: peer.kill(); peer.wait()
        if thread is not None: thread.join(timeout=10)
        if account is not None: account.close()
        if previous_issuer is None: os.environ.pop('THEIA_OIDC_ISSUER', None)
        else: os.environ['THEIA_OIDC_ISSUER'] = previous_issuer
        if owned_window(first_pid) == first_window:
            desktop.focus(first_window)
    result['gpu_diagnostics'] = check_gpu_diagnostics((folder/'client.log').read_text(), allow_surface_resize=True)
    (folder/'qualification.json').write_text(json.dumps(result, indent=2))
    return result
