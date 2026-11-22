//! Live, bounded terrain view policy driven by local Graphics preferences.
use super::*;
use crate::fantasy_ui::graphics::ViewDistance;
use std::collections::BTreeSet;

pub(super) struct View {
    choice: ViewDistance,
    full: Option<BTreeSet<ChunkId>>,
    far: Option<f32>,
}
impl View {
    pub fn parse(
        setting: Option<&str>,
        layout: Option<([i64; 2], [usize; 2])>,
    ) -> Result<Self, String> {
        match setting {
            None | Some("nearby") => Ok(Self {
                choice: ViewDistance::Nearby,
                full: None,
                far: None,
            }),
            Some("full") => {
                let (origin, counts) = layout.ok_or("Full terrain view requires saved terrain")?;
                if counts.iter().any(|&n| !(1..=32).contains(&n))
                    || (0..2).any(|i| origin[i] < 0 || origin[i] > 125 - counts[i] as i64)
                {
                    return Err(
                        "Full terrain view supports at most 32 x 32 saved tiles inside the world"
                            .into(),
                    );
                }
                let full = (0..counts[1])
                    .flat_map(|z| {
                        (0..counts[0]).map(move |x| ChunkId {
                            x: origin[0] + x as i64,
                            z: origin[1] + z as i64,
                        })
                    })
                    .collect();
                // Cover opposite corners, the admitted +/-1024 m height range,
                // and a margin for the third-person camera, without changing near Z.
                let far = (counts[0] as f32 * 128.)
                    .hypot(counts[1] as f32 * 128.)
                    .hypot(2048.)
                    + 512.;
                Ok(Self {
                    choice: ViewDistance::Maximum,
                    full: Some(full),
                    far: Some(far),
                })
            }
            _ => Err("THEIA_TERRAIN_VIEW must be nearby or full".into()),
        }
    }
    pub fn for_choice(choice: ViewDistance, layout: Option<([i64; 2], [usize; 2])>) -> Self {
        if choice == ViewDistance::Maximum
            && let Ok(view) = Self::parse(Some("full"), layout)
        {
            return view;
        }
        Self {
            choice,
            full: None,
            far: (choice != ViewDistance::Nearby).then(|| {
                ((choice.radius() + 1) as f32 * 128. * std::f32::consts::SQRT_2).hypot(2048.) + 512.
            }),
        }
    }
    pub fn budget(&self) -> usize {
        self.full.as_ref().map_or(
            ((self.choice.radius() * 2 + 1).pow(2)) as usize,
            BTreeSet::len,
        )
    }
    pub fn name(&self) -> &'static str {
        if self.full.is_some() {
            "full"
        } else {
            match self.choice {
                ViewDistance::Nearby => "nearby",
                ViewDistance::Far => "far",
                ViewDistance::Maximum => "maximum",
            }
        }
    }
    pub fn desired(&self, source: &procedural::Source, owner: ChunkId) -> BTreeMap<ChunkId, usize> {
        let ids = self
            .full
            .clone()
            .unwrap_or_else(|| source.neighbours(owner, self.choice.radius()));
        let full_cells = source.cells();
        let far_cells = if full_cells > 32 { 32 } else { 8 };
        ids.into_iter()
            .map(|id| {
                let distance = (id.x - owner.x).abs().max((id.z - owner.z).abs());
                (id, if distance <= 2 { full_cells } else { far_cells })
            })
            .collect()
    }
    fn projection(&self, projection: &mut Projection) {
        if let (Some(far), Projection::Perspective(p)) = (self.far, projection) {
            p.far = p.far.max(far);
        }
    }
}

#[derive(Component)]
pub(super) struct OriginalFar(f32);

