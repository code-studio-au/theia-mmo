//! Scheduling boundary for retaining the original UI inside a larger editor.
//! Hosts must also manage UI visibility, cameras and preview lifetime when inactive.
use bevy::{ecs::system::ScheduleSystem, prelude::*};

#[derive(Resource)]
pub struct AuthoringHost {
    pub active: bool,
}

impl Default for AuthoringHost {
    fn default() -> Self {
        Self { active: true }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuthoringUpdate;

pub struct AuthoringHostPlugin;

impl Plugin for AuthoringHostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AuthoringHost>()
            .configure_sets(Update, AuthoringUpdate.run_if(authoring_active));
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
