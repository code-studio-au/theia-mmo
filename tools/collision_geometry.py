"""Independent Euclidean capsule/triangle checks for offline qualification."""
import math


def sub(a, b):
    return tuple(x-y for x,y in zip(a,b))


def dot(a, b):
    return sum(x*y for x,y in zip(a,b))


def at(a, d, t):
    return tuple(x+t*y for x,y in zip(a,d))


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def point_triangle_sq(p, a, b, c):
    ab,ac,ap=sub(b,a),sub(c,a),sub(p,a)
    d1,d2=dot(ab,ap),dot(ac,ap)
    if d1<=0 and d2<=0: return dot(ap,ap)
    bp=sub(p,b);d3,d4=dot(ab,bp),dot(ac,bp)
    if d3>=0 and d4<=d3: return dot(bp,bp)
    vc=d1*d4-d3*d2
    if vc<=0 and d1>=0 and d3<=0:
        q=sub(p,at(a,ab,d1/(d1-d3)));return dot(q,q)
    cp=sub(p,c);d5,d6=dot(ab,cp),dot(ac,cp)
    if d6>=0 and d5<=d6:return dot(cp,cp)
    vb=d5*d2-d1*d6
    if vb<=0 and d2>=0 and d6<=0:
        q=sub(p,at(a,ac,d2/(d2-d6)));return dot(q,q)
    va=d3*d6-d5*d4
    if va<=0 and d4-d3>=0 and d5-d6>=0:
        q=sub(p,at(b,sub(c,b),(d4-d3)/((d4-d3)+(d5-d6))));return dot(q,q)
    denom=1/(va+vb+vc)
    q=sub(p,at(at(a,ab,vb*denom),ac,vc*denom));return dot(q,q)


def segment_segment_sq(p, q, a, b):
    d,e,r=sub(q,p),sub(b,a),sub(p,a)
    dd,ee,de,dr,er=dot(d,d),dot(e,e),dot(d,e),dot(d,r),dot(e,r)
    if dd<1e-20 and ee<1e-20:return dot(r,r)
    if dd<1e-20:s,t=0,min(1,max(0,er/ee))
    elif ee<1e-20:s,t=min(1,max(0,-dr/dd)),0
    else:
        denom=dd*ee-de*de
        s=min(1,max(0,(de*er-dr*ee)/denom)) if denom>1e-20 else 0
        t=(de*s+er)/ee
        if t<0:s,t=min(1,max(0,-dr/dd)),0
        elif t>1:s,t=min(1,max(0,(de-dr)/dd)),1
    gap=sub(at(p,d,s),at(a,e,t));return dot(gap,gap)


def segment_triangle_sq(p, q, a, b, c):
    normal=cross(sub(b,a),sub(c,a));d=sub(q,p);denom=dot(normal,d)
    if dot(normal, normal) <= 1e-20:
        return min(segment_segment_sq(p,q,x,y) for x,y in [(a,b),(b,c),(c,a)])
    if abs(denom)>1e-20:
        t=dot(normal,sub(a,p))/denom
        if 0<=t<=1 and point_triangle_sq(at(p,d,t),a,b,c)<1e-16:return 0
    return min(point_triangle_sq(p,a,b,c), point_triangle_sq(q,a,b,c),
               *(segment_segment_sq(p,q,x,y) for x,y in [(a,b),(b,c),(c,a)]))


def mesh_capsule_clearance(triangles, player):
    """Actual 0.35 m radius / 1.8 m tall capsule; no highest-floor assumption."""
    x,z=player['xz'];feet=player['y']-.9
    p,q=(x,feet+.35,z),(x,feet+1.45,z)
    nearest=math.inf
    # Exact distance only for candidates near the footprint. Filtering preserves
    # every triangle that could contact within the accepted 2 cm ground gap.
    for triangle in triangles:
        if any(max(v[i] for v in triangle)<coordinate-.4 or
               min(v[i] for v in triangle)>coordinate+.4 for i,coordinate in [(0,x),(2,z)]):continue
        nearest=min(nearest,segment_triangle_sq(p,q,*triangle))
    return math.sqrt(nearest)-.35
