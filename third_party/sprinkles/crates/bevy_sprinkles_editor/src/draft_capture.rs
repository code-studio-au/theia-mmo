//! Periodic, best-effort snapshots of the active particle project.
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_sprinkles::prelude::ParticlesAsset;

use crate::draft_writer::DraftWriter;
use crate::drafts::{Draft, DraftStore};
use crate::state::{DirtyState, EditorState};
use crate::ui::components::toasts::ToastEvent;

const INTERVAL: Duration = Duration::from_secs(5);

pub(crate) struct DraftCapture {
    writer: Option<DraftWriter>,
    handle: Option<Handle<ParticlesAsset>>,
    session: u64,
    last_queued: Option<String>,
    baseline: Option<String>,
    previous_path: Option<PathBuf>,
    previous_dirty: bool,
    previous_active: bool,
    transitions: VecDeque<(u64, Draft)>,
    next: Instant,
    startup_error: Option<String>,
    #[cfg(test)]
    pub(crate) disabled: bool,
}

impl DraftCapture {
    pub(crate) fn new(store: DraftStore) -> Self {
        let (writer, startup_error) = match DraftWriter::start(store) {
            Ok(writer) => (Some(writer), None),
            Err(error) => (None, Some(error.to_string())),
        };
        Self {
            writer,
            handle: None,
            session: 0,
            last_queued: None,
            baseline: None,
            previous_path: None,
            previous_dirty: false,
            previous_active: false,
            transitions: VecDeque::new(),
            next: Instant::now(),
            startup_error,
            #[cfg(test)]
            disabled: false,
        }
    }
}

