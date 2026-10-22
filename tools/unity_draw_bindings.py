"""Bind one planned ordinary-prefab renderer to actual scene overrides.

Canonical geometry is independent of the source world pose. The complete plan's
world matrices require the independent scene oracle before batch publication.
"""
from functools import lru_cache
import hashlib
import json
from pathlib import Path
from unity_scene import records
from unity_prefabs import static_prefab
from unity_prefab_ids import compile_single_model_variant


@lru_cache(maxsize=128)
def source_records(path,digest):
    path=Path(path)
    if hashlib.sha256(path.read_bytes()).hexdigest()!=digest:raise ValueError('Source document changed before binding')
    return records(path)


def checked_records(sources,name):
    path=sources.checked(name);sources.metadata(name)
    return source_records(str(path),sources.manifest['selected'][name]['sha256'])


def select_draw(sources,plan_path,instance,renderer,prefab,*,allow_model_variant=False):
    plan_path=Path(plan_path).resolve(strict=True)
    if not 1<=plan_path.stat().st_size<=64*1024*1024:raise ValueError('Draw plan exceeds byte budget')
    data=plan_path.read_bytes();plan=json.loads(data)
    if plan['archive_sha256']!=sources.manifest['archive_sha256'] or plan.get('lod_policy')!='highest-detail first serialized LOD; no runtime crossfade' or not 1<=len(plan['draws'])<=65536:
        raise ValueError('Select the bound highest-detail source plan')
    matches=[d for d in plan['draws'] if d['instance']==instance and d['renderer']==renderer]
    if len(matches)!=1:raise ValueError('Planned draw must have a unique source instance/renderer')
    planned,=matches
    if planned['source']!=prefab:raise ValueError('Selected draw prefab differs')
    scene=checked_records(sources,plan['scene']);rr=checked_records(sources,prefab)
    variant=any(v['kind']==1001 for v in rr.values())
    if instance is None or (variant and not allow_model_variant) or (not variant and any(v['stripped'] for v in rr.values())):raise ValueError('Inline/model-variant binding requires separate instantiation')
    source=scene.get(instance)
    if source is None or source['kind']!=1001:raise ValueError('Missing source PrefabInstance')
    fields=source['fields'];guid=sources.manifest['selected'][prefab]['guid']
    if fields['m_SourcePrefab'].get('guid')!=guid:raise ValueError('Source instance/prefab GUID differs')
    modification=fields['m_Modification']
    if any(v for k,v in modification.items() if k.startswith(('m_Removed','m_Added'))):raise ValueError('Structural scene override unsupported')
    if variant:rr=compile_single_model_variant(sources,prefab,modification['m_Modifications'],guid)
    static=static_prefab(rr,modification['m_Modifications'],guid,lod_level=0)
    matches=[d for d in static['draws'] if d['renderer']==renderer]
    if len(matches)!=1:raise ValueError('Source renderer is inactive or outside highest LOD')
    draw,=matches
    if any(draw[key]!=planned[key] for key in ['mesh','materials','transform']):raise ValueError('Planned draw mesh/material/transform differs from source overrides')
    binding={'plan':str(plan_path),'plan_sha256':hashlib.sha256(data).hexdigest(),'scene':plan['scene'],
             'scene_sha256':sources.manifest['selected'][plan['scene']]['sha256'],'instance':instance,'renderer':renderer,
             'world_matrix':planned['matrix']}
    if variant:binding['source_kind']='single-model-variant'
    return draw,static,binding
