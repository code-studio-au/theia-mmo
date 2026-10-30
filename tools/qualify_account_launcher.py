#!/usr/bin/env python3
"""Actual account launcher cleanup; system-browser command is an owned fixture."""
import argparse
import hashlib
from http.cookiejar import CookieJar, DefaultCookiePolicy
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from private_identity import Identity, _LoginForm, _NoRedirect
from private_postgres import Cluster


def browser(url):
    """Owned fake browser command, actual provider form/PKCE callback. No prints."""
    issuer=urllib.parse.urlsplit(os.environ['THEIA_TEST_BROWSER_ISSUER'])
    parsed=urllib.parse.urlsplit(url)
    assert (parsed.scheme,parsed.netloc)==(issuer.scheme,issuer.netloc)
    if os.environ.get('THEIA_TEST_BROWSER_DEFER')=='1':return
    jar=CookieJar(DefaultCookiePolicy(secure_protocols=('http','https','wss')))
    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),urllib.request.HTTPCookieProcessor(jar),_NoRedirect())
    with opener.open(url,timeout=7) as response:html=response.read(1024*1024+1)
    assert len(html)<=1024*1024
    form=_LoginForm();form.feed(html.decode())
    action=urllib.parse.urlsplit(form.action)
    assert (action.scheme,action.netloc)==(issuer.scheme,issuer.netloc)
    body=urllib.parse.urlencode({'username':os.environ['THEIA_TEST_BROWSER_USER'],'password':os.environ['THEIA_TEST_BROWSER_PASSWORD']}).encode()
    try:
        opener.open(urllib.request.Request(form.action,data=body),timeout=7).close()
        raise RuntimeError('Expected provider redirect')
    except urllib.error.HTTPError as error:
        try:
            assert error.code in (302,303)
            callback=error.headers['Location'];actual=urllib.parse.urlsplit(callback)
            expected=urllib.parse.urlsplit(os.environ['THEIA_TEST_BROWSER_REDIRECT'])
            assert (actual.scheme,actual.netloc,actual.path)==(expected.scheme,expected.netloc,expected.path)
        finally:error.close()
    with opener.open(callback,timeout=7) as response:assert response.code==200


