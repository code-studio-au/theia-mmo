//! Connected presentation and Avian coordinates. Geometry remains anchored in
//! content space; rebases never change global authority or imported child poses.
use bevy::prelude::*;
use theia_core::coordinates::{GlobalPosition, LocalFrame, LocalPosition, snapped_frame};
use theia_network::wire::Welcome;

#[derive(Component, Clone, Copy)]
pub struct Anchored(pub [f64; 3]);
impl Anchored {
    pub const ZERO: Self = Self([0.; 3]);
    pub fn at(p: Vec3) -> Self {
        Self(p.to_array().map(f64::from))
    }
}
#[derive(Resource)]
pub struct Frame {
    pub content: LocalFrame,
    pub render: LocalFrame,
    epoch: Option<u64>,
    pub span: f32,
    pub revision: u64,
}
impl Default for Frame {
    fn default() -> Self {
        let zero = LocalFrame::new(GlobalPosition::new([0.; 3]).unwrap());
        Self {
            content: zero,
            render: zero,
            epoch: None,
            span: 128.,
            revision: 0,
        }
    }
}
impl Frame {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut frame = Self::default();
        if let Ok(value) = std::env::var("THEIA_RENDER_SPAN") {
            frame.span = value.parse()?;
            if !frame.span.is_finite() || !(1. ..=1024.).contains(&frame.span) {
                return Err("Render span must be finite, 1..1024 m".into());
            }
        }
        Ok(frame)
    }
    pub fn admit(&mut self, welcome: &Welcome) {
        self.admit_content(welcome, welcome.frame());
    }
    pub fn admit_content(&mut self, welcome: &Welcome, content: LocalFrame) {
        if self.epoch != Some(welcome.epoch) {
            self.content = content;
            self.render = self.content;
            self.epoch = Some(welcome.epoch);
            self.revision = 0;
        }
    }
    pub fn focus_large(&mut self, position: GlobalPosition) {
        if self
            .render
            .to_local(position)
            .is_none_or(|p| p.metres()[0].abs() > 128. || p.metres()[2].abs() > 128.)
        {
            let p = position.metres();
            self.render = LocalFrame::new(
                GlobalPosition::new([
                    (p[0] / 128.).floor() * 128.,
                    0.,
                    (p[2] / 128.).floor() * 128.,
                ])
                .unwrap(),
            );
            self.revision += 1;
        }
    }
    pub fn focus(&mut self, position: GlobalPosition) {
        if self
            .render
            .to_local(position)
            .is_none_or(|p| p.metres().iter().any(|v| v.abs() > self.span))
        {
            self.render =
                snapped_frame(self.content, position, self.span).expect("Bounded content focus");
            self.revision += 1;
        }
    }
    pub fn focus_bounded(
        &mut self,
        position: GlobalPosition,
        required: impl IntoIterator<Item = GlobalPosition>,
    ) {
        let previous = self.render.origin();
        let revision = self.revision;
        self.focus(position);
        if required
            .into_iter()
            .any(|p| self.render.to_local(p).is_none())
        {
            self.render = self.content;
        }
        self.revision = revision + u64::from(self.render.origin() != previous);
    }
    pub fn project(&self, position: GlobalPosition) -> Vec3 {
        Vec3::from_array(
            self.render
                .to_local(position)
                .expect("Bounded render frame")
                .metres(),
        )
    }
    pub fn content_point(&self, p: Vec3) -> Vec3 {
        let global = self
            .render
            .to_global(LocalPosition::new(p.to_array()).expect("Bounded rendered position"))
            .expect("Checked global position");
        Vec3::from_array(
            self.content
                .to_local(global)
                .expect("Bounded content frame")
                .metres(),
        )
    }
    pub fn anchored(&self, position: [f64; 3]) -> Vec3 {
        let base = self.content.origin().metres();
        self.project(
            GlobalPosition::new(std::array::from_fn(|i| base[i] + position[i]))
                .expect("Checked content anchor"),
        )
    }
}

