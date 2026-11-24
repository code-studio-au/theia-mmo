//! Scheduling boundary for retaining the original UI inside a larger editor.
//! Hosts must also manage UI visibility, cameras and preview lifetime when inactive.
use bevy::{ecs::system::ScheduleSystem, prelude::*};

#[derive(Resource)]
pub struct AuthoringHost {
    pub active: bool,
    /// Run reusable inspector/widget systems while the full workspace is hidden.
    pub inline_ui: bool,
    pub embedded: bool,
    pub scene_layer: usize,
}

impl Default for AuthoringHost {
    fn default() -> Self {
        Self {
            active: true,
            inline_ui: false,
            embedded: false,
            scene_layer: 0,
        }
    }
}

impl AuthoringHost {
    pub fn embedded(scene_layer: usize) -> Self {
        Self {
            active: false,
            inline_ui: false,
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

/// Reusable inspector and widget updates, independent of the full workspace.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuthoringUiUpdate;

pub struct AuthoringHostPlugin;

impl Plugin for AuthoringHostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AuthoringHost>()
            .configure_sets(Update, AuthoringUpdate.run_if(authoring_active))
            .configure_sets(Update, AuthoringUiUpdate.run_if(authoring_ui_active))
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
fn authoring_ui_active(host: Res<AuthoringHost>) -> bool {
    host.active || host.inline_ui
}

pub(crate) trait AuthoringAppExt {
    fn add_authoring_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;
    fn add_inline_ui_systems<M>(
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
    fn add_inline_ui_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.add_systems(Update, systems.in_set(AuthoringUiUpdate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Counts {
        workspace: u32,
        ui: u32,
    }

    #[test]
    fn inline_inspector_runs_without_workspace_systems() {
        let mut app = App::new();
        app.insert_resource(AuthoringHost::embedded(1))
            .init_resource::<Counts>()
            .add_plugins(AuthoringHostPlugin)
            .add_authoring_systems(|mut counts: ResMut<Counts>| counts.workspace += 1)
            .add_inline_ui_systems(|mut counts: ResMut<Counts>| counts.ui += 1);
        app.update();
        assert_eq!(
            (
                app.world().resource::<Counts>().workspace,
                app.world().resource::<Counts>().ui
            ),
            (0, 0)
        );
        app.world_mut().resource_mut::<AuthoringHost>().inline_ui = true;
        app.update();
        assert_eq!(
            (
                app.world().resource::<Counts>().workspace,
                app.world().resource::<Counts>().ui
            ),
            (0, 1)
        );
        app.world_mut().resource_mut::<AuthoringHost>().active = true;
        app.update();
        assert_eq!(
            (
                app.world().resource::<Counts>().workspace,
                app.world().resource::<Counts>().ui
            ),
            (1, 2)
        );
    }
}
