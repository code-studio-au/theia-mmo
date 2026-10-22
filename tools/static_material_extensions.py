"""Narrow static glTF material extensions with no additional index references."""
import math

SUPPORTED=frozenset({'KHR_materials_unlit','KHR_materials_emissive_strength'})


def validate(document):
    used=document.get('extensionsUsed',[]);required=document.get('extensionsRequired',[])
    for values in [used,required]:
        if not isinstance(values,list) or any(type(v) is not str or v not in SUPPORTED for v in values) or len(set(values))!=len(values):raise ValueError('Unsupported static material extension declaration')
    if not set(required)<=set(used):raise ValueError('Required material extension is undeclared')
    pending=[((),document)]
    while pending:
        path,value=pending.pop()
        if isinstance(value,dict):
            if 'extensions' in value:
                extensions=value['extensions']
                if not isinstance(extensions,dict):raise ValueError('Malformed static extension object')
                if extensions:
                    if len(path)!=2 or path[0]!='materials' or type(path[1]) is not int:raise ValueError('Only material-level static extensions are supported')
                    for name,fields in extensions.items():
                        if name not in used or name not in SUPPORTED:raise ValueError('Unsupported/undeclared static material extension payload')
                        if name=='KHR_materials_unlit':
                            if fields!={}:raise ValueError('Unsupported unlit material payload')
                        else:
                            if not isinstance(fields,dict) or set(fields)!={'emissiveStrength'}:raise ValueError('Unsupported emissive strength payload')
                            strength=fields['emissiveStrength']
                            if type(strength) not in (int,float) or not math.isfinite(strength) or not 0<=strength<=16:raise ValueError('Emissive strength exceeds bounded static profile')
            pending.extend((path+(key,),child) for key,child in value.items() if key!='extensions')
        elif isinstance(value,list):pending.extend((path+(index,),child) for index,child in enumerate(value))