pub(crate) fn capture_drafts(
    mut capture: NonSendMut<DraftCapture>,
    host: Res<crate::hosting::AuthoringHost>,
    state: Res<EditorState>,
    dirty: Res<DirtyState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut commands: Commands,
) {
    #[cfg(test)]
    if capture.disabled {
        return;
    }
    if let Some(error) = capture.startup_error.take() {
        commands.trigger(ToastEvent::error(format!(
            "Particle draft writer unavailable: {error}"
        )));
    }
    while let Some(result) = capture.writer.as_ref().and_then(DraftWriter::try_result) {
        if let Err(error) = result.result {
            if result.session == capture.session {
                capture.last_queued = None;
            }
            commands.trigger(ToastEvent::error(format!(
                "Could not save particle draft: {error}"
            )));
        }
    }
    while let Some((session, draft)) = capture.transitions.front() {
        let Some(writer) = capture.writer.as_ref() else {
            break;
        };
        match writer.capture(*session, draft.clone()) {
            Ok(()) => {
                capture.transitions.pop_front();
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error) => {
                commands.trigger(ToastEvent::error(format!(
                    "Could not queue transition draft: {error}"
                )));
                capture.transitions.pop_front();
            }
        }
    }
    let switched = capture.handle != state.current_project;
    let leaving = capture.previous_active && !host.active;
    if (switched || leaving) && capture.previous_active {
        let old_snapshot = capture
            .handle
            .as_ref()
            .and_then(|handle| assets.get(handle))
            .and_then(|asset| crate::project::serialize_snapshot(asset).ok());
        if let Some(snapshot) = old_snapshot
            && (capture.previous_dirty || capture.baseline.as_ref() != Some(&snapshot))
            && capture.last_queued.as_ref() != Some(&snapshot)
        {
            let draft = Draft::new(capture.previous_path.clone(), snapshot);
            let session = capture.session;
            if capture.transitions.len() < 32 {
                capture.transitions.push_back((session, draft));
            } else {
                commands.trigger(ToastEvent::error(
                    "Too many pending particle drafts; save the project",
                ));
            }
        }
    }
    capture.previous_active = host.active;
    if !host.active {
        return;
    }
    if switched {
        capture.handle = state.current_project.clone();
        capture.session = capture.session.wrapping_add(1);
        capture.last_queued = None;
        capture.baseline = state
            .current_project
            .as_ref()
            .and_then(|handle| assets.get(handle))
            .and_then(|asset| crate::project::serialize_snapshot(asset).ok());
        capture.next = Instant::now();
    }
    let became_clean = capture.previous_dirty && !dirty.has_unsaved_changes;
    capture.previous_dirty = dirty.has_unsaved_changes;
    capture.previous_path = state.current_project_path.clone();
    if !dirty.has_unsaved_changes {
        if became_clean {
            capture.baseline = state
                .current_project
                .as_ref()
                .and_then(|handle| assets.get(handle))
                .and_then(|asset| crate::project::serialize_snapshot(asset).ok());
        }
        return;
    }
    if Instant::now() < capture.next {
        return;
    }
    capture.next = Instant::now() + INTERVAL;
    let Some(asset) = state
        .current_project
        .as_ref()
        .and_then(|handle| assets.get(handle))
    else {
        return;
    };
    let Ok(snapshot) = crate::project::serialize_snapshot(asset) else {
        commands.trigger(ToastEvent::error("Could not serialize particle draft"));
        return;
    };
    if capture.last_queued.as_ref() == Some(&snapshot) {
        return;
    }
    let draft = Draft::new(state.current_project_path.clone(), snapshot.clone());
    let Some(writer) = capture.writer.as_ref() else {
        return;
    };
    match writer.capture(capture.session, draft) {
        Ok(()) => capture.last_queued = Some(snapshot),
        Err(error) => commands.trigger(ToastEvent::error(format!(
            "Could not queue particle draft: {error}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::prelude::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn active_dirty_project_is_captured_without_writing_its_source() {
        let dir = std::env::temp_dir().join(format!(
            "theia-capture-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("effect.ron");
        std::fs::write(&source, b"original").unwrap();
        let store = DraftStore::new(dir.join("drafts"));
        let mut app = App::new();
        app.insert_non_send(DraftCapture::new(store.clone()))
            .init_resource::<crate::hosting::AuthoringHost>()
            .init_resource::<EditorState>()
            .init_resource::<DirtyState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .add_systems(Update, capture_drafts);
        let asset = ParticlesAsset::new(
            "Snapshot".into(),
            ParticlesDimension::D3,
            default(),
            vec![],
            vec![],
            false,
            default(),
        );
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(asset);
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = Some(handle.clone());
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project_path = Some(source.clone());
        app.update();
        assert!(store.list().unwrap().is_empty());
        app.world_mut()
            .resource_mut::<crate::hosting::AuthoringHost>()
            .active = false;
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = true;
        app.update();
        assert!(store.list().unwrap().is_empty());
        app.world_mut()
            .resource_mut::<crate::hosting::AuthoringHost>()
            .active = true;
        app.world_mut().non_send_mut::<DraftCapture>().next = Instant::now();
        let deadline = Instant::now() + Duration::from_secs(10);
        let id = loop {
            assert!(Instant::now() < deadline, "capture timed out");
            app.update();
            if let Some(id) = store.list().unwrap().into_iter().next() {
                break id;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        let restored = store.recover(&id).unwrap();
        assert_eq!(restored.asset.name, "Snapshot");
        assert_eq!(restored.original_path.as_deref(), Some(source.as_path()));
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .name = "Before leaving".into();
        app.world_mut()
            .resource_mut::<crate::hosting::AuthoringHost>()
            .active = false;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(Instant::now() < deadline, "transition capture timed out");
            app.update();
            if store.recover(&id).unwrap().asset.name == "Before leaving" {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        app.world_mut()
            .resource_mut::<crate::hosting::AuthoringHost>()
            .active = true;
        app.update();
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&handle)
            .unwrap()
            .name = "Before switching".into();
        let replacement = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Other".into(),
                ParticlesDimension::D3,
                default(),
                vec![],
                vec![],
                false,
                default(),
            ));
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = Some(replacement);
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project_path = None;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                Instant::now() < deadline,
                "project switch capture timed out"
            );
            app.update();
            if store.recover(&id).unwrap().asset.name == "Before switching" {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(std::fs::read(source).unwrap(), b"original");
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
