use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::ViewportNode;

use crate::viewport::EditorCamera;

#[derive(Component, Default, Clone)]
pub struct EditorViewportContainer;

#[derive(Component)]
pub struct EditorViewport;

pub fn viewport_container() -> impl Scene {
    bsn! {
        EditorViewportContainer
        Node {
            flex_grow: 1.0,
            height: percent(100),
        }
        Hovered
    }
}

pub fn setup_viewport(
    mut commands: Commands,
    camera: Query<Entity, With<EditorCamera>>,
    container: Query<Entity, (With<EditorViewportContainer>, Without<EditorViewport>)>,
) {
    let Ok(camera_entity) = camera.single() else {
        return;
    };

    let Ok(container_entity) = container.single() else {
        return;
    };

    commands
        .entity(container_entity)
        .insert((EditorViewport, ViewportNode::new(camera_entity)));
}

#[cfg(test)]
mod readiness_tests {
    use super::*;

    #[test]
    fn viewport_binds_when_scene_container_arrives_after_camera_creation_frame() {
        let mut app = App::new();
        app.init_resource::<bevy::picking::pointer::PointerMap>()
            .add_systems(Update, setup_viewport);
        app.world_mut().spawn(EditorCamera);
        app.update();
        let container = app.world_mut().spawn(EditorViewportContainer).id();
        app.update();
        assert!(app.world().get::<EditorViewport>(container).is_some());
        assert!(app.world().get::<ViewportNode>(container).is_some());
        app.update();
        assert!(app.world().get::<EditorViewport>(container).is_some());
    }
}
