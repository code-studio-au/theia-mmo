//! Checked global map coordinates, independent of floating render origins.
use theia_core::coordinates::{ChunkId, GlobalPosition};

pub(super) const RADIUS: f64 = 128.;
pub(super) const PIXELS: usize = 64;
pub(super) const DISPLAY: f32 = 176.;

#[derive(Clone, Copy)]
pub(super) struct View {
    pub centre: GlobalPosition,
}
impl View {
    pub fn project(self, point: GlobalPosition) -> Option<[f32; 2]> {
        let a = self.centre.metres();
        let b = point.metres();
        let dx = b[0] - a[0];
        let dz = b[2] - a[2];
        if dx.abs() > RADIUS || dz.abs() > RADIUS {
            return None;
        }
        Some([
            ((dx / RADIUS + 1.) * f64::from(DISPLAY) * 0.5) as f32,
            ((dz / RADIUS + 1.) * f64::from(DISPLAY) * 0.5) as f32,
        ])
    }

    pub fn pixel(self, x: usize, z: usize) -> Option<GlobalPosition> {
        if x >= PIXELS || z >= PIXELS {
            return None;
        }
        let a = self.centre.metres();
        let offset = |v: usize| ((v as f64 + 0.5) / PIXELS as f64 * 2. - 1.) * RADIUS;
        GlobalPosition::new([a[0] + offset(x), 0., a[2] + offset(z)])
    }
}

pub(super) struct HeightTile<'a> {
    pub id: ChunkId,
    pub cells: usize,
    pub positions: &'a [[f32; 3]],
}
impl HeightTile<'_> {
    /// Match the rendered grid's two triangles; ignore its appended skirts.
    pub fn height(&self, point: GlobalPosition) -> Option<f32> {
        if !matches!(self.cells, 8 | 32)
            || self.positions.len() < (self.cells + 1).pow(2)
            || point.chunk(128)? != self.id
        {
            return None;
        }
        let p = point.metres();
        let stride = 128. / self.cells as f64;
        let u = (p[0] - self.id.x as f64 * 128.) / stride;
        let v = (p[2] - self.id.z as f64 * 128.) / stride;
        let x = u.floor() as usize;
        let z = v.floor() as usize;
        if x >= self.cells || z >= self.cells {
            return None;
        }
        let (u, v) = ((u - x as f64) as f32, (v - z as f64) as f32);
        let i = z * (self.cells + 1) + x;
        let [a, b, c, d] = [
            self.positions[i][1],
            self.positions[i + 1][1],
            self.positions[i + self.cells + 1][1],
            self.positions[i + self.cells + 2][1],
        ];
        if ![a, b, c, d].into_iter().all(f32::is_finite) {
            return None;
        }
        Some(if u + v <= 1. {
            a + u * (b - a) + v * (c - a)
        } else {
            d + (1. - u) * (c - d) + (1. - v) * (b - d)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: f64, z: f64) -> GlobalPosition {
        GlobalPosition::new([x, 0., z]).unwrap()
    }
    #[test]
    fn north_up_projection_subtracts_global_coordinates_before_float_conversion() {
        for centre in [
            p(0., 0.),
            p(127.999, 128.),
            p(8000., 8000.),
            p(1e12 - 256., -1e12 + 256.),
        ] {
            let [x, _, z] = centre.metres();
            let v = View { centre };
            assert_eq!(v.project(centre), Some([88., 88.]));
            assert_eq!(v.project(p(x + 128., z)), Some([176., 88.]));
            assert_eq!(v.project(p(x, z - 128.)), Some([88., 0.]));
            assert!(v.project(p(x + 128.1, z)).is_none());
            assert!(v.project(p(x, z + 128.1)).is_none());
        }
    }
    #[test]
    fn pixel_centres_and_world_edges_remain_checked_and_do_not_wrap_chunks() {
        let v = View { centre: p(0., 0.) };
        assert_eq!(v.pixel(0, 0).unwrap().metres(), [-126., 0., -126.]);
        assert_eq!(v.pixel(63, 63).unwrap().metres(), [126., 0., 126.]);
        assert!(v.pixel(64, 0).is_none());
        assert!(theia_network::procedural::owner(v.pixel(0, 0).unwrap()).is_none());
        let edge = View {
            centre: p(15999., 15999.),
        };
        assert!(theia_network::procedural::owner(edge.pixel(63, 63).unwrap()).is_none());
        assert!(theia_network::procedural::owner(edge.pixel(0, 0).unwrap()).is_some());
        let far = View {
            centre: p(1e12, 0.),
        };
        assert!(far.pixel(63, 0).is_none());
    }
    #[test]
    fn height_uses_both_actual_mesh_triangles_and_refuses_unknown_or_invalid_grid() {
        let mut g = theia_network::procedural::grid(8);
        g.positions[0][1] = 0.;
        g.positions[1][1] = 10.;
        g.positions[9][1] = 20.;
        g.positions[10][1] = 50.;
        let id = ChunkId { x: 62, z: 62 };
        let tile = HeightTile {
            id,
            cells: 8,
            positions: &g.positions,
        };
        assert_eq!(tile.height(p(7940., 7940.)), Some(7.5));
        assert_eq!(tile.height(p(7948., 7948.)), Some(32.5));
        assert!(tile.height(p(7935.9, 7940.)).is_none());
        assert!(
            HeightTile { cells: 7, ..tile }
                .height(p(7940., 7940.))
                .is_none()
        );
        assert!(
            HeightTile {
                positions: &[],
                ..tile
            }
            .height(p(7940., 7940.))
            .is_none()
        );
        g.positions[0][1] = f32::NAN;
        assert!(
            HeightTile {
                id,
                cells: 8,
                positions: &g.positions
            }
            .height(p(7940., 7940.))
            .is_none()
        );
    }
    #[test]
    fn shared_tile_edges_match_across_lods_without_reading_or_generating_new_tiles() {
        for cells in [8, 32] {
            let mut a = theia_network::procedural::grid(cells);
            let mut b = theia_network::procedural::grid(cells);
            for vertex in &mut a.positions {
                vertex[1] = vertex[0] * 0.25 + vertex[2] * 0.5;
            }
            for vertex in &mut b.positions {
                vertex[1] = (vertex[0] + 128.) * 0.25 + vertex[2] * 0.5;
            }
            let left = HeightTile {
                id: ChunkId { x: 0, z: 0 },
                cells,
                positions: &a.positions,
            };
            let right = HeightTile {
                id: ChunkId { x: 1, z: 0 },
                cells,
                positions: &b.positions,
            };
            assert!(
                (left.height(p(127.999, 61.)).unwrap() - right.height(p(128., 61.)).unwrap()).abs()
                    < 0.001
            );
        }
    }
}
