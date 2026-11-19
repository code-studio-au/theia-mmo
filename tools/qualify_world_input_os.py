#!/usr/bin/env python3
"""Inject X11 input only into owned Theia windows; synthetic OS, not hardware."""
import argparse
import ctypes as c
import ctypes.util
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time
import urllib.parse
from PIL import Image
from private_identity import Identity
from private_postgres import Cluster
from gpu_validation import check_gpu_diagnostics


def wait(predicate, seconds=20):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        try:
            value=predicate()
            if value:return value
        except (FileNotFoundError,json.JSONDecodeError):pass
        time.sleep(.05)
    raise TimeoutError('Owned X11 input qualification timed out')


def capture_ready(path):
    try:
        with Image.open(path) as picture:
            assert picture.format == 'PNG'
            picture.verify()
        return True
    except (OSError, SyntaxError):
        return False


class Desktop:
    def __init__(self):
        self.held_keys=set()
        self.held_buttons=set()
        self.window=None
        self.x=c.CDLL(ctypes.util.find_library('X11'))
        self.t=c.CDLL(ctypes.util.find_library('Xtst'))
        self.x.XOpenDisplay.argtypes=[c.c_char_p];self.x.XOpenDisplay.restype=c.c_void_p
        self.display=self.x.XOpenDisplay(b':0')
        assert self.display,'No owned desktop display'
        for name,args in [('XRaiseWindow',[c.c_void_p,c.c_ulong]),('XSetInputFocus',[c.c_void_p,c.c_ulong,c.c_int,c.c_ulong]),('XFlush',[c.c_void_p]),('XCloseDisplay',[c.c_void_p])]:
            getattr(self.x,name).argtypes=args
        self.x.XStringToKeysym.argtypes=[c.c_char_p];self.x.XStringToKeysym.restype=c.c_ulong
        self.x.XKeysymToKeycode.argtypes=[c.c_void_p,c.c_ulong];self.x.XKeysymToKeycode.restype=c.c_ubyte
        pu=c.POINTER(c.c_ulong);pi=c.POINTER(c.c_int)
        self.x.XGetInputFocus.argtypes=[c.c_void_p,pu,pi]
        self.x.XQueryTree.argtypes=[c.c_void_p,c.c_ulong,pu,pu,c.POINTER(pu),c.POINTER(c.c_uint)]
        self.x.XQueryPointer.argtypes=[c.c_void_p,c.c_ulong,pu,pu,pi,pi,pi,pi,c.POINTER(c.c_uint)]
        self.x.XGetGeometry.argtypes=[c.c_void_p,c.c_ulong,pu,pi,pi,c.POINTER(c.c_uint),c.POINTER(c.c_uint),c.POINTER(c.c_uint),c.POINTER(c.c_uint)]
        self.x.XFree.argtypes=[c.c_void_p]
        self.t.XTestFakeMotionEvent.argtypes=[c.c_void_p,c.c_int,c.c_int,c.c_int,c.c_ulong]
        self.t.XTestFakeRelativeMotionEvent.argtypes=[c.c_void_p,c.c_int,c.c_int,c.c_ulong]
        self.t.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
        self.t.XTestFakeKeyEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
    def focus(self,window):
        self.window=window
        self.refresh_ancestry()
        # Reparenting window managers stack the decorated root child. Raising
        # only its client child does not bring the owned frame above a peer.
        self.x.XRaiseWindow(self.display,self.frame)
        # Ask the window manager to activate this exact owned client as well;
        # direct X focus does not update its stacking/active-window policy.
        subprocess.run(['wmctrl','-i','-a',hex(window)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=3)
        self.x.XSetInputFocus(self.display,window,2,0)
        self.x.XFlush(self.display)
    def refresh_ancestry(self):
        current=self.window
        for _ in range(8):
            root,parent=c.c_ulong(),c.c_ulong();children=c.POINTER(c.c_ulong)();count=c.c_uint()
            assert self.x.XQueryTree(self.display,current,c.byref(root),c.byref(parent),c.byref(children),c.byref(count))
            if children:self.x.XFree(children)
            if parent.value==root.value:
                self.root=root.value;self.frame=current;break
            current=parent.value
        else:raise AssertionError('Owned window ancestry exceeds bound')
    def owned_focus(self,pointer=False):
        focused,revert=c.c_ulong(),c.c_int()
        self.x.XGetInputFocus(self.display,c.byref(focused),c.byref(revert))
        assert self.window and focused.value==self.window,'Desktop focus left the owned Theia window'
        if pointer:
            # The window manager may reparent a newly mapped window. Check the
            # current ancestry of the same owned client rather than a stale frame.
            self.refresh_ancestry()
            def query(window):
                root,child=c.c_ulong(),c.c_ulong();rx,ry,wx,wy=c.c_int(),c.c_int(),c.c_int(),c.c_int();mask=c.c_uint()
                assert self.x.XQueryPointer(self.display,window,c.byref(root),c.byref(child),c.byref(rx),c.byref(ry),c.byref(wx),c.byref(wy),c.byref(mask))
                return child.value,wx.value,wy.value
            child,root_x,root_y=query(self.root)
            assert child==self.frame,('Pointer left the owned Theia window',{'root_child':hex(child),'owned_frame':hex(self.frame),'owned_client':hex(self.window),'root_xy':[root_x,root_y]})
            _,x,y=query(self.window)
            root=c.c_ulong();gx,gy=c.c_int(),c.c_int();width,height,border,depth=c.c_uint(),c.c_uint(),c.c_uint(),c.c_uint()
            assert self.x.XGetGeometry(self.display,self.window,c.byref(root),c.byref(gx),c.byref(gy),c.byref(width),c.byref(height),c.byref(border),c.byref(depth))
            assert 0<=x<width.value and 0<=y<height.value,'Pointer left owned client area'
    def move(self,window,point,scale):
        self.owned_focus();assert window==self.window
        data=subprocess.check_output(['xwininfo','-id',hex(window)],text=True)
        width=int(re.search(r'Width:\s+(\d+)',data)[1]);height=int(re.search(r'Height:\s+(\d+)',data)[1])
        assert all(math.isfinite(v) for v in point) and 0<=point[0]*scale<width and 0<=point[1]*scale<height,'Requested pointer location lies outside owned client'
        x=int(re.search(r'Absolute upper-left X:\s+(-?\d+)',data)[1]);y=int(re.search(r'Absolute upper-left Y:\s+(-?\d+)',data)[1])
        self.t.XTestFakeMotionEvent(self.display,0,x+round(point[0]*scale),y+round(point[1]*scale),0);self.x.XFlush(self.display);time.sleep(.1)
    def button(self,button,pressed):
        if pressed:self.owned_focus(pointer=True);self.held_buttons.add(button)
        else:self.held_buttons.discard(button)
        self.t.XTestFakeButtonEvent(self.display,button,int(pressed),0);self.x.XFlush(self.display);time.sleep(.08)
    def click(self,window,point,scale,button):
        self.move(window,point,scale);self.button(button,True);self.button(button,False)
    def key_state(self,name,pressed):
        if pressed:self.owned_focus();self.held_keys.add(name)
        else:self.held_keys.discard(name)
        code=self.x.XKeysymToKeycode(self.display,self.x.XStringToKeysym(name.encode()))
        assert code
        self.t.XTestFakeKeyEvent(self.display,code,int(pressed),0);self.x.XFlush(self.display);time.sleep(.1)
    def look(self,x,y):
        self.owned_focus(pointer=True)
        self.t.XTestFakeRelativeMotionEvent(self.display,int(x),int(y),0);self.x.XFlush(self.display);time.sleep(.2)
    def key(self,name):
        self.key_state(name,True);self.key_state(name,False)
    def drag(self,window,point,scale):
        self.move(window,point,scale);self.button(3,True)
        for _ in range(8):
            self.owned_focus(pointer=True)
            self.t.XTestFakeRelativeMotionEvent(self.display,5,0,0);self.x.XFlush(self.display);time.sleep(.03)
        self.button(3,False)
    def close(self):
        for key in list(self.held_keys):self.key_state(key,False)
        for button in list(self.held_buttons):self.button(button,False)
        self.x.XCloseDisplay(self.display)


def owned_window(pid):
    for line in subprocess.check_output(['wmctrl','-lp'],text=True).splitlines():
        fields=line.split(None,4)
        if len(fields)>=3 and fields[2]==str(pid):return int(fields[0],16)
    return None


def camera_checks(desktop,window,state,scale,folder):
    # Locking while over a HUD button must prevent stale UI activation.
    if state()['inventory_open']:
        desktop.key('i');wait(lambda:not state()['inventory_open'])
    width,height,_=state()['window']
    desktop.move(window,[width-182,139],scale)
    desktop.key('F8');wait(lambda:state()['camera_mode']['active'])
    assert state()['cursor']=={'visible':False,'grab':'Locked'}
    before=state()['camera_angles'];desktop.look(30,0)
    wait(lambda:abs(state()['camera_angles'][0]-before[0])>.01)
    desktop.button(1,True);desktop.button(1,False);time.sleep(.2)
    assert not state()['inventory_open'],'Locked reticle mouse also activated UI'
    desktop.key_state('Control_L',True)
    wait(lambda:not state()['camera_mode']['active'] and state()['cursor']['visible'])
    desktop.click(window,[width-182,139],scale,1)
    wait(lambda:state()['inventory_open'])
    desktop.key_state('Control_L',False);time.sleep(.2)
    assert state()['camera_mode']['requested'] and not state()['camera_mode']['active']
    desktop.key('F8');assert state()['camera_mode']['requested'],'Panel consumed camera preference'
    desktop.key('Escape');wait(lambda:state()['camera_mode']['active'])
    desktop.key('Home');time.sleep(.3)
    def aim(field,offset=(0,0)):
        for _ in range(30):
            point=state()[field]
            assert point is not None,f'{field} left the owned viewport'
            dx,dy=point[0]-width/2+offset[0],point[1]-height/2+offset[1]
            if math.hypot(dx,dy)<8:return
            focal=height/(2*math.tan(math.pi/8))
            desktop.look(max(-100,min(100,round(math.atan(dx/focal)/.004))),max(-100,min(100,round(math.atan(dy/focal)/.003))))
        raise AssertionError(f'Reticle did not converge: {state()[field]}')
    aim('guide_screen')
    selected=state()['controls']['friendly_clicks']
    desktop.button(3,True);desktop.button(3,False)
    wait(lambda:state()['controls']['friendly_clicks']>selected)
    desktop.key('f');wait(lambda:state()['dialogue_open'])
    wait(lambda:not state()['camera_mode']['active'] and state()['cursor']['visible'])
    desktop.key('Escape');wait(lambda:not state()['dialogue_open'])
    if state()['inventory_open']:
        desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    wait(lambda:state()['camera_mode']['active'])
    desktop.key('Home');time.sleep(.3)
    for _ in range(16):
        if state()['enemy_screen'] is not None:break
        desktop.look(100,0)
    aim('enemy_screen')
    enemy=state()['controls']['enemy_clicks'];basic=state()['controls']['basic_requests']
    def hit(button,counter,previous):
        # Track the moving visible mesh and retry a bounded set of points within
        # its projected body; an AABB centre need not intersect a triangle.
        for offset in [(0,0),(0,-8),(0,8),(-8,0),(8,0),(0,-16),(0,16)]:
            aim('enemy_screen',offset)
            desktop.button(button,True);desktop.button(button,False);time.sleep(.25)
            if state()['controls'][counter]>previous:return
        raise AssertionError(f'Reticle mesh click did not reach {counter}')
    hit(3,'enemy_clicks',enemy)
    hit(1,'basic_requests',basic)
    (folder/'capture.request').write_text('capture owned action camera window\n')
    wait(lambda:capture_ready(folder/'input-os.png'))
    (folder/'input-os.png').rename(folder/'action-camera.png')
    desktop.key('Escape');wait(lambda:not state()['camera_mode']['requested'])
    assert state()['cursor']['visible']
    return {'toggle_mouse_look':True,'locked_mouse_does_not_activate_ui':True,'temporary_cursor_inventory_suspend_restore':True,'reticle_friendly_and_f_dialogue':True,'reticle_enemy_select_basic_request':True,'escape_releases_mode_without_exiting':True,'escape_panel_priority':True,'world_drag_with_backpack_open':True}


def dodge_checks(desktop,window,state,folder,animation=False):
    def dodge():return state()['dodge']
    def snapshot():return dodge()['snapshot']
    def idle():return snapshot()['policy']['active'] is None
    def ready():return idle() and dodge()['tick']>=snapshot()['policy']['ready_at']
    def position():return dodge()['authority_feet']
    def distance(a,b):return math.hypot(a[0]-b[0],a[2]-b[2])
    proof={}
    if animation:
        wait(lambda:state()['dodge_animation']['available'])
        if not state()['sword_equipped']:
            desktop.key('i');wait(lambda:state()['inventory_open'])
            point=wait(lambda:state()['sword_slot_screen'])
            desktop.click(window,point,state()['window'][2],1)
            point=wait(lambda:state()['equip_screen'])
            desktop.click(window,point,state()['window'][2],1)
            wait(lambda:state()['sword_equipped'])
            desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    wait(ready)
    initial=snapshot()['last_nonce']
    desktop.key('i');wait(lambda:state()['inventory_open'])
    desktop.key('v');time.sleep(.3)
    assert snapshot()['last_nonce']==initial,'Panel key press escaped into gameplay'
    desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    desktop.key('Home');time.sleep(.3)
    before=position();desktop.key('v');wait(lambda:snapshot()['last_nonce']==initial+1);wait(idle)
    time.sleep(.2);after=position()
    assert abs(distance(before,after)-3)<.05,(before,after)
    proof['classic_stationary']={'before':before,'after':after,'snapshot':snapshot()}
    desktop.key_state('v',True);time.sleep(3.4);desktop.key_state('v',False)
    assert snapshot()['last_nonce']==initial+1,'Held key repeated after cooldown'
    assert distance(after,position())<.02,'Cooldown/held key added distance'
    desktop.key('F8');wait(lambda:state()['camera_mode']['active']);desktop.key('Home');time.sleep(.4)
    wait(ready);before=position();desktop.key('v');wait(lambda:snapshot()['last_nonce']==initial+2);wait(idle);time.sleep(.2);after=position()
    assert abs(distance(before,after)-3)<.05,(before,after)
    proof['action_stationary']={'before':before,'after':after,'snapshot':snapshot()}
    wait(ready);desktop.key_state('w',True);before=position();desktop.key_state('v',True)
    wait(lambda:snapshot()['last_nonce']==initial+3)
    accepted=dodge();desktop.key_state('v',False);desktop.key_state('w',False);wait(idle);time.sleep(.3);after=position()
    assert 2.8<distance(before,after)<4.5,(before,after)
    proof['moving']={'before':before,'after':after,'accepted':accepted,'snapshot':snapshot()}
    if animation:
        for nonce,yaw,name in [(4,-math.pi/2,'left'),(5,math.pi/2,'right'),(6,0.,'back')]:
            wait(ready)
            for _ in range(10):
                error=math.atan2(math.sin(yaw-state()['camera_angles'][0]),math.cos(yaw-state()['camera_angles'][0]))
                if abs(error)<.01:break
                desktop.look(max(-100,min(100,round(-error/.004))),0)
            before=position();heading=state()['pawn_heading'];desktop.key('v')
            wait(lambda:snapshot()['last_nonce']==initial+nonce);wait(idle);time.sleep(.3);after=position()
            assert abs(distance(before,after)-3)<.05,(name,before,after)
            assert abs(sum(a*b for a,b in zip(heading,state()['pawn_heading'])))>.9999,'Stationary dodge changed body facing'
            proof[name]={'before':before,'after':after,'snapshot':snapshot()}
        observed=state()['dodge_animation']
        assert all(n>0 for n in observed['starts']),observed
        assert all(n>0 for n in observed['maximum_pose_changes']),observed
        proof['animation']=observed
    proof.update(panel_blocks_dodge=True,held_key_never_repeats=True,classic_and_action_camera=True,no_invulnerability_claim=True)
    (folder/'dodge.json').write_text(json.dumps(proof,indent=2)+'\n')
    return proof


def run(output,binaries,camera_modes=False,dodge=False,dodge_animation=False):
    output=output.resolve();binaries=binaries.resolve();output.mkdir(mode=0o700,parents=True,exist_ok=False)
    bundle=output/'gameplay';shutil.copytree('content/gameplay',bundle)
    creature=bundle/'development-creature-deflect-pursuit.json';data=json.loads(creature.read_text());data.pop('aggro',None);creature.write_text(json.dumps(data)+'\n')
    desktop=Desktop();results={};launcher=None
    try:
        with Identity(output/'provider',Path('build/identity-sdk'),token_seconds=300) as identity, Cluster(output/'cluster',Path('build/postgres-sdk')) as cluster:
            for index,size in enumerate(['1920x1200','3440x1440','1280x720']):
                folder=output/size;folder.mkdir()
                env={k:v for k,v in os.environ.items() if not k.startswith('THEIA_') and k not in ['VK_INSTANCE_LAYERS','VK_LOADER_DEBUG','WGPU_VALIDATION','WGPU_DEBUG']}
                env.update(DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],THEIA_OIDC_ISSUER=identity.issuer,THEIA_OIDC_CLIENT_ID=identity.client_id,THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,THEIA_TEST_BROWSER_ISSUER=identity.issuer,THEIA_TEST_BROWSER_REDIRECT=identity.redirect,THEIA_TEST_BROWSER_USER=identity.users[0][0],THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',THEIA_WORLD_INPUT_OS_QUALIFY=str(folder))
                if dodge_animation:env['THEIA_DODGE_ANIMATION_QUALIFY']='1'
                with (folder/'launcher.log').open('w') as log:
                    command=[sys.executable,'tools/play.py','--account-login','--binaries',str(binaries),'--gameplay-manifest',str(bundle/('development-gameplay-dodge.json' if dodge else 'development-gameplay-projectiles.json')),'--players',str(binaries/'art/player'),'--creature-asset',str(binaries/'art/wolf/wolf.glb'),'--dev-spawn','8002,8002','--window-size',size]
                    if size=='3440x1440':command.append('--fullscreen')
                    launcher=subprocess.Popen(command,env=env,stdin=subprocess.PIPE,stdout=log,stderr=subprocess.STDOUT,text=True)
                    try:
                        launcher.stdin.write((('' if index==0 else 'n\n')+('DodgeArden'+chr(65+index)+'\n')) if dodge else ('PointerArden\n' if index==0 else '1\n'));launcher.stdin.flush()
                        def state():
                            assert launcher.poll() is None,'Owned launcher exited early'
                            return json.loads((folder/'input-os.json').read_text())
                        def ready():
                            d=state()
                            return d if d['connected'] and d['guide_screen'] else None
                        initial=wait(ready,40)
                        status=Path(f'/proc/{initial["pid"]}/status').read_text()
                        assert int(re.search(r'^PPid:\s+(\d+)',status,re.M)[1])==launcher.pid,'Window process is not an owned child'
                        window=wait(lambda:owned_window(initial['pid']));desktop.focus(window);time.sleep(2)
                        scale=initial['window'][2]
                        desktop.click(window,state()['guide_screen'],scale,1)
                        wait(lambda:state()['friendly_selected'])
                        desktop.click(window,state()['guide_screen'],scale,3)
                        wait(lambda:state()['dialogue_open'])
                        desktop.key('Escape');wait(lambda:not state()['dialogue_open'])
                        desktop.key('f');wait(lambda:state()['dialogue_open'])
                        desktop.key('Escape');wait(lambda:not state()['dialogue_open'])
                        if camera_modes:assert state()['inventory_open'],'Vendor dialogue should leave its backpack open'
                        time.sleep(.3)
                        if camera_modes:(folder/'before-world-drag.json').write_text(json.dumps(state(),indent=2))
                        before=state()['camera_angles']
                        width,height,_=state()['window']
                        desktop.drag(window,[width*.65,height*.45],scale)
                        if camera_modes:(folder/'after-world-drag.json').write_text(json.dumps(state(),indent=2))
                        wait(lambda:abs(state()['camera_angles'][0]-before[0])>.005)
                        time.sleep(.2);before_ui=state()['camera_angles']
                        desktop.drag(window,[width-182,139],scale);time.sleep(.3)
                        final=state()
                        assert max(abs(a-b) for a,b in zip(before_ui,final['camera_angles']))<.0001,'UI drag moved the camera'
                        assert final['controls']['friendly_clicks']==2 and final['controls']['interaction_requests']==1,final['controls']
                        camera_result=camera_checks(desktop,window,state,scale,folder) if camera_modes else None
                        dodge_result=dodge_checks(desktop,window,state,folder,dodge_animation) if dodge else None
                        (folder/'capture.request').write_text('capture owned game window\n');wait(lambda:capture_ready(folder/'input-os.png'))
                        launcher.send_signal(signal.SIGINT);assert launcher.wait(timeout=15)==0
                        gpu=check_gpu_diagnostics((folder/'launcher.log').read_text(),allow_surface_resize=size=='3440x1440')
                        results[size]={'passed':True,'camera_modes':camera_result,'dodge':dodge_result,'window':final['window'],'controls':final['controls'],'camera_rotated_on_world_drag':True,'ui_drag_retained_camera':True,'left_select_right_open_and_f':True,'gpu_diagnostics':gpu,'adapter':final['adapter']}
                    finally:
                        if launcher.poll() is None:
                            launcher.send_signal(signal.SIGINT)
                            try:launcher.wait(timeout=15)
                            except subprocess.TimeoutExpired:launcher.kill();launcher.wait()
                        launcher=None
    finally:
        desktop.close()
    result={'passed':True,'scope':'X11 synthetic OS mouse/keyboard to PID-matched owned Theia windows; no physical hardware or Windows claim','fixture':'Private copied gameplay bundle removes NPC perception; no gameplay state or input request is injected by telemetry','results':results}
    (output/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path);parser.add_argument('--binaries',type=Path,required=True)
    parser.add_argument('--camera-modes',action='store_true')
    parser.add_argument('--dodge',action='store_true')
    parser.add_argument('--dodge-animation',action='store_true')
    args=parser.parse_args();result=run(args.output,args.binaries,args.camera_modes,args.dodge or args.dodge_animation,args.dodge_animation);print(json.dumps({'passed':result['passed'],'sizes':list(result['results'])}))
