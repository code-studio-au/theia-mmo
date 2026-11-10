//! Scheduling boundary for retaining the original UI inside a larger editor.
//! Hosts must also manage UI visibility, cameras and preview lifetime when inactive.
use bevy::{ecs::system::ScheduleSystem, prelude::*};

#[derive(Resource)]
pub struct AuthoringHost {
    pub active: bool,
    pub embedded: bool,
    pub scene_layer: usize,
}

impl Default for AuthoringHost {
    fn default() -> Self {
        Self {
            active: true,
            embedded: false,
            scene_layer: 0,
        }
    }
}

impl AuthoringHost {
    pub fn embedded(scene_layer: usize) -> Self {
        Self {
            active: false,
            embedded: true,
            scene_layer,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct AuthoringUiRoot;

#[derive(Component)]
pub struct ReturnToHostButton;

#[derive(Event)]
pub struct ReturnToHostRequested;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuthoringUpdate;

pub struct AuthoringHostPlugin;

impl Plugin for AuthoringHostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AuthoringHost>()
            .configure_sets(Update, AuthoringUpdate.run_if(authoring_active))
            .add_observer(return_to_host);
    }
}

fn return_to_host(
    event: On<crate::ui::widgets::button::ButtonClickEvent>,
    buttons: Query<(), With<ReturnToHostButton>>,
    mut commands: Commands,
) {
    if buttons.contains(event.entity) {
        commands.trigger(ReturnToHostRequested);
    }
}

pub fn authoring_active(host: Res<AuthoringHost>) -> bool {
    host.active
}

pub(crate) trait AuthoringAppExt {
    fn add_authoring_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;
}

impl AuthoringAppExt for App {
    fn add_authoring_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.add_systems(Update, systems.in_set(AuthoringUpdate))
    }
}
