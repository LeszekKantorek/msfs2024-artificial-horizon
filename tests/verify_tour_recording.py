"""Verify the recorded S8 tour run; receipt times and SSE sequences are separate axes.

Run: python tests/verify_tour_recording.py <diagnostic.jsonl> <sse.jsonl> <scenario.jsonl>
The fixed windows identify the recorded 2026-11-07 run, not a general timing oracle.
"""
import json
import hashlib
from pathlib import Path
import sys

assert len(sys.argv) == 4
native, sse, scenario = sys.argv[1:]
scenario_hash = hashlib.sha256(Path(scenario).read_bytes()).hexdigest()
assert scenario_hash == '3da8856f9b050bcf60fbc3e30028772909cac472152f9afb17adceddf985724f'
session = '1794083704133073100-25952-0'
# Journal elapsed seconds are bounded around established holds, allowing startup delay.
native_checks = {
    'tas': (43, 55, 'AIRSPEED TRUE', 68.25),
    'gs': (43, 55, 'GROUND VELOCITY', 71.5),
    'turn_left': (120, 125, 'TURN INDICATOR RATE', -3),
    'ball_left': (120, 125, 'TURN COORDINATOR BALL', 31.75),
    'turn_right': (135, 140, 'TURN INDICATOR RATE', 3),
    'ball_right': (135, 140, 'TURN COORDINATOR BALL', -31.75),
    'baro': (120, 125, 'KOHLSMAN SETTING HG:1', 29.92),
    'heading_target': (120, 125, 'AUTOPILOT HEADING LOCK DIR:0', 90),
    'altitude_target': (120, 125, 'AUTOPILOT ALTITUDE LOCK VAR:0', 2000),
    'approach_target': (150, 155, 'AUTOPILOT ALTITUDE LOCK VAR:0', 0),
    'second_pass_tas': (251.2, 253, 'AIRSPEED TRUE', 0),
}
found=set(); transitions=[]; identity=False; executable_identity=False
with open(native, encoding='utf-8') as f:
 for line in f:
  x=json.loads(line); assert x['session_id']==session
  t=int(x['elapsed_us'])/1e6; d=x['data']
  if x['kind']=='environment': executable_identity=d['executable']['sha256']=='036be31e791e1c735ccac0c07052af779b1a188d1543e9cfa62aea393f834600'
  if x['kind']=='dll_identity': identity=d['sha256']=='86a9cb40817b5369c446c80eb2dd1d53c753f17cc67f5905f1487238d460b092'
  if x['kind']=='state_transition': transitions.append((d['published']['sequence'],d['published']['state']))
  if x['kind']=='instrument_sample':
   for k,(lo,hi,name,value) in native_checks.items():
    if lo<=t<=hi and d['name']==name and d['value']['value']==value: found.add(k)
  if x['kind']=='attitude_raw':
   raw=d['raw']
   if 15.2<=t<=16 and raw['pitch_radians']['value']==0: found.add('native_normal_resume')
   if 19.2<=t<=20 and raw['pitch_radians']['value']==0: found.add('native_active_resume')
   if 43<=t<=55 and abs(raw['pitch_radians']['value']+0.20944)<1e-8: found.add('native_pitch')
   if 120<=t<=125 and abs(raw['bank_radians']['value']-0.523599)<1e-8: found.add('native_bank_left')
   if 135<=t<=140 and abs(raw['bank_radians']['value']+0.523599)<1e-8: found.add('native_bank_right')
assert identity and executable_identity
assert set(native_checks)|{'native_pitch','native_bank_left','native_bank_right','native_normal_resume','native_active_resume'} == found, found
assert [(721,'paused'),(722,'waiting'),(723,'live'),(1440,'paused'),(1441,'waiting'),(1442,'live'),(75960,'disconnected'),(75963,'live')] == [v for v in transitions if v[0] in {721,722,723,1440,1441,1442,75960,75963}]
# These sequence bounds identify independent windows of the same captured SSE stream;
# no claim is made that a native record and SSE sample share an acquisition timestamp.
def near(x,y,tol=0.001): return isinstance(x,(float,int)) and abs(x-y)<tol
sse_found=set()
with open(sse, encoding='utf-8') as f:
 for line in f:
  x=json.loads(line); q=x['sequence']
  if x['state']!='live': continue
  a=x.get('attitude') or {}
  if 11000<=q<=13000 and near(a.get('pitch_deg'),12,0.01) and near(x.get('tas_kt'),68.25) and near(x.get('gs_kt'),71.5): sse_found.add('climb')
  if 37000<=q<=40000 and near(a.get('roll_deg'),-30,0.01) and near(x.get('turn_rate_dps'),-3) and near(x.get('slip_skid'),0.25) and near(x.get('tas_kt'),115.5) and near(x.get('gs_kt'),121) and near(x.get('selected_heading_deg'),90) and near(x.get('selected_altitude_ft'),2000) and x.get('baro_text')=='29.92 inHg' and near((x.get('baro_setting') or {}).get('pressure_hpa'),1013.20749,0.01): sse_found.add('left')
  if 42000<=q<=46000 and near(a.get('roll_deg'),30,0.01) and near(x.get('turn_rate_dps'),3) and near(x.get('slip_skid'),-0.25): sse_found.add('right')
  if 50000<=q<=54000 and near(a.get('pitch_deg'),-3,0.01) and near(x.get('selected_altitude_ft'),0): sse_found.add('approach')
  if 75963<=q<76680 and near(x.get('ias_kt'),0) and near(x.get('tas_kt'),0) and near(x.get('gs_kt'),0) and near(a.get('pitch_deg'),0) and near(x.get('selected_altitude_ft'),2000): sse_found.add('second_pass')
assert sse_found=={'climb','left','right','approach','second_pass'}, sse_found
print('PASS scenario SHA256', scenario_hash)
print('PASS native receipt',sorted(found),'state transitions',transitions)
print('PASS SSE independent sequence windows',sorted(sse_found))
