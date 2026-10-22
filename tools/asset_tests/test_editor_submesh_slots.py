"""Editor material-slot geometry must not be swapped, dropped or rewound."""
import copy
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from asset_tests.test_unity_reframe import fixture
from verify_unity_submesh_slots import compare


def data():
    doc,binary=fixture();doc['materials']=[{'extras':{'theia_source_slot':0}},{'extras':{'theia_source_slot':1}}]
    face=np.array([[0],[2],[1]],dtype='<u4');offset=len(binary);binary+=face.tobytes();view=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':offset,'byteLength':face.nbytes})
    idx=len(doc['accessors']);doc['accessors'].append({'bufferView':view,'componentType':5125,'type':'SCALAR','count':3});second=copy.deepcopy(doc['meshes'][0]['primitives'][0]);second['indices']=idx;second['material']=1;doc['meshes'][0]['primitives'].append(second);doc['buffers'][0]['byteLength']=len(binary)
    mesh={'positions':[0,0,0,-2,0,0,0,3,1],'submeshes':[{'topology':0,'indices':[0,2,1]},{'topology':0,'indices':[0,1,2]}]}
    return doc,binary,mesh


class SlotTests(unittest.TestCase):
    def test_exact_editor_slot_geometry_accepts_cyclic_order_and_preserves_winding(self):
        doc,binary,mesh=data();self.assertEqual(compare(doc,binary,mesh,2),{'0':1,'1':1})
        mesh['submeshes'][0]['indices']=[2,1,0];self.assertEqual(compare(doc,binary,mesh,2),{'0':1,'1':1})

    def test_swaps_missing_duplicates_unknown_slots_and_corrupt_editor_indices_reject(self):
        for fault in ['swap','drop','duplicate','unknown','bool','winding','editor','topology','pose']:
            doc,binary,mesh=data();primitives=doc['meshes'][0]['primitives']
            if fault=='swap':primitives[0]['material']=1;primitives[1]['material']=0
            if fault=='drop':primitives.pop()
            if fault=='duplicate':primitives.append(copy.deepcopy(primitives[0]))
            if fault=='unknown':doc['materials'][0]['extras']['theia_source_slot']=9
            if fault=='bool':doc['materials'][0]['extras']['theia_source_slot']=False
            if fault=='winding':primitives[0]['indices']=primitives[1]['indices']
            if fault=='editor':mesh['submeshes'][0]['indices'][0]=3
            if fault=='topology':mesh['submeshes'][0]['topology']=2
            if fault=='pose':doc['nodes'][0]['translation']=[1,0,0]
            with self.subTest(fault=fault),self.assertRaises(ValueError):compare(doc,binary,mesh,2)
