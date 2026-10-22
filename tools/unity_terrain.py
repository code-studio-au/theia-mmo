"""Read a bounded, hash-bound Unity 2020 static TerrainData profile offline.

No Unity scripts or external binary dependencies are loaded. Normal maps and
surface-layer data are retained as source bindings, not silently interpreted.
"""
import importlib.metadata
import math
import numpy as np
from unity_scene import Sources,records

HEIGHT_MAX=32766
UNITYPY_VERSION='1.25.0'


def serialized_guid(value):
    if not isinstance(value,bytes) or len(value)!=16:raise ValueError('Expected serialized 16-byte Unity GUID')
    return ''.join(f'{byte&15:x}{byte>>4:x}' for byte in value)


def height_grid(value):
    resolution=value['m_Resolution']
    if type(resolution) is not int or resolution not in (33,65,129,257,513,1025):
        raise ValueError('Unsupported bounded heightmap resolution')
    heights=np.asarray(value['m_Heights'])
    if heights.shape!=(resolution*resolution,) or not np.issubdtype(heights.dtype,np.integer) or heights.min()<0 or heights.max()>HEIGHT_MAX:
        raise ValueError('Invalid normalized heightmap samples')
    if value.get('m_Holes') or value.get('m_HolesLOD'):raise ValueError('Terrain holes require a separate supported profile')
    scale=np.array([value['m_Scale'][axis] for axis in 'xyz'],dtype=float)
    if not np.isfinite(scale).all() or (scale<=0).any() or (scale>4096).any():raise ValueError('Invalid terrain sample scale')
    # Require the serialized independent patch extrema to establish the height
    # normalization and row-major sample convention for this source profile.
    raw=heights.reshape(resolution,resolution);normalized=raw.astype(float)/HEIGHT_MAX
    expected=[]
    for step_power in range(4,int(math.log2(resolution-1))+1):
        step=2**step_power
        for z in range(0,resolution-1,step):
            for x in range(0,resolution-1,step):
                patch=normalized[z:z+step+1,x:x+step+1]
                expected.extend([float(patch.min()),float(patch.max())])
    reference=np.asarray(value['m_MinMaxPatchHeights'],dtype=float)
    if reference.shape!=(len(expected),) or not np.isfinite(reference).all():raise ValueError('Missing bounded terrain patch extrema')
    error=float(np.max(np.abs(reference-expected)))
    if error>3e-8:raise ValueError('Terrain patch extrema disagree with decoded heightmap')
    return raw.astype('<u2'),scale,error


def alpha_weights(textures,layer_count,resolution):
    if type(layer_count) is not int or not 1<=layer_count<=8 or type(resolution) is not int or resolution not in (32,64,128,256,512,1024):
        raise ValueError('Unsupported bounded terrain layer/alpha resolution')
    if len(textures)!=(layer_count+3)//4:raise ValueError('Terrain control texture count differs from layers')
    arrays=[]
    for texture in textures:
        if (texture['m_Width'],texture['m_Height'],texture['m_TextureFormat'],texture['m_ImageCount'],texture['m_TextureDimension'],texture['m_ColorSpace'])!=(resolution,resolution,4,1,2,0):
            raise ValueError('Expected linear embedded RGBA32 terrain control texture')
        if texture['m_StreamData']['size'] or texture['m_StreamData']['path']:raise ValueError('External terrain texture streams unsupported')
        if texture['m_MipCount']!=int(math.log2(resolution))+1 or texture['m_MipsStripped']!=0:raise ValueError('Unsupported terrain control mip layout')
        count=sum(max(1,resolution>>level)**2*4 for level in range(texture['m_MipCount']))
        if len(texture['image data'])!=count or texture['m_CompleteImageSize']!=count:raise ValueError('Truncated terrain control pixels')
        settings=texture['m_TextureSettings']
        if settings['m_FilterMode']!=1 or settings['m_WrapU']!=1 or settings['m_WrapV']!=1:raise ValueError('Unsupported terrain control sampler')
        arrays.append(np.frombuffer(texture['image data'][:resolution*resolution*4],dtype=np.uint8).reshape(resolution,resolution,4))
    weights=np.concatenate(arrays,axis=2)
    if weights[:,:,layer_count:].any():raise ValueError('Nonzero control weights address missing terrain layers')
    weights=weights[:,:,:layer_count].copy()
    if (weights.astype(np.uint16).sum(2)==0).any():raise ValueError('Terrain control pixel has no assigned layer')
    return weights