pub fn select(
    mut frame: ResMut<Frame>,
    network: Option<Res<crate::network::Network>>,
    chunks: Option<Res<crate::chunk_world::Config>>,
    graybox: Option<Res<crate::graybox::Graybox>>,
    anchors: Query<&Anchored>,
    remotes: Query<&crate::network::RemotePawn>,
) {
    let Some(network) = network else {
        return;
    };
    let state = network.state();
    if let Some(welcome) = &state.welcome {
        if let Some(chunks) = &chunks {
            frame.admit_content(welcome, chunks.data.index().frame());
        } else {
            frame.admit(welcome);
        }
    }
    if graybox.is_some() {
        if let Some(position) = state.authority_focus() {
            frame.focus_large(position);
        }
        return;
    }
    if let (Some(snapshot), Some(id)) = (&state.snapshot, state.id)
        && let Some(player) = snapshot.players.iter().find(|p| p.id == id)
    {
        let position = snapshot
            .global_position(player)
            .expect("Validated global pawn");
        if graybox.is_some() {
            frame.focus_large(position);
        } else if chunks.is_some() {
            let base = frame.content.origin().metres();
            let required = anchors
                .iter()
                .map(|a| {
                    GlobalPosition::new(std::array::from_fn(|i| base[i] + a.0[i]))
                        .expect("Validated static anchor")
                })
                .chain(remotes.iter().flat_map(|r| r.global_endpoints()))
                .chain(network.rendered_global())
                .chain(
                    snapshot
                        .players
                        .iter()
                        .map(|p| snapshot.global_position(p).unwrap()),
                );
            frame.focus_bounded(position, required);
        } else {
            frame.focus(position);
        }
    }
}

/// Only marked roots move. Children keep authored skeletal/mesh-local transforms.
/// Set Avian Position as well as Transform; parent motion alone cannot move bodies.
pub fn place_static(
    frame: Res<Frame>,
    graybox: Option<Res<crate::graybox::Graybox>>,
    mut roots: Query<(
        &Anchored,
        &mut Transform,
        Option<&mut avian3d::prelude::Position>,
    )>,
) {
    if graybox.is_some() {
        return;
    }
    for (anchor, mut transform, position) in &mut roots {
        let p = frame.anchored(anchor.0);
        if transform.translation != p {
            transform.translation = p;
        }
        if let Some(mut position) = position
            && position.0 != p
        {
            position.0 = p;
        }
    }
}

#[cfg(test)]
mod chunk_tests {
    use super::*;
    #[test]
    fn explicit_chunk_content_frame_is_independent_of_server_query_origin_and_render_rebases() {
        let welcome = Welcome {
            terrain_stream: false,
            id: 1,
            epoch: 7,
            world: [1; 32],
            origin: [1e9, 0., -1e9],
        };
        let content = LocalFrame::new(GlobalPosition::new([1e9 + 8., 25., -1e9 - 8.]).unwrap());
        let body = GlobalPosition::new([1e9 + 126., 0.908, -1e9 + 32.]).unwrap();
        let mut frame = Frame {
            span: 2.,
            ..Frame::default()
        };
        frame.admit_content(&welcome, content);
        let before = frame.content_point(frame.project(body));
        assert!((before - Vec3::new(118., -24.092, 40.)).length() < 0.00001);
        frame.focus(body);
        assert!(frame.revision > 0);
        assert_eq!(frame.content.origin(), content.origin());
        let after = frame.content_point(frame.project(body));
        assert!((after - before).length() < 0.00001);
        // Repeated welcome observation must not undo a live render rebase.
        let render = frame.render.origin();
        frame.admit_content(&welcome, content);
        assert_eq!(frame.render.origin(), render);
    }
    #[test]
    fn chunk_render_frames_retain_distant_roots_and_previous_interpolation_endpoints() {
        let point = |x| GlobalPosition::new([x, 0., 32.]).unwrap();
        let mut frame = Frame {
            span: 2.,
            ..Frame::default()
        };
        frame.focus_bounded(point(-2500.), [point(-2560.), point(2560.), point(2600.)]);
        assert_eq!(frame.render.origin(), frame.content.origin());
        assert_eq!(frame.revision, 0);
        assert_eq!(frame.project(point(2600.)).x, 2600.);
        // The old local pose/remote endpoints still need projection during a
        // teleport/recovery frame, before presentation replaces/removes them.
        frame.focus_bounded(point(-2500.), [point(-2500.)]);
        assert!(frame.revision > 0);
        frame.focus_bounded(point(2600.), [point(-2500.), point(2600.)]);
        assert_eq!(frame.render.origin(), frame.content.origin());
        assert_eq!(frame.project(point(-2500.)).x, -2500.);
        assert_eq!(frame.project(point(2600.)).x, 2600.);
    }
}