def run(output,identity_sdk,postgres_sdk,client,server_binary=None,modes=None,inventory=False,resources=False,feedback=False,status_effects=False,creatures=False,equipment=False,rewards=False,progression=False,quests=False,dialogue=False,vendors=False):
    dialogue = dialogue or vendors
    equipment = equipment or vendors
    progression = progression or quests
    creatures = creatures or progression
    equipment = equipment or rewards
    creatures = creatures or rewards
    inventory = inventory or equipment
    resources = resources or status_effects or creatures
    assert sys.platform=='linux','Process ownership checks require Linux /proc'
    output.mkdir(mode=0o700,parents=True,exist_ok=False)
    if server_binary is None:
        with (output/'build.log').open('w') as log:
            subprocess.run(['cargo','build','--locked','-p','theia-server','--bin','theia-server','-j2'],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=600)
        server_binary=Path('target/debug/theia-server').resolve()
    binary_hashes={'server_sha256':hashlib.sha256(server_binary.read_bytes()).hexdigest(),'client_sha256':hashlib.sha256(client.read_bytes()).hexdigest()}
    binaries=output/'binaries';binaries.mkdir()
    (binaries/'theia-server').symlink_to(server_binary)
    (binaries/'theia-client').symlink_to(client)
    results=[]
    earned_before=False
    with Identity(output/'provider',identity_sdk) as identity, Cluster(output/'cluster',postgres_sdk) as cluster:
        env=dict(cluster.env,THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                 THEIA_OIDC_ISSUER=identity.issuer,THEIA_OIDC_CLIENT_ID=identity.client_id,
                 THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                 THEIA_TEST_BROWSER_ISSUER=identity.issuer,THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                 THEIA_TEST_BROWSER_USER=identity.users[0][0],THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                 BROWSER=f'{sys.executable} {Path(__file__).resolve()} --owned-browser %s',
                 DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',THEIA_WINDOW_SIZE='1280x720',THEIA_GRAYBOX_AUTOPLAY='1',THEIA_PERFORMANCE_UNCAPPED='1')
        for mode in modes or ['interrupt','child_failure','pending_login_interrupt']:
            session=Path('build/development-session');before=set(session.glob('renet-*'));owned=[]
            with (output/(mode+'.log')).open('w') as log:
                native_env={}
                if inventory:
                    native_env=dict(THEIA_GRAYBOX_QUALIFY=str((output/(mode+'-client')).resolve()),THEIA_GRAYBOX_QUALIFY_SECONDS='120')
                    if rewards:native_env['THEIA_REWARDS_QUALIFY']='1'
                    native_env['THEIA_EQUIPMENT_QUALIFY' if equipment else 'THEIA_INVENTORY_QUALIFY']='A'
                launcher=subprocess.Popen([sys.executable,'tools/play.py','--account-login',*(['--inventory'] if inventory else []),*(['--equipment'] if equipment else []),*(['--rewards'] if rewards else []),*(['--progression'] if progression else []),*(['--quests'] if quests else []),*(['--dialogue'] if dialogue else []),*(['--vendors'] if vendors else []),*(['--resources'] if resources else []),*(['--status-effects'] if status_effects else []),*(['--creatures'] if creatures else []),*(['--spell-feedback'] if feedback else []),'--binaries',str(binaries.resolve())],
                                          env=dict(env,**native_env,THEIA_TEST_BROWSER_DEFER='1' if mode=='pending_login_interrupt' else '0'),stdin=subprocess.PIPE,stdout=log,stderr=subprocess.STDOUT,text=True)
                launcher.stdin.write('Arden\n');launcher.stdin.flush()
                try:
                    deadline=time.monotonic()+40
                    while time.monotonic()<deadline:
                        assert launcher.poll() is None,'Account launcher exited before admission'
                        owned=[int(p) for p in Path(f'/proc/{launcher.pid}/task/{launcher.pid}/children').read_text().split()]
                        server_log=session/'server.log';text=server_log.read_text() if server_log.exists() else ''
                        if mode=='pending_login_interrupt':
                            ready=next((json.loads(line.removeprefix('account_game_ready ')) for line in text.splitlines() if line.startswith('account_game_ready ')),None)
                            if ready and len(owned)==1:
                                try:
                                    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),_NoRedirect())
                                    with opener.open(ready['account_address']+'/development/auth-metrics',timeout=2) as response:metrics=json.load(response)
                                    if metrics['pending_logins']==1 and metrics['active_account_sessions']==0:break
                                except OSError:pass
                        elif len(owned)==2 and 'account_admit ' in text and 'chunks=9' in text:
                            if inventory:
                                report_path=output/(mode+'-client')/'inventory.json'
                                try:report=json.loads(report_path.read_text())
                                except (FileNotFoundError,json.JSONDecodeError):
                                    time.sleep(.05);continue
                                if not report['passed'] or not (report_path.parent/'inventory.png').is_file():
                                    time.sleep(.05);continue
                            break
                        time.sleep(.05)
                    else:raise TimeoutError('Account launcher admission timeout')
                    time.sleep(1.5)
                    ticket_dirs=set(session.glob('renet-*'))-before;assert len(ticket_dirs)==1
                    for directory in ticket_dirs:
                        files=list(directory.iterdir())
                        assert len(files)==(0 if mode=='pending_login_interrupt' else 1)
                        if files:assert files[0].stat().st_mode&0o777==0o600
                    clients=[pid for pid in owned if Path(f'/proc/{pid}/comm').read_text().strip()=='theia-client'];assert len(clients)==(0 if mode=='pending_login_interrupt' else 1)
                    # The native client needs only its private ticket, not SQL credentials.
                    if clients:
                        environment=Path(f'/proc/{clients[0]}/environ').read_bytes()
                        assert b'THEIA_DATABASE_URL=' not in environment
                    if mode!='child_failure':launcher.send_signal(signal.SIGINT)
                    else:os.kill(clients[0],signal.SIGTERM)
                    code=launcher.wait(timeout=20)
                    assert (code!=0) if mode=='child_failure' else (code==0)
                    assert all(not Path(f'/proc/{pid}').exists() for pid in owned)
                    assert all(not directory.exists() for directory in ticket_dirs)
                    text=server_log.read_text()
                    summary=next(json.loads(line.removeprefix('account_persistence_summary ')) for line in text.splitlines() if line.startswith('account_persistence_summary '))
                    assert not summary['failed'] and summary['worker']['stale_records']==0
                    assert summary['worker']['committed_records']>=(0 if mode=='pending_login_interrupt' else 1)
                    if mode=='pending_login_interrupt':
                        api=next(json.loads(line.removeprefix('account_summary ')) for line in text.splitlines() if line.startswith('account_summary '))
                        assert api['cancelled_challenges']==1 and api['pending_logins']==0 and api['active_account_sessions']==0
                        assert 'account_admit ' not in text
                    if creatures and mode!='pending_login_interrupt':
                        transport=next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
                        assert transport['patrol']['moving_ticks']>0 and transport['patrol']['blocked']==0
                    if inventory and mode!='pending_login_interrupt':
                        assert summary['worker']['inventory_ensures']>=1
                        check=cluster.query(f'SELECT count(*) FROM theia.backpacks WHERE revision={5 if rewards else 4 if equipment else 6};')
                        assert check.returncode==0 and check.stdout.strip()=='1'
                    if quests and mode!='pending_login_interrupt':
                        assert summary['worker']['quest_ensures']>=1
                        check=cluster.query('SELECT count(*) FROM theia.quest_objectives WHERE kill_count=1 AND revision=1;')
                        assert check.returncode==0 and check.stdout.strip()=='1'
                        assert summary['worker']['quest_advances']==int(not earned_before)
                    if progression and mode!='pending_login_interrupt':
                        assert summary['worker']['progression_ensures']>=1
                        check=cluster.query('SELECT count(*) FROM theia.progression WHERE experience=25 AND revision=1;')
                        assert check.returncode==0 and check.stdout.strip()=='1'
                        assert summary['worker']['experience_committed']==int(not earned_before)
                    if rewards and mode!='pending_login_interrupt':
                        check=cluster.query('SELECT count(*) FROM theia.reward_claims WHERE consumed_revision=5;')
                        assert check.returncode==0 and check.stdout.strip()=='1'
                        assert summary['worker']['reward_committed']==int(not earned_before)
                    query=cluster.query("SELECT count(*) FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
                    assert query.returncode==0 and query.stdout.strip()=='0'
                    (output/(mode+'-server.log')).write_text(text)
                    assert identity.password not in text and cluster.password not in text
                    logtext=(output/(mode+'.log')).read_text()
                    assert identity.password not in logtext and cluster.password not in logtext
                    if clients:earned_before=True
                    results.append({'mode':mode,'owned_children':len(owned),'native_clients':len(clients),'tickets_removed':True,'final_commit_before_database_close':bool(clients),'pending_login_cancelled':mode=='pending_login_interrupt','database_owner_closed':True,'client_database_environment_removed':True})
                finally:
                    if launcher.poll() is None:
                        launcher.send_signal(signal.SIGINT)
                        try:launcher.wait(timeout=20)
                        except subprocess.TimeoutExpired:launcher.kill();launcher.wait()
                    launcher.stdin.close()
                    for pid in owned:
                        if Path(f'/proc/{pid}').exists():os.kill(pid,signal.SIGTERM)
    result={'passed':True,'platform':'linux','actual_oidc_and_postgres':True,'browser_command_intercepted':True,'physical_browser_tested':False,
            'vendors':vendors,'dialogue':dialogue,'quests':quests,'progression':progression,'rewards':rewards,'equipment':equipment,'creatures':creatures,'status_effects':status_effects,'inventory':inventory,'resources':resources,'spell_feedback':feedback,'results':results,**binary_hashes,'owned_services_stopped':identity.server is None and cluster.server is None}
    (output/'verification.json').write_text(json.dumps(result,indent=2)+'\n');return result


if __name__=='__main__':
    if len(sys.argv)==3 and sys.argv[1]=='--owned-browser':
        try:browser(sys.argv[2])
        except Exception:
            print('Owned browser fixture failed',file=sys.stderr);sys.exit(1)
        sys.exit(0)
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--identity-sdk',type=Path,default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk',type=Path,default=Path('build/postgres-sdk'))
    parser.add_argument('--client',type=Path,default=Path('build/persistence-qualified-v1-20261029/theia-client'))
    parser.add_argument('--server',type=Path,help='Use an already built exact server executable')
    parser.add_argument('--inventory',action='store_true')
    parser.add_argument('--equipment',action='store_true')
    parser.add_argument('--vendors',action='store_true')
    parser.add_argument('--dialogue',action='store_true')
    parser.add_argument('--quests',action='store_true')
    parser.add_argument('--progression',action='store_true')
    parser.add_argument('--rewards',action='store_true')
    parser.add_argument('--creatures',action='store_true')
    parser.add_argument('--status-effects',action='store_true')
    parser.add_argument('--resources',action='store_true')
    parser.add_argument('--spell-feedback',action='store_true')
    parser.add_argument('--mode',action='append',choices=['interrupt','child_failure','pending_login_interrupt'])
    args=parser.parse_args()
    print(json.dumps(run(args.output,args.identity_sdk.resolve(),args.postgres_sdk.resolve(),args.client.resolve(),args.server.resolve() if args.server else None,args.mode,inventory=args.inventory,resources=args.resources,feedback=args.spell_feedback,status_effects=args.status_effects,creatures=args.creatures,equipment=args.equipment,rewards=args.rewards,progression=args.progression,quests=args.quests,dialogue=args.dialogue,vendors=args.vendors),indent=2))
