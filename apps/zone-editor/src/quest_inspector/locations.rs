//! Read-only pickup references; shared standard meshes, no physics or picking.
use super::*;

const PARTS: usize = 4;
#[derive(Component)]
pub(crate) struct Part {
    location: usize,
    shape: usize,
}
#[derive(Resource, Default)]
pub(crate) struct Scene {
    signature: Option<(Option<Key>, u64, bool)>,
    locations: Vec<Location>,
}
struct Location {
    area: String,
    position: GlobalPosition,
    local: Vec3,
}
impl Scene {
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!(
            self.locations
                .iter()
                .map(|p| serde_json::json!({
                    "area":p.area,"position":p.position.metres(),"local":p.local.to_array(),
                    "parts":PARTS,
                }))
                .collect::<Vec<_>>()
        )
    }
}

pub(crate) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1., 1., 1.));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1., 0.65, 0.1),
        unlit: true,
        ..default()
    });
    for location in 0..theia_core::rewards::MAX_DEFINITIONS {
        for shape in 0..PARTS {
            commands.spawn((
                Part { location, shape },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                Visibility::Hidden,
                Pickable::IGNORE,
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
            ));
        }
    }
    commands.init_resource::<Scene>();
}

fn locations(session: &Session) -> Vec<Location> {
    if !session.marker_mode {
        return Vec::new();
    }
    let Some((context, npc)) =
        selected(session).and_then(|m| session.quest_context.as_ref().zip(m.definition.as_deref()))
    else {
        return Vec::new();
    };
    context
        .pickups
        .iter()
        .filter(|p| {
            context
                .catalog
                .quest(&p.quest)
                .is_some_and(|q| q.giver().as_str() == npc || q.turn_in().as_str() == npc)
        })
        .filter_map(|p| {
            session
                .editor
                .zone()
                .local(p.position.metres())
                .ok()
                .map(|local| Location {
                    area: p.area.clone(),
                    position: p.position,
                    local: Vec3::from_array(local),
                })
        })
        .take(theia_core::rewards::MAX_DEFINITIONS)
        .collect()
}

fn transforms(point: Vec3) -> [Transform; PARTS] {
    [
        Transform::from_translation(point + Vec3::Y * 0.08).with_scale(Vec3::new(1.2, 0.06, 0.06)),
        Transform::from_translation(point + Vec3::Y * 0.08).with_scale(Vec3::new(0.06, 0.06, 1.2)),
        Transform::from_translation(point + Vec3::Y * 0.78).with_scale(Vec3::new(0.06, 1.4, 0.06)),
        Transform::from_translation(point + Vec3::Y * 1.6)
            .with_scale(Vec3::splat(0.22))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ]
}

pub(crate) fn sync(
    session: Res<Session>,
    mut scene: ResMut<Scene>,
    mut parts: Query<(&Part, &mut Transform, &mut Visibility)>,
) {
    let signature = (
        current_key(&session),
        session.editor.revision(),
        session.marker_mode,
    );
    if scene.signature.as_ref() == Some(&signature) {
        return;
    }
    scene.locations = locations(&session);
    for (part, mut transform, mut visibility) in &mut parts {
        if let Some(location) = scene.locations.get(part.location) {
            *transform = transforms(location.local)[part.shape];
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    scene.signature = Some(signature);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/development-gameplay-starter-economy.json"),
        )
        .unwrap()
    }
    fn fixture(origin: [f64; 3]) -> App {
        let zone = Zone {
            schema: 4,
            origin,
            spawn: origin,
            markers: vec![Marker {
                id: 1,
                name: "Rowan".into(),
                kind: MarkerKind::Npc,
                position: origin,
                yaw_degrees: 0.,
                radius_metres: 1.,
                definition: Some("theia.dev.villager".into()),
            }],
            ..default()
        };
        let mut context = context();
        context.pickups[0].position = GlobalPosition::new(origin.map(|v| v + 0.125)).unwrap();
        let mut session = Session::new(
            crate::Editor::new(zone.clone()).unwrap(),
            PathBuf::new(),
            Some(zone),
        );
        session.set_mode(EditorMode::Markers);
        session.marker_selected = Some(1);
        session.quest_context = Some(context);
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(session)
            .add_systems(Startup, setup)
            .add_systems(Update, sync);
        app.update();
        app
    }
    fn visible(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<&Visibility, With<Part>>()
            .iter(app.world())
            .filter(|v| **v == Visibility::Visible)
            .count()
    }
    #[test]
    fn collection_guides_preserve_far_origin_and_never_add_picking_collision_or_assets_on_selection()
     {
        let mut app = fixture([128_000_000_000.; 3]);
        let original = app.world().resource::<Session>().editor.zone().clone();
        assert_eq!(visible(&mut app), 4);
        assert_eq!(
            app.world().resource::<Scene>().locations[0].local,
            Vec3::splat(0.125)
        );
        for (_, picking) in app
            .world_mut()
            .query::<(&Part, &Pickable)>()
            .iter(app.world())
        {
            assert!(!picking.is_hoverable && !picking.should_block_lower);
        }
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<avian3d::prelude::Collider>>()
                .iter(app.world())
                .count(),
            0
        );
        for _ in 0..20 {
            app.world_mut().resource_mut::<Session>().marker_selected = None;
            app.update();
            assert_eq!(visible(&mut app), 0);
            app.world_mut().resource_mut::<Session>().marker_selected = Some(1);
            app.update();
            assert_eq!(visible(&mut app), 4);
        }
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 1);
        assert_eq!(
            app.world_mut().query::<&Part>().iter(app.world()).count(),
            PARTS * theia_core::rewards::MAX_DEFINITIONS
        );
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &original);
        assert_eq!(session.editor.history(), (0, 0));
        assert_eq!(session.editor.revision(), 0);
    }

    #[test]
    fn collection_guides_clear_for_play_catalogue_changes_and_outside_frame_positions() {
        let mut app = fixture([8192., 0., 8192.]);
        assert_eq!(visible(&mut app), 4);
        let play = crate::playtest::Playtest::new(app.world().resource::<Session>().editor.zone())
            .unwrap();
        app.world_mut().resource_mut::<Session>().play = Some(play);
        app.update();
        assert_eq!(visible(&mut app), 0);
        app.world_mut().resource_mut::<Session>().play = None;
        app.update();
        assert_eq!(visible(&mut app), 4);
        app.world_mut().resource_mut::<Session>().quest_context = None;
        app.update();
        assert_eq!(visible(&mut app), 0);
        app.world_mut().resource_mut::<Session>().quest_context = Some(context());
        app.update();
        assert_eq!(visible(&mut app), 4);
        assert_eq!(app.world().resource::<Scene>().locations[0].local.x, 2.);
        let mut outside = context();
        outside.pickups[0].position = GlobalPosition::new([0.; 3]).unwrap();
        app.world_mut().resource_mut::<Session>().quest_context = Some(outside);
        app.update();
        assert_eq!(visible(&mut app), 0);
        assert!(app.world().resource::<Scene>().locations.is_empty());
    }
}
