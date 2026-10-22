//! Y-up metres. Global positions never pass through f32 before frame subtraction.
//! Runtime authority uses these checked types; fixture geometry remains local f32.

/// A conservative bound makes chunk indices and relative-coordinate arithmetic safe.
pub const MAX_GLOBAL_METRES: f64 = 1.0e12;
pub const MAX_LOCAL_METRES: f64 = 4096.;
pub const INITIAL_CHUNK_METRES: u32 = 128;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobalPosition([f64; 3]);

impl GlobalPosition {
    pub fn new(xyz: [f64; 3]) -> Option<Self> {
        xyz.iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_GLOBAL_METRES)
            .then_some(Self(xyz))
    }
    pub fn metres(self) -> [f64; 3] {
        self.0
    }
    /// Chunk ownership is half-open and uses floor, including negative coordinates.
    pub fn chunk(self, size_metres: u32) -> Option<ChunkId> {
        if size_metres == 0 {
            return None;
        }
        let size = f64::from(size_metres);
        Some(ChunkId {
            x: (self.0[0] / size).floor() as i64,
            z: (self.0[2] / size).floor() as i64,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkId {
    pub x: i64,
    pub z: i64,
}

/// Logical addressing contract only; this does not assign simulation ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionId {
    pub x: i64,
    pub z: i64,
}
impl ChunkId {
    /// Checked global lower XZ corner; logical ownership is half-open.
    pub fn origin(self, size_metres: u32) -> Option<GlobalPosition> {
        if size_metres == 0 {
            return None;
        }
        let size = f64::from(size_metres);
        GlobalPosition::new([self.x as f64 * size, 0., self.z as f64 * size])
    }
    pub fn region(self, chunks_per_axis: u32) -> Option<RegionId> {
        if chunks_per_axis == 0 {
            return None;
        }
        let size = i64::from(chunks_per_axis);
        Some(RegionId {
            x: self.x.div_euclid(size),
            z: self.z.div_euclid(size),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalPosition([f32; 3]);

impl LocalPosition {
    pub fn new(xyz: [f32; 3]) -> Option<Self> {
        xyz.iter()
            .all(|v| v.is_finite() && f64::from(v.abs()) <= MAX_LOCAL_METRES)
            .then_some(Self(xyz))
    }
    pub fn metres(self) -> [f32; 3] {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LocalFrame {
    origin: GlobalPosition,
}

impl LocalFrame {
    pub fn new(origin: GlobalPosition) -> Self {
        Self { origin }
    }
    pub fn origin(self) -> GlobalPosition {
        self.origin
    }
    pub fn to_local(self, position: GlobalPosition) -> Option<LocalPosition> {
        let relative = std::array::from_fn::<_, 3, _>(|i| position.0[i] - self.origin.0[i]);
        if relative.iter().any(|v| v.abs() > MAX_LOCAL_METRES) {
            return None;
        }
        LocalPosition::new(relative.map(|v| v as f32))
    }
    pub fn to_global(self, position: LocalPosition) -> Option<GlobalPosition> {
        GlobalPosition::new(std::array::from_fn(|i| {
            self.origin.0[i] + f64::from(position.0[i])
        }))
    }
    /// Return a new frame; callers reproject all local state from global truth.
    /// No incremental f32 translation is accumulated across origin changes.
    pub fn rebased(self, origin: GlobalPosition) -> Self {
        Self::new(origin)
    }
}

/// Snap a local frame around a global focus, relative to an immutable content
/// anchor. Subtraction and cell selection stay in f64, including negative seams.
pub fn snapped_frame(anchor: LocalFrame, focus: GlobalPosition, span: f32) -> Option<LocalFrame> {
    if !span.is_finite() || !(1. ..=1024.).contains(&span) {
        return None;
    }
    anchor.to_local(focus)?;
    let base = anchor.origin().metres();
    let p = focus.metres();
    let size = f64::from(span);
    Some(LocalFrame::new(GlobalPosition::new(std::array::from_fn(
        |i| base[i] + ((p[i] - base[i]) / size).floor() * size,
    ))?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_world_centimetres_survive_round_trip_and_origin_changes() {
        for sign in [-1., 1.] {
            let origin = GlobalPosition::new([sign * 1.0e9, 0., sign * 1.0e9]).unwrap();
            let position =
                GlobalPosition::new([origin.0[0] + 0.01, 1.8, origin.0[2] - 0.02]).unwrap();
            let frame = LocalFrame::new(origin);
            let local = frame.to_local(position).unwrap();
            assert!((local.0[0] - 0.01).abs() < 0.000001);
            let restored = frame.to_global(local).unwrap();
            for i in 0..3 {
                assert!((restored.0[i] - position.0[i]).abs() < 0.000001);
            }
            let next =
                frame.rebased(GlobalPosition::new([origin.0[0] + 128., 0., origin.0[2]]).unwrap());
            let projected = next.to_local(position).unwrap();
            let restored = next.to_global(projected).unwrap();
            for i in 0..3 {
                assert!((restored.0[i] - position.0[i]).abs() < 0.00001);
            }
        }
    }

    #[test]
    fn seams_have_single_owner_on_both_sides_of_zero() {
        for (x, expected) in [
            (-128.001, -2),
            (-128., -1),
            (-0.001, -1),
            (0., 0),
            (127.999, 0),
            (128., 1),
        ] {
            let p = GlobalPosition::new([x, 0., -x]).unwrap();
            assert_eq!(p.chunk(INITIAL_CHUNK_METRES).unwrap().x, expected);
            assert!(p.chunk(0).is_none());
        }
    }

    #[test]
    fn invalid_and_out_of_frame_coordinates_are_rejected() {
        for value in [f64::NAN, f64::INFINITY, MAX_GLOBAL_METRES + 1.] {
            assert!(GlobalPosition::new([value, 0., 0.]).is_none());
        }
        assert!(LocalPosition::new([f32::NAN, 0., 0.]).is_none());
        assert!(LocalPosition::new([4097., 0., 0.]).is_none());
        let origin = GlobalPosition::new([0.; 3]).unwrap();
        assert!(
            LocalFrame::new(origin)
                .to_local(GlobalPosition::new([4097., 0., 0.]).unwrap())
                .is_none()
        );
        let edge = GlobalPosition::new([MAX_GLOBAL_METRES, 0., 0.]).unwrap();
        assert!(
            LocalFrame::new(edge)
                .to_global(LocalPosition::new([1., 0., 0.]).unwrap())
                .is_none()
        );
    }
    #[test]
    fn chunk_corners_are_checked_and_round_trip_to_their_half_open_owner() {
        for x in [-7_812_500, -1, 0, 1, 7_812_500] {
            let id = ChunkId { x, z: -x };
            assert_eq!(id.origin(128).unwrap().chunk(128), Some(id));
            assert!(id.origin(0).is_none());
        }
        assert!(ChunkId { x: i64::MAX, z: 0 }.origin(128).is_none());
    }
    #[test]
    fn region_seams_use_floor_division_for_negative_chunks() {
        for (chunk, region) in [(-9, -2), (-8, -1), (-1, -1), (0, 0), (7, 0), (8, 1)] {
            let id = ChunkId { x: chunk, z: chunk };
            assert_eq!(
                id.region(8),
                Some(RegionId {
                    x: region,
                    z: region
                })
            );
            assert!(id.region(0).is_none());
        }
    }
    #[test]
    fn snapped_frames_reproject_canonical_positions_across_signed_seams_without_drift() {
        for sign in [-1., 1.] {
            let anchor = LocalFrame::new(
                GlobalPosition::new([sign * 1e9 + 0.01, sign * 1e9 + 0.02, sign * 1e9 - 0.03])
                    .unwrap(),
            );
            let landmark = anchor
                .to_global(LocalPosition::new([20.125, 1.875, -31.25]).unwrap())
                .unwrap();
            for offset in [-256., -128.01, -0.01, 0., 127.99, 128., 256.] {
                let focus = anchor
                    .to_global(LocalPosition::new([offset, 0.9, -offset]).unwrap())
                    .unwrap();
                let frame = snapped_frame(anchor, focus, 128.).unwrap();
                let local = frame.to_local(focus).unwrap().metres();
                assert!(local.iter().all(|v| *v >= 0. && *v < 128.));
                let restored = frame
                    .to_global(frame.to_local(landmark).unwrap())
                    .unwrap()
                    .metres();
                for (a, b) in restored.into_iter().zip(landmark.metres()) {
                    assert!((a - b).abs() < 0.0001);
                }
            }
            assert!(snapped_frame(anchor, landmark, 0.).is_none());
        }
    }
}
