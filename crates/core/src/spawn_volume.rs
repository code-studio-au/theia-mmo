//! Validated polygon prisms for spawn authoring/admission. Containment alone never
//! establishes walkable support, clearance, navigation or permission to spawn.
use crate::coordinates::GlobalPosition;
use serde::{Deserialize, Serialize};

pub const MAX_VERTICES: usize = 64;
const EPS: f64 = 1e-9;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Document", into = "Document")]
pub struct PolygonVolume {
    polygon: Vec<[f64; 2]>,
    min_y: f64,
    max_y: f64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    polygon: Vec<[f64; 2]>,
    min_y: f64,
    max_y: f64,
}
impl TryFrom<Document> for PolygonVolume {
    type Error = String;
    fn try_from(value: Document) -> Result<Self, Self::Error> {
        Self::new(value.polygon, value.min_y, value.max_y)
    }
}
impl From<PolygonVolume> for Document {
    fn from(value: PolygonVolume) -> Self {
        Self {
            polygon: value.polygon,
            min_y: value.min_y,
            max_y: value.max_y,
        }
    }
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn on_segment(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> bool {
    cross(a, b, p).abs() <= EPS
        && (0..2).all(|i| p[i] >= a[i].min(b[i]) - EPS && p[i] <= a[i].max(b[i]) + EPS)
}
fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let opposite = |x: f64, y: f64| (x > EPS && y < -EPS) || (x < -EPS && y > EPS);
    (opposite(cross(a, b, c), cross(a, b, d)) && opposite(cross(c, d, a), cross(c, d, b)))
        || on_segment(a, b, c)
        || on_segment(a, b, d)
        || on_segment(c, d, a)
        || on_segment(c, d, b)
}
impl PolygonVolume {
    /// Implicitly closes the X/Z polygon. Both windings and concave polygons are
    /// accepted; duplicate closing vertices, touching edges and holes are not.
    pub fn new(polygon: Vec<[f64; 2]>, min_y: f64, max_y: f64) -> Result<Self, String> {
        if !(3..=MAX_VERTICES).contains(&polygon.len()) {
            return Err("Spawn boundary needs 3..64 vertices".into());
        }
        if !min_y.is_finite() || !max_y.is_finite() || !(0.01..=128.).contains(&(max_y - min_y)) {
            return Err("Spawn vertical span must be 0.01..128 metres".into());
        }
        for &[x, z] in &polygon {
            if GlobalPosition::new([x, min_y, z]).is_none()
                || GlobalPosition::new([x, max_y, z]).is_none()
            {
                return Err("Invalid global spawn boundary coordinate".into());
            }
        }
        let n = polygon.len();
        for axis in 0..2 {
            let min = polygon
                .iter()
                .map(|p| p[axis])
                .fold(f64::INFINITY, f64::min);
            let max = polygon
                .iter()
                .map(|p| p[axis])
                .fold(f64::NEG_INFINITY, f64::max);
            if max - min > 1024. {
                return Err("Spawn boundary exceeds 1024 metre span".into());
            }
        }
        let mut area = 0.;
        for i in 0..n {
            let a = polygon[i];
            let b = polygon[(i + 1) % n];
            let c = polygon[(i + 2) % n];
            if (b[0] - a[0]).hypot(b[1] - a[1]) < 0.25 {
                return Err("Spawn boundary edges need at least 0.25 metres".into());
            }
            if cross(a, b, c).abs() <= EPS
                && (a[0] - b[0]) * (c[0] - b[0]) + (a[1] - b[1]) * (c[1] - b[1]) > 0.
            {
                return Err("Spawn boundary doubles back along an edge".into());
            }
            for j in i + 1..n {
                if j == i + 1 || (i == 0 && j == n - 1) {
                    continue;
                }
                if intersects(a, b, polygon[j], polygon[(j + 1) % n]) {
                    return Err("Spawn boundary edges cross or touch".into());
                }
            }
            area += cross(polygon[0], a, b);
        }
        if area.abs() < 0.0002 {
            return Err("Spawn boundary has no usable area".into());
        }
        Ok(Self {
            polygon,
            min_y,
            max_y,
        })
    }
    pub fn polygon(&self) -> &[[f64; 2]] {
        &self.polygon
    }
    pub fn min_y(&self) -> f64 {
        self.min_y
    }
    pub fn max_y(&self) -> f64 {
        self.max_y
    }
    pub fn translated(&self, offset: [f64; 3]) -> Result<Self, String> {
        if offset.iter().any(|v| !v.is_finite()) {
            return Err("Invalid spawn volume offset".into());
        }
        Self::new(
            self.polygon
                .iter()
                .map(|p| [p[0] + offset[0], p[1] + offset[2]])
                .collect(),
            self.min_y + offset[1],
            self.max_y + offset[1],
        )
    }
    /// Includes the polygon/height boundary. Ground and capsule checks are separate.
    pub fn contains(&self, position: GlobalPosition) -> bool {
        let [x, y, z] = position.metres();
        if y < self.min_y || y > self.max_y {
            return false;
        }
        let origin = self.polygon[0];
        let p = [x - origin[0], z - origin[1]];
        let local = |v: [f64; 2]| [v[0] - origin[0], v[1] - origin[1]];
        let mut inside = false;
        for i in 0..self.polygon.len() {
            let a = local(self.polygon[i]);
            let b = local(self.polygon[(i + 1) % self.polygon.len()]);
            if on_segment(a, b, p) {
                return true;
            }
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1])
            {
                inside = !inside;
            }
        }
        inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn boundary() -> Vec<[f64; 2]> {
        vec![[0., 0.], [8., 0.], [8., 3.], [3., 3.], [3., 8.], [0., 8.]]
    }
    #[test]
    fn concave_custom_bounds_retain_the_notch_and_height_layer_at_far_origins() {
        for offset in [[0.; 3], [128_000_000_000.125, 20.25, -128_000_000_000.5]] {
            for reverse in [false, true] {
                let mut points = boundary();
                if reverse {
                    points.reverse();
                }
                let volume = PolygonVolume::new(points, -2., 4.)
                    .unwrap()
                    .translated(offset)
                    .unwrap();
                for (p, expected) in [
                    ([1., 0., 7.], true),
                    ([7., 0., 1.], true),
                    ([7., 0., 7.], false),
                    ([3., 0., 5.], true),
                    ([-0.125, 0., 1.], false),
                    ([1., 4., 1.], true),
                    ([1., 4.125, 1.], false),
                    ([1., -2.125, 1.], false),
                ] {
                    let p = GlobalPosition::new(std::array::from_fn(|i| p[i] + offset[i])).unwrap();
                    assert_eq!(volume.contains(p), expected, "{p:?}");
                }
                let bytes = serde_json::to_vec(&volume).unwrap();
                assert_eq!(
                    serde_json::from_slice::<PolygonVolume>(&bytes).unwrap(),
                    volume
                );
            }
        }
    }
    #[test]
    fn malformed_boundaries_and_untrusted_deserialization_refuse() {
        for polygon in [
            vec![[0., 0.], [8., 8.], [0., 8.], [8., 0.]],
            vec![[0., 0.], [4., 0.], [2., 0.], [2., 4.]],
            vec![[0., 0.], [4., 0.], [8., 0.]],
            vec![[0., 0.], [4., 0.], [4., 4.], [0., 0.]],
            vec![[0., 0.], [4., 0.], [4., 4.], [2., 0.], [0., 4.]],
            vec![[0., 0.], [0.01, 0.], [4., 4.]],
            vec![[0., 0.], [1025., 0.], [0., 4.]],
        ] {
            assert!(PolygonVolume::new(polygon, -1., 1.).is_err());
        }
        assert!(PolygonVolume::new(vec![[0., 0.]; 65], -1., 1.).is_err());
        assert!(PolygonVolume::new(boundary(), 1., 1.).is_err());
        assert!(PolygonVolume::new(boundary(), 0., 129.).is_err());
        assert!(PolygonVolume::new(vec![[0., 0.], [f64::NAN, 0.], [1., 1.]], 0., 1.).is_err());
        assert!(
            serde_json::from_str::<PolygonVolume>(
                r#"{"polygon":[[0,0],[4,4],[0,4],[4,0]],"min_y":0,"max_y":1}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<PolygonVolume>(
                r#"{"polygon":[[0,0],[4,0],[0,4]],"min_y":0,"max_y":1,"unknown":true}"#
            )
            .is_err()
        );
    }
}
