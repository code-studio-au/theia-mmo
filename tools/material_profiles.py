"""Strict data-only triplanar parameters/PNG paths bound inside material extras."""
import math
import re
import struct

EXTRA = 'theia_triplanar_profile'
SAMPLER = [10497,10497,9729,9729]
ROLES = [('top','srgb'),('side','srgb'),('top_normal','linear'),('side_normal','linear')]
URI = re.compile(r'\.\./shared/[0-9a-f]{64}-([sl])-10497-10497-9729-9729\.png')


def number(value,limit):
    if type(value) not in (int,float) or not math.isfinite(value) or abs(value)>limit:
        raise ValueError('Invalid bound material numeric parameter')
    return float(value)


def parse(extras):
    if not isinstance(extras,dict) or EXTRA not in extras:return None
    value=extras[EXTRA]
    if not isinstance(value,dict) or set(value)!={'schema','profile','id','tiling','falloff','source_anchor','textures'}:
        raise ValueError('Unknown or missing bound material profile fields')
    if type(value['schema']) is not int or value['schema']!=1 or value['profile']!='synty-triplanar-world-v1' or type(value['id']) is not int or not 1<=value['id']<=16 or type(extras.get('theia_triplanar')) is not int or extras['theia_triplanar']!=value['id']:
        raise ValueError('Invalid bound material profile/version/mark')
    result=dict(value)
    for key,limit in [('tiling',1000),('falloff',100)]:
        scalar=struct.unpack('<f',struct.pack('<f',number(value[key],limit)))[0]
        if scalar<=0:raise ValueError('Expected positive finite f32 material parameter')
        result[key]=scalar
    anchor=value['source_anchor']
    if not isinstance(anchor,list) or len(anchor)!=3:raise ValueError('Invalid global source anchor')
    result['source_anchor']=[number(v,1e12) or 0. for v in anchor]
    textures=value['textures']
    if not isinstance(textures,dict) or set(textures)!={key for key,_ in ROLES}:raise ValueError('Invalid bound material image roles')
    for key,color in ROLES:
        uri=textures[key]
        match=URI.fullmatch(uri) if isinstance(uri,str) else None
        if not match or match[1]!=('s' if color=='srgb' else 'l'):raise ValueError('Noncanonical projected material image URI/role/sampler')
    return result


def profiles(document):
    result=[];keys=set()
    for material in document.get('materials',[]):
        profile=parse(material.get('extras'))
        if profile is None:continue
        key=(profile['tiling'],profile['falloff'],tuple(profile['source_anchor']),tuple(profile['textures'][role] for role,_ in ROLES))
        keys.add(key)
        if len(keys)>16:raise ValueError('Bound material profile budget exceeded')
        result.append(profile)
    return result


def image_uses(document):
    return [(profile['textures'][role],color) for profile in profiles(document) for role,color in ROLES]
