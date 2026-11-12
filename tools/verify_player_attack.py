#!/usr/bin/env python3
"""Compare live upper-body sword attacks and lower-body gaits to source curves."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from static_geometry import local_matrix
from verify_synty_character import read, tracks
from verify_player_grip import poses


def verify(asset, folder):
    document, binary = read(asset)
    curves = {a['name']: tracks(document, binary, a) for a in document['animations']}
    nodes = document['nodes']
    spine = [i for i, n in enumerate(nodes) if n.get('name') == 'Spine_01']
    assert len(spine) == 1
    indices = list(spine)
    for i in indices:
        indices.extend(nodes[i].get('children', []))
    upper = {nodes[i]['name'] for i in indices}
    reports = {}
    for peer, local in [('A', True), ('B', False)]:
        report = json.loads((folder/peer/'melee.json').read_bytes())
        metrics = report['attacks']
        assert report['passed'] and metrics['available'] and len(metrics['samples']) <= 8
        results = {}
        for observation in metrics['samples']:
            if observation['local'] != local:
                continue
            name = observation['clip']
            base = poses(document, curves[name], observation['base_time'])
            strike = poses(document, curves['SwordAttack'], observation['attack_time'])
            upper_error = lower_error = 0.
            changed = 0
            assert {'Hand_R', 'Prop_R', 'Hips', 'UpperLeg_R', 'UpperLeg_L'} <= observation['bones'].keys()
            for bone, actual in observation['bones'].items():
                expected = strike[bone] if bone in upper else base[bone]
                error = float(np.max(abs(local_matrix(actual)-local_matrix(expected))))
                if bone in upper:
                    upper_error = max(upper_error, error)
                    changed += int(np.max(abs(local_matrix(strike[bone])-local_matrix(base[bone]))) > .01)
                else:
                    lower_error = max(lower_error, error)
            assert upper_error < .002, (peer, name, 'upper body', upper_error)
            assert lower_error < .002, (peer, name, 'hips/legs/root', lower_error)
            assert changed >= 3, (peer, name, 'Attack indistinguishable from base gait')
            results[name] = {'upper_local_matrix_error': upper_error,
                             'lower_local_matrix_error': lower_error,
                             'changed_upper_targets': changed,
                             'bones': len(observation['bones']),
                             'base_time': observation['base_time'],
                             'attack_time': observation['attack_time']}
        assert {'Idle', 'Walk', 'Run'} <= results.keys(), (peer, results)
        reports[peer] = results
    result = {'passed': True, 'asset_sha256': hashlib.sha256(asset.read_bytes()).hexdigest(),
              'peers': reports, 'scope': 'Actual local and remote native poses: authored SwordAttack above Spine_01, original gait below it; damage remains server owned'}
    (folder/'attack-verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


def verify_directional(asset, folder):
    document, binary = read(asset)
    curves = {a['name']: tracks(document, binary, a) for a in document['animations']}
    nodes = document['nodes']
    indices = [i for i, n in enumerate(nodes) if n.get('name') == 'Spine_01']
    assert len(indices) == 1
    for i in indices:
        indices.extend(nodes[i].get('children', []))
    upper = {nodes[i]['name'] for i in indices}
    reports = {}
    for peer, local in [('owner', True), ('observer', False)]:
        trace = json.loads((folder/peer/'input-os.json').read_bytes())
        metrics = trace['attack_animation']
        assert metrics['available'] and len(metrics['samples']) <= 36
        results = {}
        for observation in metrics['samples']:
            if observation['local'] != local:
                continue
            name = observation['clip']
            base = poses(document, curves[name], observation['base_time'])
            strike = poses(document, curves['SwordAttack'], observation['attack_time'])
            upper_error = lower_error = 0.
            assert {'Hand_R', 'Prop_R', 'Hips', 'UpperLeg_R', 'UpperLeg_L'} <= observation['bones'].keys()
            for bone, actual in observation['bones'].items():
                expected = strike[bone] if bone in upper else base[bone]
                error = float(np.max(abs(local_matrix(actual)-local_matrix(expected))))
                if bone in upper:
                    upper_error = max(upper_error, error)
                else:
                    lower_error = max(lower_error, error)
            assert upper_error < .002, (peer, name, 'upper body', upper_error)
            assert lower_error < .002, (peer, name, 'hips/legs/root', lower_error)
            results[name] = dict(upper_local_matrix_error=upper_error, lower_local_matrix_error=lower_error,
                                 bones=len(observation['bones']), base_time=observation['base_time'],
                                 attack_time=observation['attack_time'])
        assert 'RunBack' in results, (peer, results)
        reports[peer] = results
    result = dict(passed=True, asset_sha256=hashlib.sha256(asset.read_bytes()).hexdigest(), peers=reports,
                  scope='Actual native local/remote sword strike: authored upper body and independent RunBack lower body; server owns damage')
    (folder/'attack-directional-verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset', type=Path)
    parser.add_argument('folder', type=Path)
    parser.add_argument('--directional-os',action='store_true')
    args = parser.parse_args()
    print(json.dumps((verify_directional if args.directional_os else verify)(args.asset, args.folder), indent=2))