pub(super) fn install(app: &mut App) {
    // The legacy environment switch supplies an initial choice. A saved Graphics
    // preference (installed later) and live menu actions take precedence.
    let choice = app.world().resource::<Graybox>().view.choice;
    if !app.world().contains_resource::<ViewDistance>() {
        app.insert_resource(choice);
    }
    app.add_systems(Update, configure_camera.before(super::tick));
}
pub(super) fn configure_camera(
    mut commands: Commands,
    choice: Res<ViewDistance>,
    mut gray: ResMut<Graybox>,
    mut cameras: Query<(Entity, &mut Projection, Option<&OriginalFar>), With<Camera3d>>,
) {
    if gray.view.choice != *choice {
        gray.view = View::for_choice(
            *choice,
            gray.saved.as_ref().map(|store| store.chunk_layout()),
        );
    }
    for (entity, mut projection, original) in &mut cameras {
        if let Projection::Perspective(p) = &mut *projection {
            let base = original.map_or(p.far, |original| original.0);
            if original.is_none() {
                commands.entity(entity).insert(OriginalFar(base));
            }
            p.far = base;
            gray.view.projection(&mut projection);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fantasy_ui::graphics::ViewDistance;
    #[test]
    fn live_distance_changes_restore_camera_and_bound_each_ring() {
        let mut app = App::new();
        app.init_resource::<ViewDistance>()
            .insert_resource(Graybox::new(procedural::Source::Procedural))
            .add_systems(Update, configure_camera);
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    far: 1200.,
                    near: 0.2,
                    ..default()
                }),
            ))
            .id();
        for (choice, count) in [
            (ViewDistance::Nearby, 81),
            (ViewDistance::Far, 289),
            (ViewDistance::Maximum, 961),
            (ViewDistance::Nearby, 81),
        ] {
            *app.world_mut().resource_mut::<ViewDistance>() = choice;
            app.update();
            let gray = app.world().resource::<Graybox>();
            assert_eq!(gray.view.budget(), count);
            let desired = gray.view.desired(&gray.source, ChunkId { x: 60, z: 60 });
            assert_eq!(desired.len(), count);
            assert_eq!(
                desired
                    .values()
                    .filter(|&&n| n == gray.source.cells())
                    .count(),
                25
            );
            let Projection::Perspective(p) = app.world().get::<Projection>(camera).unwrap() else {
                panic!()
            };
            assert_eq!(p.near, 0.2);
            if choice == ViewDistance::Nearby {
                assert_eq!(p.far, 1200.);
            } else {
                assert!(p.far > 2000.);
            }
        }
        let full = View::for_choice(ViewDistance::Maximum, Some(([0, 0], [20, 23])));
        assert_eq!(full.budget(), 460);
        let large = View::for_choice(ViewDistance::Maximum, Some(([0, 0], [125, 125])));
        assert_eq!(large.budget(), 961);
    }
    #[test]
    fn full_zone_keeps_all_offset_tiles_while_near_detail_follows_player() {
        let view = View::parse(Some("full"), Some(([7, 9], [20, 23]))).unwrap();
        let source = procedural::Source::Procedural;
        let a = view.desired(&source, ChunkId { x: 15, z: 20 });
        let b = view.desired(&source, ChunkId { x: 26, z: 31 });
        assert_eq!(view.budget(), 460);
        assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
        assert_eq!(a.values().filter(|&&cells| cells == 32).count(), 25);
        assert_eq!(b.values().filter(|&&cells| cells == 32).count(), 9);
        assert_eq!(a[&ChunkId { x: 26, z: 31 }], 8);
        assert_eq!(b[&ChunkId { x: 26, z: 31 }], 32);
        assert!(
            a.keys()
                .all(|id| (7..27).contains(&id.x) && (9..32).contains(&id.z))
        );
    }
    #[test]
    fn nearby_policy_and_camera_remain_unchanged() {
        let view = View::parse(None, None).unwrap();
        assert_eq!(view.budget(), 81);
        assert_eq!(
            view.desired(&procedural::Source::Procedural, ChunkId { x: 15, z: 20 })
                .len(),
            81
        );
        let mut projection = Projection::Perspective(PerspectiveProjection::default());
        let original = projection.clone();
        view.projection(&mut projection);
        let (Projection::Perspective(a), Projection::Perspective(b)) = (projection, original)
        else {
            panic!()
        };
        assert_eq!(a.far, b.far);
        assert_eq!(a.near, b.near);
    }
    #[test]
    fn full_view_rejects_unbounded_sources_and_covers_maximum_diagonal() {
        for layout in [
            None,
            Some(([0, 0], [125, 125])),
            Some(([0, 0], [0, 1])),
            Some(([124, 0], [2, 1])),
            Some(([-1, 0], [1, 1])),
        ] {
            assert!(View::parse(Some("full"), layout).is_err());
        }
        assert!(View::parse(Some("maximum"), None).is_err());
        let view = View::parse(Some("full"), Some(([93, 93], [32, 32]))).unwrap();
        assert_eq!(view.budget(), 1024);
        let mut projection = Projection::Perspective(PerspectiveProjection::default());
        view.projection(&mut projection);
        let Projection::Perspective(p) = projection else {
            panic!()
        };
        assert!(p.far > 4096_f32.hypot(4096.).hypot(2048.));
        assert_eq!(p.near, PerspectiveProjection::default().near);
    }
}
