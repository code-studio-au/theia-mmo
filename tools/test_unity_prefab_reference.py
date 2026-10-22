"""Independent reference fault injection; run with the isolated Blender Python."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parent/'asset_tests'))
from test_unity_prefabs import prefab, record, transform, write_records, GUID
from verify_unity_prefabs import verify


class ReferenceTests(unittest.TestCase):
    def test_reference_rejects_matrix_material_lod_and_inactive_scene_corruption(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);write_records(root/'source.prefab',prefab())
            scene={100:record(1,'GameObject',{'m_Name':'parent','m_IsActive':1}),
                   101:record(4,'Transform',transform(100,position=[100,0,0])),
                   501:record(1001,'PrefabInstance',{'m_SourcePrefab':{'guid':GUID,'fileID':100100000},
                       'm_Modification':{'m_TransformParent':{'fileID':101},'m_Modifications':[]}})}
            write_records(root/'scene.unity',scene)
            class Sources:
                by_guid={GUID:'source.prefab'}
                manifest={'archive_sha256':'0'*64}
                def __init__(self,unused):pass
                def checked(self,name):return root/name
                def metadata(self,name):pass
            draw={'instance':501,'source':'source.prefab','renderer':12,'transform':11,
                  'mesh':{'guid':'c'*32,'fileID':43},'materials':[{'guid':'b'*32,'fileID':2100000}],
                  'matrix':[[0,-3,0,104],[2,0,0,2],[0,0,1,0],[0,0,0,1]]}
            plan={'archive_sha256':'0'*64,'scene':'scene.unity','draws':[draw]}
            path=root/'plan.json'
            with patch('verify_unity_prefabs.Sources',Sources):
                path.write_text(json.dumps(plan));self.assertTrue(verify(root,path)['passed'])
                for kind in ['matrix','material','lod','inactive','nan']:
                    bad=copy.deepcopy(plan);d=bad['draws'][0]
                    if kind=='matrix':d['matrix'][0][3]+=1
                    if kind=='material':d['materials'][0]['guid']='d'*32
                    if kind=='lod':d['renderer']=22;d['transform']=21
                    if kind=='inactive':scene[100]['fields']['m_IsActive']=0;write_records(root/'scene.unity',scene)
                    if kind=='nan':d['matrix'][0][0]=float('nan')
                    path.write_text(json.dumps(bad))
                    with self.assertRaises(ValueError,msg=kind):verify(root,path)
                    scene[100]['fields']['m_IsActive']=1;write_records(root/'scene.unity',scene)


if __name__=='__main__':unittest.main()
