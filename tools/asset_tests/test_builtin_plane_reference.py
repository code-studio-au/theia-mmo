"""Reference integrity/budgets reject corrupt Editor exports independently of authoring."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_builtin_plane_reference import stage, inspect, COLLECTOR, GUID
from unity_editor_reference import BUILD, digest


def report(project):
    return {'schema':1,'unity_version':'6000.6.6f1','platform':'LinuxEditor',
        'request_sha256':digest(project/'Reference/request.json'),'collector_sha256':digest(COLLECTOR),
        'primitive':'Plane','guid':GUID,'file_id':'10209','positions':[0,0,0, 1,0,0, 0,0,1],
        'normals':[0,1,0]*3,'uv0':[0,0,1,0,0,1],'indices':[0,2,1],
        'bounds_center':dict(x=.5,y=0,z=.5),'bounds_size':dict(x=1,y=0,z=1)}


class BuiltinPlaneTests(unittest.TestCase):
    def test_reference_hashes_version_identity_and_staged_collector_are_bound(self):
        with tempfile.TemporaryDirectory(dir=BUILD) as tmp:
            project=Path(tmp)/'project';stage(project,'6000.6.6f1');value=report(project)
            (project/'Reference/plane.json').write_text(json.dumps(value));self.assertTrue(inspect(project)['passed'])
            for key,fault in [('request_sha256','0'*64),('unity_version','6000.6.5f1'),('guid','0'*32),('file_id','10210'),('collector_sha256','0'*64)]:
                changed=dict(value);changed[key]=fault;(project/'Reference/plane.json').write_text(json.dumps(changed))
                with self.subTest(key=key),self.assertRaises(ValueError):inspect(project)
            (project/'Reference/plane.json').write_text(json.dumps(value));(project/'Assets/Editor/TheiaBuiltinPlaneReference.cs').write_text('changed')
            with self.assertRaises(ValueError):inspect(project)

    def test_invalid_indices_nonfinite_streams_normals_and_bounds_reject(self):
        with tempfile.TemporaryDirectory(dir=BUILD) as tmp:
            project=Path(tmp)/'project';stage(project,'6000.6.6f1');value=report(project)
            for key,fault in [('indices',[0,2,3]),('indices',[0,True,1]),('indices',[0,1]),('positions',[float('nan')]*9),('normals',[0]*9),('uv0',[0,0]),('bounds_size',dict(x=0,y=0,z=0))]:
                changed=dict(value);changed[key]=fault;(project/'Reference/plane.json').write_text(json.dumps(changed))
                with self.subTest(key=key),self.assertRaises(ValueError):inspect(project)