def read_terrain(root,scene):
    if importlib.metadata.version('UnityPy')!=UNITYPY_VERSION:raise ValueError('Use the pinned offline UnityPy version')
    import UnityPy
    sources=Sources(root);rr=records(sources.checked(scene));sources.metadata(scene)
    terrains=[(i,v['fields']) for i,v in rr.items() if v['kind']==218]
    if len(terrains)!=1:raise ValueError('Select a scene with one static terrain')
    terrain_id,terrain=terrains[0];go_id=terrain['m_GameObject']['fileID'];go=rr[go_id]
    if terrain['m_Enabled']!=1 or terrain['m_DrawHeightmap']!=1 or go['kind']!=1 or go['fields']['m_IsActive']!=1:
        raise ValueError('Expected enabled visible terrain')
    colliders=[v['fields'] for v in rr.values() if v['kind']==154 and v['fields']['m_GameObject']['fileID']==go_id]
    if len(colliders)!=1 or colliders[0]['m_Enabled']!=1 or colliders[0]['m_TerrainData']!=terrain['m_TerrainData']:
        raise ValueError('Visible terrain lacks matching enabled source TerrainCollider')
    transforms=[v['fields'] for v in rr.values() if v['kind']==4 and not v['stripped'] and v['fields']['m_GameObject']['fileID']==go_id]
    if len(transforms)!=1:raise ValueError('Expected one ordinary terrain transform')
    transform=transforms[0]
    if transform['m_Father']['fileID'] or transform['m_LocalRotation']!={'x':0,'y':0,'z':0,'w':1} or transform['m_LocalScale']!={'x':1,'y':1,'z':1}:
        raise ValueError('Only unrotated unscaled scene-root terrain is qualified')
    origin=np.array([transform['m_LocalPosition'][axis] for axis in 'xyz'],dtype=float)
    if not np.isfinite(origin).all() or np.abs(origin).max()>4096:raise ValueError('Terrain origin outside local frame')
    ref=terrain['m_TerrainData'];name=sources.by_guid.get(ref['guid']);path=sources.checked(name);sources.metadata(name)
    if path.stat().st_size>64*1024*1024:raise ValueError('TerrainData exceeds binary byte budget')
    environment=UnityPy.load(path.read_bytes());objects={obj.path_id:obj for obj in environment.objects}
    if len(objects)>32 or ref['fileID'] not in objects:raise ValueError('Unresolved bounded TerrainData object')
    obj=objects[ref['fileID']]
    if obj.type.name!='TerrainData' or str(obj.assets_file.unity_version)!='2020.3.40f1':raise ValueError('Unqualified binary TerrainData version/type')
    data=obj.read_typetree();heights,scale,error=height_grid(data['m_Heightmap'])
    detail=data['m_DetailDatabase']
    if detail['m_TreeInstances'] or any(p['layerIndices'] or p['numberOfObjects'] for p in detail['m_Patches']):
        raise ValueError('Populated terrain trees/details require explicit instance import')
    layers=[]
    for ref in data['m_SplatDatabase']['m_TerrainLayers']:
        file_id=ref['m_FileID']
        if not 1<=file_id<=len(obj.assets_file.externals):raise ValueError('Invalid external terrain layer index')
        external=obj.assets_file.externals[file_id-1];guid=serialized_guid(external.guid);layer_name=sources.by_guid.get(guid)
        layer=records(sources.checked(layer_name)).get(ref['m_PathID']);sources.metadata(layer_name)
        if layer is None or layer['kind']!=1953259897:raise ValueError('Terrain layer GUID/fileID/type mismatch')
        fields=layer['fields'];textures={}
        for key in ['m_DiffuseTexture','m_NormalMapTexture','m_MaskMapTexture']:
            texture=fields[key]
            if texture['fileID']:
                texture_name=sources.by_guid.get(texture.get('guid'));sources.checked(texture_name);sources.metadata(texture_name);textures[key]=texture_name
        if 'm_DiffuseTexture' not in textures:raise ValueError('Terrain layer requires source diffuse pixels')
        layers.append({'source':layer_name,'guid':guid,'fileID':ref['m_PathID'],'fields':fields,'textures':textures})
    textures=[]
    for ref in data['m_SplatDatabase']['m_AlphaTextures']:
        if ref['m_FileID']!=0 or ref['m_PathID'] not in objects or objects[ref['m_PathID']].type.name!='Texture2D':
            raise ValueError('Terrain controls must be embedded Texture2D objects')
        textures.append(objects[ref['m_PathID']].read_typetree())
    weights=alpha_weights(textures,len(layers),data['m_SplatDatabase']['m_AlphamapResolution'])
    extent=(heights.shape[0]-1)*scale[[0,2]]
    bounds=np.array([[origin[0],origin[1]+heights.min()/HEIGHT_MAX*scale[1],origin[2]],
                     [origin[0]+extent[0],origin[1]+heights.max()/HEIGHT_MAX*scale[1],origin[2]+extent[1]]])
    if np.abs(bounds).max()>4096:raise ValueError('Terrain geometry exceeds local frame')
    bindings={name+('.meta' if meta else ''):sources.manifest['metadata' if meta else 'selected'][name]['sha256'] for name,meta in sorted(sources.verified)}
    return {'sources':sources,'scene':scene,'terrain':name,'terrain_id':terrain_id,'terrain_data_id':obj.path_id,
            'origin':origin,'scale':scale,'heights':heights,'weights':weights,'layers':layers,
            'patch_extrema_error':error,'source_bindings':bindings,'material_template':terrain['m_MaterialTemplate']}
