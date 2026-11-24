"""Relocate a saved population draft onto an imported heightfield without editing either source."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import struct


def triangle(grid, x, z):
    rows, cols = len(grid), len(grid[0])
    if not (math.isfinite(x) and math.isfinite(z) and 0 <= x <= cols-1 and 0 <= z <= rows-1):
        raise ValueError('Ground query outside heightfield')
    ix, iz = min(int(x), cols-2), min(int(z), rows-2)
    u, v = x-ix, z-iz
    a, b = grid[iz][ix:ix+2]
    c, d = grid[iz+1][ix:ix+2]
    return a+u*(b-a)+v*(c-a) if u+v <= 1 else d+(1-u)*(c-d)+(1-v)*(b-d)


class Ground:
    def __init__(self, zone):
        self.zone, self.cache = zone, {}
        if 'imported_terrain' in zone:
            if zone['imported_terrain'].get('journal'):
                raise ValueError('Publish edited terrain before using this relocation reader')
            self.root = Path(zone['imported_terrain']['package'])
            self.manifest = json.loads((self.root/'terrain.manifest.json').read_text())
        else:
            patch = zone['terrain']; axis = math.isqrt(len(patch['heights']))
            if axis*axis != len(patch['heights']):
                raise ValueError('Invalid source height patch')
            self.grid = [patch['heights'][i*axis:(i+1)*axis] for i in range(axis)]
            self.start = [p*128 for p in patch['chunk']]
            self.step = patch.get('span_chunks', 1)*128/(axis-1)

    def __call__(self, x, z):
        if not hasattr(self, 'manifest'):
            return self.zone['origin'][1] + triangle(self.grid, (x-self.start[0])/self.step, (z-self.start[1])/self.step)
        m = self.manifest; origin = m['chunk_origin']; counts = m['chunks_xz']; axis = m['samples_per_axis']
        relative = [x-origin[0]*128, z-origin[1]*128]
        if not all(math.isfinite(v) and 0 <= v <= n*128 for v, n in zip(relative, counts)):
            raise ValueError('Placement outside imported terrain')
        tile = [min(int(v//128), n-1) for v, n in zip(relative, counts)]
        key = tuple(tile)
        if key not in self.cache:
            ids = [tile[i]+origin[i] for i in range(2)]
            data = (self.root/f'tile_{ids[0]}_{ids[1]}.height').read_bytes()
            expected = bytes(m['tiles'][tile[1]*counts[0]+tile[0]])
            if hashlib.sha256(data).digest() != expected or data[:8] != b'THTILE01' or struct.unpack('<qq', data[8:24]) != tuple(ids):
                raise ValueError('Height tile identity mismatch')
            heights = struct.unpack('<'+'f'*(axis*axis), data[24:])
            self.cache[key] = [heights[i*axis:(i+1)*axis] for i in range(axis)]
        return triangle(self.cache[key], (relative[0]-tile[0]*128)*(axis-1)/128, (relative[1]-tile[1]*128)*(axis-1)/128)


COLLECTIONS = ('objects', 'scenery', 'paths', 'markers', 'plants', 'particles', 'test_players', 'spawn_groups', 'quest_areas')


def relocate(source, target, centre, old_ground, new_ground):
    if any(target.get(k) for k in COLLECTIONS):
        raise ValueError('Destination must be an empty terrain draft; existing placements are never replaced')
    offset = [centre[0]-source['origin'][0], centre[1]-source['origin'][2]]

    def position(p):
        x, z = p[0]+offset[0], p[2]+offset[1]
        return [x, new_ground(x, z)+p[1]-old_ground(p[0], p[2]), z]

    def bounds(lo, hi):
        # Keep the old vertical margins, shifted by the full range of terrain
        # displacement across the volume (conservatively including its rectangle).
        nx, nz = [max(1, math.ceil((hi[i]-lo[i])/2)) for i in (0, 2)]
        if (nx+1)*(nz+1) > 1000000:
            raise ValueError('Volume too large for bounded grounding check')
        delta = []
        for row in range(nz+1):
            for col in range(nx+1):
                x=lo[0]+(hi[0]-lo[0])*col/nx; z=lo[2]+(hi[2]-lo[2])*row/nz
                delta.append(new_ground(x+offset[0], z+offset[1])-old_ground(x, z))
        return ([lo[0]+offset[0],lo[1]+min(delta),lo[2]+offset[1]],
                [hi[0]+offset[0],hi[1]+max(delta),hi[2]+offset[1]])

    draft = copy.deepcopy(source)
    draft.pop('terrain', None)
    draft['imported_terrain'] = copy.deepcopy(target['imported_terrain'])
    draft['schema'] = max(16, target['schema'], source['schema'])
    draft['origin'] = target['origin'][:]
    draft['name'] = 'Millhaven — population village'
    draft['spawn'] = position(source['spawn'])
    for key in ('scenery','markers','plants','particles','test_players'):
        for row in draft.get(key, []):
            row['position'] = position(row['position'])
    for row in draft.get('objects', []):
        row['pose']['position'] = position(row['pose']['position'])
    for path in draft.get('paths', []):
        old = path['points']; dense = [old[0]]
        for a,b in zip(old,old[1:]):
            steps=max(1, math.ceil(math.hypot(b[0]-a[0], b[2]-a[2])/4))
            for step in range(1,steps+1):
                t=step/steps
                x,z=a[0]+(b[0]-a[0])*t,a[2]+(b[2]-a[2])*t
                clearance=(a[1]-old_ground(a[0],a[2]))*(1-t)+(b[1]-old_ground(b[0],b[2]))*t
                dense.append([x,old_ground(x,z)+clearance,z])
        if len(dense)>64:
            raise ValueError('Draped guide exceeds 64 points; split it explicitly')
        path['points'] = [position(p) for p in dense]
    for area in draft.get('quest_areas', []):
        area['min'],area['max']=bounds(area['min'],area['max'])
    for group in draft.get('spawn_groups', []):
        volume=group['volume']; polygon=volume['polygon']
        lo=[min(p[0] for p in polygon),volume['min_y'],min(p[1] for p in polygon)]
        hi=[max(p[0] for p in polygon),volume['max_y'],max(p[1] for p in polygon)]
        lo,hi=bounds(lo,hi)
        volume['polygon']=[[p[0]+offset[0],p[1]+offset[1]] for p in polygon]
        volume['min_y'],volume['max_y']=lo[1],hi[1]
    return draft, position


def write_json(path, value):
    path.write_text(json.dumps(value,indent=2,ensure_ascii=False,allow_nan=False)+'\n')


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source',type=Path,required=True);p.add_argument('--terrain-zone',type=Path,required=True)
    p.add_argument('--centre',type=float,nargs=2,required=True);p.add_argument('--gameplay',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    source=json.loads(a.source.read_text()); target=json.loads(a.terrain_zone.read_text())
    new_ground=Ground(target)
    result,position=relocate(source,target,a.centre,Ground(source),new_ground)
    manifest=json.loads(a.gameplay.read_text()); profiles={}
    for role,relative in manifest.items():
        if role in ('schema','id','revision'):continue
        path=(a.gameplay.parent/relative).resolve()
        if not path.is_relative_to(a.gameplay.parent.resolve()):raise ValueError('Profile escapes source directory')
        profiles[role]=json.loads(path.read_text())
    for reward in profiles['rewards']['rewards']:
        if 'ground' in reward:reward['ground']['position']=position(reward['ground']['position'])
    profiles['quest_areas']['areas']=[{k:v for k,v in row.items() if k not in ('id','definition')} | {'id':row['definition']} for row in result['quest_areas']]
    # The copied manifest is a staged authoring context, not a runtime export.
    a.output.mkdir(parents=True,exist_ok=False)
    gameplay=a.output/'gameplay';gameplay.mkdir()
    for role,profile in profiles.items():
        write_json(gameplay/(role+'.json'),profile);manifest[role]=role+'.json'
    manifest['id']='theia.dev.gameplay-millhaven-population'
    write_json(gameplay/'manifest.json',manifest);write_json(a.output/'zone.json',result)
    report={'source':str(a.source.resolve()),'source_sha256':hashlib.sha256(a.source.read_bytes()).hexdigest(),
            'terrain_zone':str(a.terrain_zone.resolve()),'terrain_zone_sha256':hashlib.sha256(a.terrain_zone.read_bytes()).hexdigest(),
            'centre_xz_m':a.centre,'terrain_unchanged':True,'source_terrain_used':False,
            'counts':{k:len(result.get(k,[])) for k in COLLECTIONS},'spawn':result['spawn'],
            'grounded_height_tiles_verified':len(new_ground.cache),'runtime_published':False}
    write_json(a.output/'placement-report.json',report)
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
