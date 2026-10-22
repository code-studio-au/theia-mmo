"""Ray-sample diagnostic GPU pixels against an independent mip-zero source oracle.

Uses actual captured camera matrices and exported geometry. Imports neither the
profile's authoring math nor shader source. Interior samples avoid silhouettes,
UI and triangle boundaries; limits cover float interpolation/sRGB8 conversion.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
from merge_static_glb import read
from static_geometry import accessor,hierarchy


def sample(image,uv,srgb):
    height,width=image.shape[:2];x=uv[0]*width-.5;y=(1-uv[1])*height-.5;ix=math.floor(x);iy=math.floor(y);fx=x-ix;fy=y-iy;result=np.zeros(4)
    for ox,oy,weight in [(0,0,(1-fx)*(1-fy)),(1,0,fx*(1-fy)),(0,1,(1-fx)*fy),(1,1,fx*fy)]:
        pixel=image[(iy+oy)%height,(ix+ox)%width].astype(float)/255
        if srgb:pixel[:3]=np.where(pixel[:3]<=.04045,pixel[:3]/12.92,((pixel[:3]+.055)/1.055)**2.4)
        result+=pixel*weight
    return result


def reference(images,p,n,falloff,tiling,mode):
    signs=[(1 if value>0 else -1 if value<0 else 0) for value in n];powers=[abs(float(v))**falloff for v in n];denominator=sum(powers)+.00001;weights=[powers[0]/denominator,max(0,powers[1]*signs[1]/denominator),max(0,-powers[1]*signs[1]/denominator),powers[2]/denominator]
    coordinates=[(p[2]*signs[0]*tiling,p[1]*tiling),(p[0]*signs[1]*tiling,p[2]*tiling),(p[0]*signs[1]*tiling,p[2]*tiling),(-p[0]*signs[2]*tiling,p[1]*tiling)]
    colour=np.zeros(4);normal=np.zeros(4)
    for role,uv,w in zip(['side','top','side','side'],coordinates,weights):
        colour+=sample(images[role],uv,True)*w;normal+=sample(images[role+'_normal'],uv,False)*w
    if mode==2:
        x=normal[0]*normal[3]*2-1;y=normal[1]*2-1;rgb=(np.array([x,y,math.sqrt(max(0,1-min(1,x*x+y*y)))])+1)*.5
    else:rgb=colour[:3]
    return np.rint(np.where(rgb<=.0031308,rgb*12.92,1.055*np.maximum(rgb,0)**(1/2.4)-.055)*255).clip(0,255)


def pixel_edge_clearance(triangle,clip,pixel,width,height):
    """Conservative screen-space interior, including MSAA's subpixel coverage.

    A barycentric fraction alone is insufficient for tiny/distant triangles.
    This test uses only geometry, never observed or expected pixel colours.
    """
    projected=np.c_[triangle,np.ones(3)]@clip.T
    if not np.isfinite(projected).all() or (projected[:,3]<=0).any():return 0.
    ndc=projected[:,:2]/projected[:,3,None]
    points=np.c_[(ndc[:,0]+1)*width*.5,(1-ndc[:,1])*height*.5]
    edges=np.roll(points,-1,axis=0)-points;lengths=np.linalg.norm(edges,axis=1)
    if (lengths<1e-8).any():return 0.
    delta=np.asarray(pixel)-points
    return float((np.abs(edges[:,0]*delta[:,1]-edges[:,1]*delta[:,0])/lengths).min())


def footprint_surface_consistent(triangles,e1,e2,inverse,x,y,width,height,index):
    """Exclude intersecting/occluding surfaces crossing the pixel footprint.

    A point can be far from its own triangle edges but still blend a different
    front surface under MSAA. Probe footprint corners using geometry alone.
    """
    for dx,dy in [(-.5,-.5),(.5,-.5),(-.5,.5),(.5,.5)]:
        ndc=[(x+.5+dx)/width*2-1,1-(y+.5+dy)/height*2]
        near=inverse@np.array([*ndc,1,1]);far=inverse@np.array([*ndc,.0001,1])
        origin=near[:3]/near[3];direction=far[:3]/far[3]-origin;direction/=np.linalg.norm(direction)
        h=np.cross(direction,e2);det=np.einsum('ij,ij->i',e1,h);safe=np.where(np.abs(det)>1e-12,det,1)
        v0=origin-triangles[:,0];u=np.einsum('ij,ij->i',v0,h)/safe;q=np.cross(v0,e1)
        v=q@direction/safe;t=np.einsum('ij,ij->i',e2,q)/safe
        valid=(det>1e-8)&(u>=0)&(v>=0)&(u+v<=1)&(t>0)
        if not valid.any() or int(np.argmin(np.where(valid,t,np.inf)))!=index:return False
    return True


def verify(stage,capture,required_profiles=None,sample_step=17,texture_origin=(0.,0.,0.)):
    if type(sample_step) is not int or not 8<=sample_step<=32:raise ValueError('Diagnostic sample step must be 8..32 pixels')
    texture_origin=np.asarray(texture_origin,dtype=float)
    if texture_origin.shape!=(3,) or not np.isfinite(texture_origin).all() or (abs(texture_origin)>1023).any():raise ValueError('Expected a bounded finite source texture origin')
    stage=Path(stage);capture=Path(capture);settings=json.loads((stage/'triplanar.json').read_text());report=json.loads((capture/'asset.json').read_text());mode=report['stats']['triplanar']['diagnostic_unlit']
    if mode not in (1,2):raise ValueError('Use an unlit colour/normal diagnostic capture')
    if settings['geometry_sha256']!=hashlib.sha256((stage/'preview.glb').read_bytes()).hexdigest():raise ValueError('Diagnostic geometry binding changed')
    if settings['schema']==1:profiles={1:settings}
    elif settings['schema']==2 and settings['profile']=='synty-triplanar-catalog-v1':
        profiles={p['id']:p for p in settings['profiles']}
        if len(profiles)!=len(settings['profiles']):raise ValueError('Duplicate diagnostic profile ID')
    else:raise ValueError('Unknown diagnostic metadata schema')
    required=set(profiles) if required_profiles is None else set(required_profiles)
    if not required or not required<=set(profiles):raise ValueError('Required diagnostic profiles are missing')
    profile_images={}
    for identifier,profile in profiles.items():
        images={}
        for role,info in profile['textures'].items():
            p=stage/info['file']
            if not p.resolve().is_relative_to(stage.resolve()) or hashlib.sha256(p.read_bytes()).hexdigest()!=info['sha256']:raise ValueError('Diagnostic source image changed')
            images[role]=np.array(Image.open(p).convert('RGBA'))
        profile_images[identifier]=images
    doc,data=read(stage/'preview.glb',allow_external_images=True);_,world=hierarchy(doc);triangles=[];normals=[];triangle_profiles=[]
    for node,frame in world.items():
        if 'mesh' not in doc['nodes'][node]:continue
        for p in doc['meshes'][doc['nodes'][node]['mesh']]['primitives']:
            extras=doc.get('materials',[])[p['material']].get('extras',{}) if 'material' in p else {}
            identifier=extras.get('theia_triplanar',0)
            if 'theia_triplanar' in extras and (type(identifier) is not int or identifier not in profiles):raise ValueError('Diagnostic material mark has no profile')
            ids=accessor(doc,data,p['indices']).reshape(-1,3);points=accessor(doc,data,p['attributes']['POSITION']).astype(float)@frame[:3,:3].T+frame[:3,3]
            if 'NORMAL' in p['attributes']:
                n=accessor(doc,data,p['attributes']['NORMAL']).astype(float)@np.linalg.inv(frame[:3,:3]);lengths=np.linalg.norm(n,axis=1)
                if identifier and (lengths<1e-12).any():raise ValueError('Diagnostic profile requires valid normals')
                n/=np.maximum(lengths,1e-12)[:,None]
            elif identifier:raise ValueError('Diagnostic profile requires source normals')
            else:n=np.zeros_like(points)
            triangles.extend(points[ids]);normals.extend(n[ids]);triangle_profiles.extend([identifier]*len(ids))
    triangles=np.array(triangles);normals=np.array(normals);e1=triangles[:,1]-triangles[:,0];e2=triangles[:,2]-triangles[:,0];clip=np.array(report['clip_from_world']).reshape((4,4),order='F');inverse=np.linalg.inv(clip)
    picture=np.array(Image.open(capture/'scene.png').convert('RGB'));height,width=picture.shape[:2];results=[]
    for y in range(135,height-30,sample_step):
        for x in range(130,width-130,sample_step+2):
            ndc=[(x+.5)/width*2-1,1-(y+.5)/height*2];near=inverse@np.array([*ndc,1,1]);far=inverse@np.array([*ndc,.0001,1]);origin=near[:3]/near[3];direction=far[:3]/far[3]-origin;direction/=np.linalg.norm(direction)
            h=np.cross(direction,e2);det=np.einsum('ij,ij->i',e1,h);safe=np.where(np.abs(det)>1e-12,det,1);v0=origin-triangles[:,0];u=np.einsum('ij,ij->i',v0,h)/safe;q=np.cross(v0,e1);v=q@direction/safe;t=np.einsum('ij,ij->i',e2,q)/safe;valid=(det>1e-8)&(u>=0)&(v>=0)&(u+v<=1)&(t>0)
            if not valid.any():continue
            index=int(np.argmin(np.where(valid,t,np.inf)))
            identifier=triangle_profiles[index]
            if not identifier:continue
            if min(u[index],v[index],1-u[index]-v[index])<.05:continue
            if pixel_edge_clearance(triangles[index],clip,[x+.5,y+.5],width,height)<1:continue
            if not footprint_surface_consistent(triangles,e1,e2,inverse,x,y,width,height,index):continue
            point=origin+direction*t[index];normal=normals[index,0]*(1-u[index]-v[index])+normals[index,1]*u[index]+normals[index,2]*v[index]
            point+=texture_origin
            point[0]*=-1;normal[0]*=-1
            profile=profiles[identifier]
            expected=reference(profile_images[identifier],point,normal,float(np.float32(profile['falloff'])),float(np.float32(profile['tiling'])),mode);actual=picture[y,x].astype(float);error=float(np.max(np.abs(expected-actual)));results.append({'pixel':[x,y],'profile_id':identifier,'expected':expected.tolist(),'actual':actual.tolist(),'error_u8':error})
    if len(results)<30:raise ValueError('Too few interior source pixels')
    counts={i:sum(r['profile_id']==i for r in results) for i in profiles}
    if any(counts[i]<30 for i in required):raise ValueError('Too few interior pixels for each required profile: '+str(counts))
    maximum=max(x['error_u8'] for x in results);bad=[r for r in results if r['error_u8']>4]
    if bad:raise ValueError('Diagnostic pixel error above four sRGB8 levels: '+json.dumps({'count':len(bad),'total':len(results),'max':maximum,'samples':bad[:5]}))
    return {'passed':True,'samples':len(results),'samples_by_profile':{str(i):n for i,n in counts.items()},'required_profiles':sorted(required),'sample_step':sample_step,'max_error_u8':maximum,'error_limit_u8':4,'minimum_edge_clearance_pixels':1,'consistent_front_surface_at_pixel_corners':True,'mode':mode,'texture_origin':texture_origin.tolist(),'results':results,'scope':'actual camera/source geometry and independent repeat-bilinear mip-zero colour or pre-TBN normal reconstruction; no Unity compression/BRDF/filtering parity'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('stage',type=Path);p.add_argument('capture',type=Path);p.add_argument('--report',required=True,type=Path);p.add_argument('--require-profile',type=int,action='append');p.add_argument('--sample-step',type=int,default=17)
    p.add_argument('--texture-origin',type=float,nargs=3,default=[0.,0.,0.])
    a=p.parse_args();r=verify(a.stage,a.capture,a.require_profile,a.sample_step,a.texture_origin);a.report.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k!='results'}))
