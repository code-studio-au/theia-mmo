"""Owned native editor check for test-pawn setup, properties and move/undo."""
import argparse,sys,os,json,time,subprocess,signal
from pathlib import Path
sys.path.insert(0,'tools')
from launch_environment import client_environment
from qualify_world_input_os import Desktop,owned_window,wait,capture_ready
from gpu_validation import check_gpu_diagnostics
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args()
out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
source=out/'source.zone.json';original=json.dumps(dict(schema=12,name='Network test pawn editor check',origin=[8192,0,8192],spawn=[8192,0.1,8192],objects=[],test_players=[])).encode();source.write_bytes(original)
report=out/'state.json'
env=client_environment(dict(os.environ,DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',THEIA_PARTICLE_EDITOR_DATA_DIR=str(out/'particle-data')))
x=None;p=None
with (out/'editor.log').open('w') as log:
 try:
  p=subprocess.Popen([str(args.binary.resolve()),'--zone',str(source),'--report',str(report)],env=env,stdout=log,stderr=log)
  w=wait(lambda:owned_window(p.pid),45);x=Desktop();x.focus(w)
  def state():
   assert p.poll() is None,'Editor exited';return json.loads(report.read_text())
  wait(lambda:report.exists() and state().get('menus'),45)
  def fresh():
   old=state()['input_elapsed'];wait(lambda:state()['input_elapsed']>old);return state()
  def click(point):
   x.move(w,point,1);fresh();x.owned_focus(pointer=True);x.button(1,True);x.button(1,False);fresh()
  def chord(key):
   x.key_state('Control_L',True);x.key(key);x.key_state('Control_L',False);fresh()
  def menu(name):
   click(next(m['centre'] for m in state()['menus'] if m['name']==name and m['role']=='button' and min(m['size'])>0))
  def command(name):
   c=wait(lambda:next((c for c in state()['workspace_controls'] if c['target']==f'Command({name})' and c['menu_item'] and min(c['size'])>0),None));click(c['centre'])
  menu('Add');command('AddTestPlayer');wait(lambda:len(state()['document'].get('test_players',[]))==1)
  menu('Content filter');click(wait(lambda:next((c for c in state()['content_list']['controls'] if c['target']=='Filter(TestPlayers)' and min(c['size'])>0),None))['centre']);wait(lambda:state()['content_list']['filter']=='TestPlayers')
  menu('Property actions');command('TestPlayerController(true)');wait(lambda:state()['document']['test_players'][0].get('controller')=='test.walk-turn')
  menu('Property actions');command('TestPlayerEnabled');wait(lambda:not state()['document']['test_players'][0]['enabled'])
  menu('Property actions');command('TestPlayerEnabled');wait(lambda:state()['document']['test_players'][0]['enabled'])
  field=next(f for f in state()['inline_attributes'] if f['field']==0);click(field['centre']);chord('a')
  for key in '8192.125':x.key('period' if key=='.' else key)
  x.key('Return');wait(lambda:state()['document']['test_players'][0]['position'][0]==8192.125)
  field=next(f for f in state()['inline_attributes'] if f['field']==0);click(field['centre'])
  for _ in range(3):x.key('Tab')
  wait(lambda:any(f['field']==3 and f['focused'] for f in state()['inline_attributes']))
  chord('a');x.key('8');x.key('0');x.key('Return');wait(lambda:state()['document']['test_players'][0]['network']['delay_ms']==80)
  def select():
   row=next(c for c in state()['content_list']['controls'] if c['target']=='TestPlayer(1)' and min(c['size'])>0);click(row['centre'])
  select();chord('d');wait(lambda:len(state()['document']['test_players'])==2)
  x.key('Delete');wait(lambda:len(state()['document']['test_players'])==1)
  chord('z');wait(lambda:len(state()['document']['test_players'])==2)
  chord('z');wait(lambda:len(state()['document']['test_players'])==1)
  select();x.key('f');fresh()
  before=state()['document']['test_players'][0]['position'][0]
  h=wait(lambda:next((h for h in state()['move_handles'] if h['axis']==0 and h['tool']=='Move' and h['visible'] and h['screen'] and h['target']),None))
  x.move(w,h['screen'],1);fresh();x.button(1,True);x.move(w,h['target'],1);fresh();x.button(1,False)
  wait(lambda:state()['document']['test_players'][0]['position'][0]>before+1)
  chord('z');wait(lambda:state()['document']['test_players'][0]['position'][0]==before)
  select();x.key('F12');wait(lambda:capture_ready(report.with_suffix('.png')))
  assert source.read_bytes()==original
  (out/'qualification.json').write_text(json.dumps(dict(passed=True,filter=True,add=True,controller=True,enabled=True,inline_position=True,inline_delay=True,duplicate_delete_undo=True,axis_drag_undo=True,source_unchanged=True,gpu=state()['gpu']),indent=2))
 finally:
  if x:x.close()
  if p and p.poll() is None:
   p.send_signal(signal.SIGINT)
   try:p.wait(timeout=10)
   except subprocess.TimeoutExpired:p.terminate();p.wait(timeout=10)
assert check_gpu_diagnostics((out/'editor.log').read_text())==0
