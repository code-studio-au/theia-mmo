//! Altered Rust adaptation of Recast 1.6.0's contour hole joining.
//! See ../RECAST-PROVENANCE.md and ../LICENSE-RECAST for origin and license.
//! Rerecast 0.5 builds contours but omits this step before polygon triangulation.
//! Preserve each hole by a visible doubled bridge within its own region/layer.
use nav_glam::U16Vec3;
use rerecast::{Contour, ContourSet};
use theia_network::Result;

type Vertex = (U16Vec3, u32);
const MAX_VERTICES: usize = 8192;
const MAX_CHECKS: usize = 8_000_000;

pub(super) fn join(set: &mut ContourSet) -> Result<()> {
    let count: usize = set.contours.iter().map(|c| c.vertices.len()).sum();
    if count > MAX_VERTICES {
        return Err("Navigation contour vertex budget exceeded".into());
    }
    let mut holes: Vec<_> = set
        .contours
        .iter()
        .enumerate()
        .filter(|(_, c)| area(&c.vertices) > 0)
        .map(|(i, _)| i)
        .collect();
    holes.sort_by_key(|&i| leftmost(&set.contours[i].vertices).1);
    let mut checks = 0;
    for hole_index in holes.iter().copied() {
        let hole = &set.contours[hole_index];
        let mut outlines = set.contours.iter().enumerate().filter(|(_, c)| {
            c.region == hole.region && c.area == hole.area && area(&c.vertices) < 0
        });
        let (outline_index, outline) = outlines
            .next()
            .ok_or("Navigation hole has no matching region outline")?;
        if outlines.next().is_some() {
            return Err("Navigation hole has ambiguous region outlines".into());
        }
        let start = leftmost(&hole.vertices).0;
        let mut bridge = None;
        for offset in 0..hole.vertices.len() {
            let h = (start + offset) % hole.vertices.len();
            let corner = hole.vertices[h].0;
            let mut candidates: Vec<_> = outline
                .vertices
                .iter()
                .enumerate()
                .filter(|(i, _)| in_cone(&outline.vertices, *i, corner))
                .map(|(i, (v, _))| {
                    let dx = i64::from(v.x) - i64::from(corner.x);
                    let dz = i64::from(v.z) - i64::from(corner.z);
                    (dx * dx + dz * dz, i)
                })
                .collect();
            candidates.sort_unstable();
            for (_, o) in candidates {
                let point = outline.vertices[o].0;
                if intersects(point, corner, outline, &mut checks)? {
                    continue;
                }
                let mut blocked = false;
                for &other in &holes {
                    let other = &set.contours[other];
                    if other.region == hole.region
                        && other.area == hole.area
                        && intersects(point, corner, other, &mut checks)?
                    {
                        blocked = true;
                        break;
                    }
                }
                if !blocked {
                    bridge = Some((o, h));
                    break;
                }
            }
            if bridge.is_some() {
                break;
            }
        }
        let (o, h) = bridge.ok_or("Navigation hole has no visible bridge")?;
        if set.contours.iter().map(|c| c.vertices.len()).sum::<usize>() + 2 > MAX_VERTICES {
            return Err("Navigation hole bridge exceeds vertex budget".into());
        }
        let joined = (0..=outline.vertices.len())
            .map(|i| outline.vertices[(o + i) % outline.vertices.len()])
            .chain((0..=hole.vertices.len()).map(|i| hole.vertices[(h + i) % hole.vertices.len()]))
            .collect();
        set.contours[outline_index].vertices = joined;
        set.contours[hole_index].vertices.clear();
    }
    Ok(())
}

