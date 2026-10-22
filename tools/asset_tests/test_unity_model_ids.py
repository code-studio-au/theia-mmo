"""Bounded binary FBX hierarchy and serialized Unity ID cross-checks."""
from pathlib import Path
import struct
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from unity_model_ids import identity, signed_hash, bound_candidates


def fbx(path, wide=False, cycle=False, duplicate=False, collection=False, single=False):
    header_format='<QQQB' if wide else '<IIIB';size=struct.calcsize(header_format)
    def props(items):
        out=b''
        for item in items:
            if isinstance(item,int):out+=b'L'+struct.pack('<q',item)
            else:
                value=item.encode();out+=b'S'+struct.pack('<I',len(value))+value
        return out
    def node(spec,start):
        name,properties,children=spec;name=name.encode();p=props(properties);position=start+size+len(name)+len(p);body=b''
        for child in children:
            value=node(child,position);position+=len(value);body+=value
        body+=b'\0'*size;position+=size
        return struct.pack(header_format,position,len(properties),len(p),len(name))+name+p+body
    models=[('Model',[1,'Cube\0\1Model','Mesh'],[]),('Geometry',[2,'\0\1Geometry','Mesh'],[]),
            ('Model',[3,'Child\0\1Model','Mesh'],[]),('Geometry',[4,'\0\1Geometry','Mesh'],[])]
    connections=[('C',['OO',1,3 if cycle else 0],[]),('C',['OO',3,1],[]),('C',['OO',2,1],[]),('C',['OO',4,3],[])]
    if single:
        models=models[:2];connections=[('C',['OO',1,0],[]),('C',['OO',2,1],[])]
    if duplicate:
        models.append(('Model',[5,'Child\0\1Model','Null'],[]));connections.append(('C',['OO',5,1],[]))
    if collection:
        models.append(('CollectionExclusive',[8,'DisplayLayer','DisplayLayer'],[]))
        connections.extend([('C',['OO',1,8],[]),('C',['OO',3,8],[])])
    data=b'Kaydara FBX Binary  \0\x1a\0'+struct.pack('<I',7500 if wide else 7400)
    for spec in [('Objects',[],models),('Connections',[],connections)]:data+=node(spec,len(data))
    data+=b'\0'*size;path.write_bytes(data)


class UnityModelIdTests(unittest.TestCase):
    def test_published_root_ids_match_signed_64_bit_values(self):
        self.assertEqual(signed_hash('Type:Transform->//RootNode/root/Transform0'),-8679921383154817045)
        self.assertEqual(signed_hash('Type:GameObject->//RootNode/root0'),919132149155446097)
        self.assertEqual(signed_hash('Type:MeshRenderer->//RootNode/root/MeshRenderer0'),-7511558181221131132)

    def test_binary_32_and_64_bit_offsets_resolve_hierarchy_and_mesh_association(self):
        with tempfile.TemporaryDirectory() as directory:
            for wide in [False,True]:
                p=Path(directory)/'model.fbx';fbx(p,wide,collection=True)
                result=identity(p)
                self.assertEqual(result['model_paths'],{1:'//RootNode/root',3:'//RootNode/root/Child'})
                candidate=result['candidates'][signed_hash('Type:Mesh->Child0')]
                self.assertEqual(candidate['model'],3);self.assertEqual(candidate['geometry'],4)
                self.assertEqual(result['candidates'][-8679921383154817045]['model'],1)

    def test_cycles_duplicate_paths_and_out_of_bounds_offsets_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.fbx'
            for options in [{'cycle':True},{'duplicate':True}]:
                fbx(p,**options)
                with self.assertRaises(ValueError):identity(p)
            fbx(p)
            data=bytearray(p.read_bytes());struct.pack_into('<I',data,27,len(data)+100);p.write_bytes(data)
            with self.assertRaisesRegex(ValueError,'bounds'):identity(p)

    def test_explicit_legacy_metadata_ids_bind_only_matching_source_names(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'model.fbx';fbx(p);model=identity(p)
            table=[{'first':{43:4300000},'second':'Cube'},{'first':{4:400000},'second':'//RootNode'}]
            candidates,strategy=bound_candidates(model,{'internalIDToNameTable':table})
            self.assertEqual(strategy,'explicit-metadata-name-table')
            self.assertEqual(candidates[4300000]['geometry'],2)
            self.assertEqual(candidates[400000]['model'],1)
            with self.assertRaisesRegex(ValueError,'duplicate'):
                bound_candidates(model,{'internalIDToNameTable':table+table})
            table[0]['second']='MissingMesh'
            with self.assertRaisesRegex(ValueError,'does not match'):
                bound_candidates(model,{'internalIDToNameTable':table})
