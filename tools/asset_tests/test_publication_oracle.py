"""Bound actual source primitives and retained identities during prop publication."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from create_capacity_fixture import create
from shared_images import filename
from verify_multiplayer import verify_bound_publication, covered_cell


class PublicationOracleTests(unittest.TestCase):
    def fixture(self,root):
        create(root,mode='publication')
        path='shared/'+filename(json.loads((root/'chunk_0_0/world.manifest.json').read_text())['shared_images'][0])
        initial=[[0,0]];final=[[i,0] for i in range(7)];events=[]
        for phase,moment in [('prepare_started',0),('prepared',100),('staged',200),('published',300),('settled',2400)]:
            ids=initial if phase in ('prepare_started','prepared') else final
            events.append({'generation':2,'phase':phase,'elapsed_ms':moment,'shared_images_truncated':False,
                'shared_images':[{'path':path,'handle':'Image1'}],
                'roots':[{'id':id,'entity':str(id),'meshes':1,'visibility':'Hidden' if phase=='staged' and id!=[0,0] else 'Inherited'} for id in ids]})
        return initial,final,events

    def test_real_source_graph_and_retained_identity_checks_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)/'fixture';initial,final,events=self.fixture(root)
            verify_bound_publication(events,root,initial,final)

    def test_missing_mesh_changed_retained_root_image_or_visible_staging_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)/'fixture';initial,final,events=self.fixture(root)
            for corruption in ['mesh','entity','image','visibility']:
                bad=copy.deepcopy(events)
                if corruption=='mesh':bad[2]['roots'][1]['meshes']=0
                if corruption=='entity':bad[3]['roots'][0]['entity']='replaced'
                if corruption=='image':bad[4]['shared_images'][0]['handle']='new'
                if corruption=='visibility':bad[2]['roots'][1]['visibility']='Inherited'
                with self.assertRaises(AssertionError,msg=corruption):verify_bound_publication(bad,root,initial,final)

    def test_complete_coverage_uses_nonlocal_contributors_and_negative_cells(self):
        index={'chunks':[{'id':[-1,0],'bounds':[[0.,0.],[128.,128.]]},
                         {'id':[1,0],'bounds':[[-200.,10.],[-150.,20.]]}]}
        self.assertFalse(covered_cell(index,[[-1,0]],[-30.,15.]))
        self.assertTrue(covered_cell(index,[[-1,0],[1,0]],[-30.,15.]))
        self.assertFalse(covered_cell(index,[[-1,0],[1,0]],[0.,15.]))
