"""Check collision coordinate interpretation and the complete-world preparation."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np
import trimesh

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from merge_static_glb import read
from prepare_map_world import collision_plan, model_bounds, prepare
from test_map_placement import row


class MapWorldTests(unittest.TestCase):
    def test_m2_collision_bounds_use_exported_obj_axes_and_reject_invalid_data(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'tree.obj'
            metadata = path.with_suffix('.json')
            def write(lo, hi):
                metadata.write_text(json.dumps({'fileType': 'm2', 'collisionBox': {'min': lo, 'max': hi}}))
            write([-1, -2, 0], [1, 3, 20])
            points, _ = model_bounds(path)
            np.testing.assert_allclose(points.min(axis=0), [-1, 0, -3], atol=1e-12)
            np.testing.assert_allclose(points.max(axis=0), [1, 20, 2], atol=1e-12)
            for lo, hi in [([0]*3, [0]*3), ([10000000]*3, [-10000000]*3)]:
                write(lo, hi)
                self.assertIsNone(model_bounds(path)[0])
            write([1]*3, [-1]*3)
            with self.assertRaises(ValueError): model_bounds(path)

    def test_complete_world_preserves_visual_binary_and_places_collision_with_same_origin(self):
        import csv
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'terrain.obj'
            source.write_text('v 0 10 2000\nv 0 11 2100\nv 100 12 2000\nf 1 2 3\n')
            model = root / 'object.obj'
            model.write_text('v -1 0 -2\nv 1 0 -2\nv 1 4 2\nf 1 2 3\n')
            model.with_suffix('.json').write_text(json.dumps({'fileType': 'm2', 'collisionBox': {'min': [-1,-2,0], 'max': [1,2,4]}}))
            placements = root / 'placements.csv'
            placement = row(PositionX=str(51200/3 - 40), PositionZ=str(51200/3 - 2050), PositionY='10', RotationY='-90')
            with placements.open('w', newline='') as file:
                writer = csv.DictWriter(file, placement.keys(), delimiter=';')
                writer.writeheader(); writer.writerow(placement)
            liquids = root / 'liquids.json'
            chunks = [None]*256
            chunks[0] = {'instances': [{'chunkIndex':0, 'instanceIndex':0,
                'width':2, 'height':1, 'xOffset':0, 'yOffset':0,
                'terrainChunkPosition':[2050,40,0], 'bitmap':[1],
                'vertexData':{'height':[10]*6}}]}
            liquids.write_text(json.dumps({'liquidChunks': chunks}))
            scene = trimesh.Scene(trimesh.creation.box())
            visual = root / 'visual.glb'
            visual.write_bytes(trimesh.exchange.gltf.export_glb(scene))
            original_doc, binary = read(visual)
            original_hash = hashlib.sha256(visual.read_bytes()).hexdigest()
            output = root / 'world'
            report = prepare(source, placements, liquids, visual, output, [10,10,2020])
            doc, result_binary = read(output / 'terrain.glb')
            self.assertEqual(binary, result_binary)
            self.assertEqual(doc['nodes'][:-1], original_doc['nodes'])
            self.assertEqual(doc['nodes'][-1]['translation'], [-10,-10,-2020])
            profile = json.loads((output / 'terrain.json').read_text())
            self.assertEqual(profile['schema'], 2)
            np.testing.assert_array_equal(profile['triangles'][0], [[-10,0,-20],[-10,1,80],[90,2,-20]])
            np.testing.assert_allclose(profile['solids'][0]['center'], [30,2,30])
            self.assertEqual(report['object_solids'], 1)
            self.assertEqual(report['water_exclusion_cells'], 1)  # masked second cell omitted
            np.testing.assert_allclose(profile['solids'][1]['size'], [25/6,1024,25/6])
            self.assertEqual(hashlib.sha256(visual.read_bytes()).hexdigest(), original_hash)
            with self.assertRaisesRegex(ValueError, 'must be new'):
                prepare(source, placements, liquids, visual, output, [10,10,2020])

    def test_nested_collision_follows_building_transform_and_scale(self):
        import csv
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = root / 'object.obj'
            model.write_text('v -1 0 -1\nv 1 0 -1\nv 1 4 1\nf 1 2 3\n')
            model.with_suffix('.json').write_text(json.dumps({'fileType': 'wmo'}))
            child = root / 'child.obj'
            child.write_text(model.read_text())
            child.with_suffix('.json').write_text(json.dumps({'fileType':'m2','collisionBox':{'min':[-1,-1,0],'max':[1,1,2]}}))
            parent = row(Type='wmo', ScaleFactor='2', PositionX=str(51200/3), PositionZ=str(51200/3), PositionY='0', RotationY='-90')
            placements = root / 'placements.csv'
            for path, records in [(placements, [parent]), (root / 'object_ModelPlacementInformation.csv',
                    [dict(row(ModelFile='child.obj', PositionX='10', PositionY='0', PositionZ='0'), DoodadSet='Set_A')])]:
                with path.open('w', newline='') as file:
                    writer = csv.DictWriter(file, records[0].keys(), delimiter=';')
                    writer.writeheader(); writer.writerows(records)
            solids, _, _ = collision_plan(placements, np.zeros(3))
            np.testing.assert_allclose(solids[1]['center'], [20,2,0], atol=1e-10)
            np.testing.assert_allclose(solids[1]['size'], [4,4,4], atol=1e-10)


if __name__ == '__main__':
    unittest.main()
