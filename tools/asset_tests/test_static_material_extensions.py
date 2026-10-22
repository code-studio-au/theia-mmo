"""Only qualified declaration-bound material payloads may cross static composition."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from asset_tests.test_unity_reframe import fixture
from static_material_extensions import validate
from static_geometry import hierarchy, prune
from merge_static_glb import merge_documents


class ExtensionTests(unittest.TestCase):
    def test_pruning_keeps_exact_retained_extension_payloads_and_declarations(self):
        import copy
        doc,binary=fixture()
        doc['extensionsUsed']=['KHR_materials_unlit','KHR_materials_emissive_strength']
        doc['extensionsRequired']=['KHR_materials_unlit']
        doc['materials'][0]['extensions']={'KHR_materials_unlit':{}}
        doc['materials'].append({'extensions':{'KHR_materials_emissive_strength':{'emissiveStrength':2.6}}})
        before=copy.deepcopy(doc)
        selected={index for index,node in enumerate(doc['nodes']) if 'mesh' in node}
        reduced,_=prune(doc,binary,selected)
        validate(reduced)
        self.assertEqual(doc,before)
        self.assertEqual(reduced['materials'],[doc['materials'][0]])
        self.assertEqual(reduced['extensionsUsed'],['KHR_materials_unlit'])
        self.assertEqual(reduced['extensionsRequired'],['KHR_materials_unlit'])

    def test_declared_unlit_material_survives_merge_without_changing_geometry(self):
        first,binary=fixture();second,_=fixture();second['extensionsUsed']=['KHR_materials_unlit'];second['extensionsRequired']=['KHR_materials_unlit'];second['materials'][0]['extensions']={'KHR_materials_unlit':{}}
        validate(second);hierarchy(second)
        merged=merge_documents(first,second,len(binary));validate(merged)
        self.assertEqual(merged['extensionsUsed'],['KHR_materials_unlit']);self.assertEqual(merged['extensionsRequired'],['KHR_materials_unlit'])
        self.assertEqual(merged['materials'][1],second['materials'][0]);self.assertEqual(merged['meshes'][0],first['meshes'][0])

    def test_unknown_undeclared_nonmaterial_and_index_bearing_payloads_reject(self):
        for fault in ['unknown','undeclared','required','duplicate','node','texture','payload']:
            doc,_=fixture();doc['extensionsUsed']=['KHR_materials_unlit'];doc['materials'][0]['extensions']={'KHR_materials_unlit':{}}
            if fault=='unknown':doc['extensionsUsed']=['EXT_test']
            if fault=='undeclared':doc.pop('extensionsUsed')
            if fault=='required':doc['extensionsRequired']=['KHR_materials_transmission']
            if fault=='duplicate':doc['extensionsUsed']*=2
            if fault=='node':doc['nodes'][0]['extensions']={'KHR_materials_unlit':{}}
            if fault=='texture':doc['materials'][0]['pbrMetallicRoughness']={'extensions':{'KHR_materials_unlit':{}}}
            if fault=='payload':doc['materials'][0]['extensions']['KHR_materials_unlit']={'index':0}
            with self.subTest(fault=fault),self.assertRaises(ValueError):validate(doc)

    def test_bounded_emissive_strength_survives_frame_and_material_composition(self):
        from reframe_unity_static import transform_document
        import numpy as np
        first,binary=fixture();first['extensionsUsed']=['KHR_materials_emissive_strength'];first['materials'][0]['extensions']={'KHR_materials_emissive_strength':{'emissiveStrength':2.6}}
        validate(first);moved,_=transform_document(first,binary,np.eye(4));self.assertEqual(moved['materials'],first['materials'])
        second,_=fixture();merged=merge_documents(first,second,len(binary));validate(merged);self.assertEqual(merged['materials'][0],first['materials'][0])
        for value in [True,-1,float('nan'),float('inf'),17,None,'2.6']:
            first['materials'][0]['extensions']['KHR_materials_emissive_strength']['emissiveStrength']=value
            with self.subTest(value=value),self.assertRaises(ValueError):validate(first)
        first['materials'][0]['extensions']['KHR_materials_emissive_strength']={'emissiveStrength':2.6,'index':0}
        with self.assertRaises(ValueError):validate(first)
