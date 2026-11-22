import copy
import unittest
from prepare_population_on_terrain import relocate, triangle


class RelocationTests(unittest.TestCase):
    def test_triangle_diagonal_is_not_bilinear(self):
        self.assertEqual(triangle([[0., 0.], [0., 10.]], .25, .25), 0.)
        self.assertEqual(triangle([[0., 0.], [0., 10.]], .75, .75), 5.)
        with self.assertRaises(ValueError): triangle([[0., 0.], [0., 10.]], -1., 0.)

    def test_all_bindings_relative_placement_ground_offsets_and_volumes_survive(self):
        source={'schema':14,'origin':[100.,0.,100.],'spawn':[100.,.1,100.], 'terrain':{},
            'scenery':[{'id':1,'asset':'hut','position':[104.,-2.,105.],'yaw_degrees':90.,'scale':1.4}],
            'markers':[{'id':2,'position':[102.,.03,100.],'definition':'npc'}],
            'objects':[{'id':3,'pose':{'position':[104.,1.,100.],'size':[2.,2.,2.]}}],
            'paths':[{'id':4,'kind':'road','points':[[100.,0.,100.],[100.,0.,112.]]}],
            'spawn_groups':[{'id':5,'creature':'wolf','target_alive':5,'seed':42,'patrol_path':4,
                'volume':{'polygon':[[100.,100.],[104.,100.],[104.,104.]],'min_y':-2.,'max_y':8.}}],
            'quest_areas':[{'id':6,'definition':'area','min':[100.,-16.,100.],'max':[104.,16.,104.],'map_objectives':['quest']}],
            'particles':[{'id':7,'effect':'smoke','position':[100.,3.,100.]}],
            'test_players':[{'id':8,'position':[100.,.1,100.],'controller':'walk'}]}
        target={'schema':16,'origin':[1024.,0.,1024.],'imported_terrain':{'package':'checked-source','identity':[1]}}
        before=copy.deepcopy(source);terrain_before=copy.deepcopy(target)
        ground=lambda x,z:50.+(x-1000.)*.5+(z-1000.)*.25
        d,place=relocate(source,target,[1000.,1000.],lambda x,z:0.,ground)
        self.assertEqual(source,before);self.assertEqual(target,terrain_before)
        self.assertNotIn('terrain',d);self.assertEqual(d['imported_terrain'],target['imported_terrain'])
        self.assertEqual(d['spawn'],[1000.,50.1,1000.])
        self.assertEqual(d['scenery'][0]['position'],[1004.,51.25,1005.])
        self.assertEqual(d['scenery'][0]['yaw_degrees'],90.)
        self.assertEqual(d['markers'][0]['definition'],'npc')
        self.assertEqual(d['objects'][0]['pose']['position'],[1004.,53.,1000.])
        self.assertEqual(len(d['paths'][0]['points']),4)
        self.assertEqual(d['paths'][0]['points'][-1],[1000.,53.,1012.])
        g=d['spawn_groups'][0];self.assertEqual(g['target_alive'],5);self.assertEqual(g['patrol_path'],4)
        self.assertEqual(g['volume']['min_y'],48.);self.assertEqual(g['volume']['max_y'],61.)
        self.assertEqual(d['quest_areas'][0]['map_objectives'],['quest'])
        self.assertEqual(d['particles'][0]['position'],[1000.,53.,1000.])
        self.assertEqual(place([100.,0.,112.]),[1000.,53.,1012.])
        target['scenery']=[{'id':99}]
        with self.assertRaises(ValueError):relocate(source,target,[1000.,1000.],lambda x,z:0.,ground)

if __name__=='__main__':unittest.main()