fn leftmost(vertices: &[Vertex]) -> (usize, (u16, u16)) {
    vertices
        .iter()
        .enumerate()
        .map(|(i, (v, _))| (i, (v.x, v.z)))
        .min_by_key(|(_, p)| *p)
        .expect("Nonempty hole")
}
fn area(vertices: &[Vertex]) -> i64 {
    (0..vertices.len())
        .map(|i| {
            let a = vertices[i].0;
            let b = vertices[(i + 1) % vertices.len()].0;
            i64::from(a.x) * i64::from(b.z) - i64::from(b.x) * i64::from(a.z)
        })
        .sum()
}
fn cross(a: U16Vec3, b: U16Vec3, c: U16Vec3) -> i64 {
    (i64::from(b.x) - i64::from(a.x)) * (i64::from(c.z) - i64::from(a.z))
        - (i64::from(c.x) - i64::from(a.x)) * (i64::from(b.z) - i64::from(a.z))
}
fn in_cone(vertices: &[Vertex], i: usize, p: U16Vec3) -> bool {
    let a = vertices[i].0;
    let next = vertices[(i + 1) % vertices.len()].0;
    let prev = vertices[(i + vertices.len() - 1) % vertices.len()].0;
    if cross(prev, a, next) <= 0 {
        cross(a, p, prev) < 0 && cross(p, a, next) < 0
    } else {
        !(cross(a, p, next) <= 0 && cross(p, a, prev) <= 0)
    }
}
fn equal(a: U16Vec3, b: U16Vec3) -> bool {
    a.x == b.x && a.z == b.z
}
fn between(a: U16Vec3, b: U16Vec3, p: U16Vec3) -> bool {
    cross(a, b, p) == 0
        && (a.x.min(b.x)..=a.x.max(b.x)).contains(&p.x)
        && (a.z.min(b.z)..=a.z.max(b.z)).contains(&p.z)
}
fn intersects(a: U16Vec3, b: U16Vec3, contour: &Contour, checks: &mut usize) -> Result<bool> {
    for i in 0..contour.vertices.len() {
        *checks += 1;
        if *checks > MAX_CHECKS {
            return Err("Navigation hole visibility budget exceeded".into());
        }
        let c = contour.vertices[i].0;
        let d = contour.vertices[(i + 1) % contour.vertices.len()].0;
        if equal(a, c) || equal(a, d) || equal(b, c) || equal(b, d) {
            continue;
        }
        let ac = cross(a, b, c);
        let ad = cross(a, b, d);
        let ca = cross(c, d, a);
        let cb = cross(c, d, b);
        if (ac.signum() != ad.signum()
            && ca.signum() != cb.signum()
            && ac != 0
            && ad != 0
            && ca != 0
            && cb != 0)
            || between(a, b, c)
            || between(a, b, d)
            || between(c, d, a)
            || between(c, d, b)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rerecast::{AreaType, RegionId};

    fn contour(points: &[[u16; 2]], region: u16) -> Contour {
        Contour {
            vertices: points
                .iter()
                .map(|&[x, z]| (U16Vec3::new(x, 0, z), 0))
                .collect(),
            region: RegionId::from_bits_retain(region),
            area: AreaType::DEFAULT_WALKABLE,
            ..Default::default()
        }
    }
    fn outline() -> Contour {
        contour(&[[0, 0], [0, 40], [40, 40], [40, 0]], 1)
    }
    fn hole(x: u16) -> Contour {
        contour(&[[x, 10], [x + 4, 10], [x + 4, 14], [x, 14]], 1)
    }

    #[test]
    fn two_holes_preserve_excluded_area_through_actual_backend_triangulation() {
        // 640-cell outlines exceed u16 squared-distance range. Preserve both
        // holes and exact area at actual 128m / 0.2m tile scale as well.
        for scale in [1_u16, 16] {
            let mut set = ContourSet {
                contours: vec![hole(22), outline(), hole(10)],
                ..Default::default()
            };
            for contour in &mut set.contours {
                for (v, _) in &mut contour.vertices {
                    v.x *= scale;
                    v.z *= scale;
                }
            }
            assert!(set.clone().into_polygon_mesh(6).is_err());
            join(&mut set).unwrap();
            assert_eq!(
                set.contours
                    .iter()
                    .filter(|c| !c.vertices.is_empty())
                    .count(),
                1
            );
            let mesh = set.into_polygon_mesh(6).unwrap();
            let mut twice_area = 0;
            for polygon in mesh.polygons() {
                let v: Vec<_> = polygon
                    .map(|i| (mesh.vertices[usize::from(i)], 0))
                    .collect();
                twice_area += area(&v).abs();
                for x in [12, 24] {
                    let center = U16Vec3::new(x * scale, 0, 12 * scale);
                    assert!(
                        (0..v.len()).any(|i| cross(v[i].0, v[(i + 1) % v.len()].0, center) > 0),
                        "Obstacle hole was filled by a walkable polygon"
                    );
                }
            }
            assert_eq!(
                twice_area,
                2 * (40 * 40 - 2 * 4 * 4) * i64::from(scale).pow(2)
            );
        }
    }

    #[test]
    fn hole_cannot_bridge_to_another_region_or_ambiguous_outline() {
        let mut set = ContourSet {
            contours: vec![outline(), hole(10)],
            ..Default::default()
        };
        set.contours[1].region = RegionId::from_bits_retain(2);
        assert!(join(&mut set).is_err());
        set.contours[1].region = RegionId::from_bits_retain(1);
        set.contours.push(outline());
        assert!(join(&mut set).is_err());
    }

    #[test]
    fn exterior_hole_and_excessive_work_refuse_instead_of_filling_or_hanging() {
        let mut set = ContourSet {
            contours: vec![outline(), hole(50)],
            ..Default::default()
        };
        assert!(join(&mut set).is_err());
        let mut checks = MAX_CHECKS;
        assert!(
            intersects(
                U16Vec3::new(0, 0, 20),
                U16Vec3::new(40, 0, 20),
                &hole(10),
                &mut checks
            )
            .is_err()
        );
        let mut set = ContourSet {
            contours: vec![Contour {
                vertices: vec![(U16Vec3::ZERO, 0); MAX_VERTICES + 1],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(join(&mut set).is_err());
    }
}
