#!/usr/bin/env python3
"""Compare bounded live Bevy grip-layer samples with the original GLB curves."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import numpy as np
from static_geometry import local_matrix
from verify_synty_character import read, tracks, sample

DIGITS = {'Thumb_01_1','Thumb_02_1','Thumb_03_1','IndexFinger_01_1',
          'IndexFinger_02_1','IndexFinger_03_1','IndexFinger_04_1',
          'Finger_01_1','Finger_02_1','Finger_03_1','Finger_04_1'}


def poses(document, curves, time):
    nodes=copy.deepcopy(document['nodes'])
    for index,path,times,values,step in curves:
        nodes[index][path]=sample(times,values,time,path=='rotation',step).tolist()
    return {node['name']:node for node in nodes}


def verify(asset, folder):
    document,binary=read(asset)
    curves={animation['name']:tracks(document,binary,animation) for animation in document['animations']}
    closed=poses(document,curves['SwordIdle'],0.)
    reports={}
    for peer,local in [('first-a',True),('first-b',False)]:
        report=json.loads((folder/peer/'inventory.json').read_bytes())
        grip=report['grip']
        assert grip and grip['available'] and len(grip['samples'])<=8
        results={}
        for observation in grip['samples']:
            if observation['local']!=local:continue
            name=observation['clip'];base=poses(document,curves[name],observation['time'])
            digit_error=body_error=0.;different=0
            assert DIGITS <= observation['bones'].keys()
            assert {'Hand_R','Prop_R','Hips','UpperLeg_R','UpperLeg_L'}<=observation['bones'].keys()
            for bone,actual in observation['bones'].items():
                expected=closed[bone] if bone in DIGITS else base[bone]
                error=float(np.max(abs(local_matrix(actual)-local_matrix(expected))))
                if bone in DIGITS:
                    digit_error=max(digit_error,error)
                    different+=int(np.max(abs(local_matrix(closed[bone])-local_matrix(base[bone])))>.01)
                else:body_error=max(body_error,error)
            assert digit_error<.002,(peer,name,'digits',digit_error)
            assert body_error<.002,(peer,name,'body/wrist/socket',body_error)
            assert different>=3,(peer,name,'Grip is indistinguishable from relaxed locomotion')
            results[name]={'digit_local_matrix_error':digit_error,'body_wrist_socket_local_matrix_error':body_error,'digits_changed_from_base':different,'bones':len(observation['bones']),'time':observation['time']}
        assert {'Walk','Run'}<=results.keys(),(peer,results)
        reports[peer]=results
    result={'passed':True,'asset_sha256':hashlib.sha256(asset.read_bytes()).hexdigest(),
            'peers':reports,'scope':'Actual native local and remote walking/running: authored closed fingers; body, wrist and prop socket remain on original locomotion curves'}
    (folder/'grip-verification.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


def verify_dodge(asset, folder):
    document, binary = read(asset)
    curves = {a['name']: tracks(document, binary, a) for a in document['animations']}
    closed = poses(document, curves['SwordIdle'], 0.)
    trace = json.loads((folder / 'input-os.json').read_bytes())
    grip = trace['grip']
    assert grip['available'] and len(grip['samples']) <= 12
    results = {}
    for observation in grip['samples']:
        name = observation['clip']
        if not name.startswith('Dodge') or not observation['local']:
            continue
        base = poses(document, curves[name], observation['time'])
        digit_error = body_error = 0.
        assert DIGITS <= observation['bones'].keys()
        assert {'Hand_R', 'Prop_R', 'Hips', 'UpperLeg_R', 'UpperLeg_L'} <= observation['bones'].keys()
        for bone, actual in observation['bones'].items():
            expected = closed[bone] if bone in DIGITS else base[bone]
            error = float(np.max(abs(local_matrix(actual) - local_matrix(expected))))
            if bone in DIGITS: digit_error = max(digit_error, error)
            else: body_error = max(body_error, error)
        assert digit_error < .002, (name, 'digits', digit_error)
        assert body_error < .002, (name, 'body/wrist/socket', body_error)
        results[name] = {'digit_local_matrix_error': digit_error,
                         'body_wrist_socket_local_matrix_error': body_error,
                         'bones': len(observation['bones']), 'source_time': observation['time']}
    assert set(results) == {'DodgeForward', 'DodgeBack', 'DodgeLeft', 'DodgeRight'}, results
    result = {'passed': True, 'asset_sha256': hashlib.sha256(asset.read_bytes()).hexdigest(),
              'clips': results, 'scope': 'Native local directional dodge: closed sword fingers; body/wrist/socket use retimed authored curves; no remote-player claim'}
    (folder / 'grip-dodge-verification.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset',type=Path);parser.add_argument('folder',type=Path)
    parser.add_argument('--dodge-os',action='store_true')
    args=parser.parse_args();print(json.dumps((verify_dodge if args.dodge_os else verify)(args.asset,args.folder),indent=2))
