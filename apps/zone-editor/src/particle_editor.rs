//! Launch the retained Sprinkles authoring UI without changing the zone document.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use std::process::{Child, Command as ProcessCommand};

#[derive(Resource, Default)]
struct AuthoringProcess(Option<Child>);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AuthoringProcess>()
        .add_zone_systems(update.after(workspace::sync).before(present));
}

fn update(
    mut process: ResMut<AuthoringProcess>,
    mut requests: ResMut<workspace::Requests>,
    mut session: ResMut<Session>,
) {
    if let Some(child) = process.0.as_mut() {
        match child.try_wait() {
            Ok(Some(status)) => {
                process.0 = None;
                if !status.success() {
                    session.status = format!("Particle editor exited: {status}");
                }
            }
            Ok(None) => {}
            Err(error) => {
                session.status = format!("Cannot check particle editor: {error}");
                return;
            }
        }
    }
    if !std::mem::take(&mut requests.particle_editor) {
        return;
    }
    if workspace::busy(&session) {
        return;
    }
    if process.0.is_some() {
        session.status = "Particle editor is already open in its authoring window".into();
        return;
    }
    let result = std::env::current_exe().and_then(|path| {
        let tool = path.with_file_name(format!(
            "theia-particle-editor{}",
            std::env::consts::EXE_SUFFIX
        ));
        ProcessCommand::new(tool).spawn()
    });
    match result {
        Ok(child) => {
            process.0 = Some(child);
            session.status = "Opened Sprinkles particle authoring in a separate window".into();
        }
        Err(error) => {
            session.status = format!(
                "Cannot open particle editor: {error}. Build bevy_sprinkles_editor alongside the zone editor."
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_request_respects_input_ownership_and_preserves_zone_and_redo() {
        let original = Zone::default();
        let mut editor = Editor::new(original.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Temporary".into(),
                }],
            })
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let mut session = Session::new(editor, "unused.zone.json".into(), Some(original.clone()));
        let mut view = View {
            target: Vec3::ZERO,
            yaw: 0.,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        };
        let mut content = outliner::Content::default();
        for owned in [true, false] {
            let mut requests = workspace::Requests::default();
            session.ui.input_owned = owned;
            workspace::execute(
                workspace::Control::ParticleEditor,
                &mut session,
                &mut view,
                &mut requests,
                &mut content,
            );
            assert_eq!(requests.particle_editor, !owned);
            assert_eq!(session.editor.zone(), &original);
            assert_eq!(session.editor.history(), (0, 1));
            assert_eq!(session.editor.revision(), 2);
        }
    }
}
