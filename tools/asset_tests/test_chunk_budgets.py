"""Verified asset costs and contributor unions before choosing runtime limits."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from measure_chunk_budgets import contributors, demand_patterns, footprint, measure, RADIUS
from partition_world import partition
from test_partition_world import make_source


class ChunkBudgetTests(unittest.TestCase):
    def test_independent_far_apart_actors_reach_finite_index_upper_bound(self):
        entries = [{'id': [x * 4, 0], 'bounds': [[0., 0.], [128., 128.]]} for x in range(6)]
        closure = contributors(entries)
        demand = demand_patterns(closure, {(0, 0)})
        for mode in ['current_footprints', 'falling_lookahead']:
            result = demand[mode]
            self.assertEqual(result['one_actor_packages'], 2)
            self.assertEqual(result['two_actor_packages'], 3)
            self.assertEqual(result['eight_actor_greedy_packages'], 6)
            self.assertTrue(result['eight_actor_maximum_proven'])
        with self.assertRaises(ValueError):
            contributors(entries + [entries[0]])

    def test_corner_footprint_requires_four_cells_and_crossing_geometry_provider(self):
        entries = [{'id': [x, z], 'bounds': [[0., 0.], [128., 128.]]} for x in range(2) for z in range(2)]
        entries.append({'id': [-1, 0], 'bounds': [[0., 0.], [300., 128.]]})
        closure = contributors(entries)
        cells, needed = footprint([128., 128.], closure)
        self.assertEqual(cells, {(0, 0), (0, 1), (1, 0), (1, 1)})
        self.assertEqual(len(needed), 5)
        self.assertEqual(footprint([128. - 2 * RADIUS, 64.], closure)[0], {(0, 0)})
        result = demand_patterns(closure, {(0, 0), (-1, 0)})
        self.assertEqual(result['current_footprints']['one_actor_packages'], 5)
        self.assertEqual(result['falling_lookahead']['one_actor_packages'], 5)

    def test_real_graph_costs_deduplicate_images_and_reject_changed_artifacts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source, terrain = make_source(root)
            output = root / 'chunks'
            partition(source, output, 'theia.test.budget', 1, [terrain])
            result = measure(output, source)
            self.assertTrue(result['spawn_provenance']['verified'])
            all_costs = result['all_indexed']
            self.assertEqual(all_costs['packages'], 3)
            self.assertEqual(all_costs['shared_image_variants'], 2)
            self.assertEqual(all_costs['shared_image_rgba8_base_estimate'], 32)
            self.assertEqual(result['recovery']['shared_image_variants'], 1)
            self.assertEqual(all_costs['visual_bytes'], sum(p['visual_bytes'] for p in result['packages']))
            self.assertEqual(all_costs['static_colliders'], 5)
            self.assertLess(max(result['proposed_control_sizes']['bytes'].values()), 1200)
            report_path = output / 'partition-report.json'
            report = json.loads(report_path.read_bytes())
            report['initial_required_packages'] = []
            report_path.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, 'source spawn footprints'):
                measure(output, source)
            report['initial_required_packages'] = [[0, 0]]
            report_path.write_text(json.dumps(report))
            index_path = output / 'world.chunks.candidate.json'
            index = json.loads(index_path.read_bytes())
            original_bounds = index['chunks'][0]['bounds']
            index['chunks'][0]['bounds'] = [[0., 0.], [128., 128.]]
            index_path.write_text(json.dumps(index))
            with self.assertRaisesRegex(ValueError, 'bounds disagree'):
                measure(output)
            index['chunks'][0]['bounds'] = original_bounds
            index_path.write_text(json.dumps(index))
            target = output / 'chunk_0_0/terrain.json'
            profile = json.loads(target.read_bytes())
            profile['triangles'][0][0][1] += .01
            target.write_text(json.dumps(profile))
            with self.assertRaisesRegex(ValueError, 'artifact binding mismatch'):
                measure(output)
