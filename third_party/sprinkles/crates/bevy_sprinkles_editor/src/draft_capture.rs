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

/// Best-effort bounded flush when an AppExit message is visible in the last schedule.
pub(crate) fn capture_on_exit(
    exits: Option<Res<bevy::ecs::message::Messages<bevy::app::AppExit>>>,
    mut capture: NonSendMut<DraftCapture>,
    host: Res<crate::hosting::AuthoringHost>,
    state: Res<EditorState>,
    dirty: Res<DirtyState>,
    assets: Res<Assets<ParticlesAsset>>,
) {
    if exits.as_ref().is_none_or(|exits| exits.is_empty()) {
        return;
    }
    let Some(writer) = capture.writer.as_ref() else {
        return;
    };
    // Drain earlier work before submitting the final snapshot to the bounded queue.
    if let Err(error) = writer.flush(Duration::from_secs(2)) {
        warn!("Particle draft flush failed: {error}");
        return;
    }
    while let Some(result) = capture.writer.as_ref().and_then(DraftWriter::try_result) {
        if let Err(error) = result.result {
            if result.session == capture.session {
                capture.last_queued = None;
            }
            warn!("Particle draft write failed on exit: {error}");
        }
    }
    while let Some((session, draft)) = capture.transitions.pop_front() {
        let Some(writer) = capture.writer.as_ref() else {
            return;
        };
        if let Err(error) = writer.capture(session, draft.clone()) {
            if error.kind() == std::io::ErrorKind::WouldBlock {
                if let Err(error) = writer
                    .flush(Duration::from_secs(2))
                    .and_then(|()| writer.capture(session, draft))
                {
                    warn!("Particle transition draft failed on exit: {error}");
                }
            } else {
                warn!("Particle transition draft failed on exit: {error}");
            }
        }
    }
    // A project may change after Update but before the exit message is handled.
    // Preserve the previous session before allocating an ID for the new one.
    if (capture.previous_active || host.active) && capture.handle != state.current_project {
        if let Some(writer) = capture.writer.as_ref()
            && let Err(error) = writer.flush(Duration::from_secs(2))
        {
            warn!("Particle transition draft flush failed: {error}");
        }
        if let Some(asset) = capture
            .handle
            .as_ref()
            .and_then(|handle| assets.get(handle))
        {
            match crate::project::serialize_snapshot(asset) {
                Ok(snapshot)
                    if (capture.previous_dirty || capture.baseline.as_ref() != Some(&snapshot))
                        && capture.last_queued.as_ref() != Some(&snapshot) =>
                {
                    if let Some(writer) = capture.writer.as_ref()
                        && let Err(error) = writer.capture(
                            capture.session,
                            Draft::new(capture.previous_path.clone(), snapshot),
                        )
                    {
                        warn!("Outgoing particle draft could not be queued on exit: {error}");
                    }
                }
                Err(error) => {
                    warn!("Outgoing particle draft could not be serialized on exit: {error}")
                }
                _ => {}
            }
        }
        capture.session = capture.session.wrapping_add(1);
        capture.handle = state.current_project.clone();
    }
    if let Some(writer) = capture.writer.as_ref()
        && let Err(error) = writer.flush(Duration::from_secs(2))
    {
        warn!("Particle transition draft flush failed: {error}");
    }
    if dirty.has_unsaved_changes
        && capture.handle == state.current_project
        && let Some(asset) = state
            .current_project
            .as_ref()
            .and_then(|handle| assets.get(handle))
        && let Ok(snapshot) = crate::project::serialize_snapshot(asset)
        && let Some(writer) = capture.writer.as_ref()
        && let Err(error) = writer.capture(
            capture.session,
            Draft::new(state.current_project_path.clone(), snapshot),
        )
    {
        warn!("Particle draft could not be queued on exit: {error}");
    }
    if let Some(writer) = capture.writer.as_ref() {
        if let Err(error) = writer.flush(Duration::from_secs(2)) {
            warn!("Particle draft flush failed: {error}");
        }
        while let Some(result) = writer.try_result() {
            if let Err(error) = result.result {
                warn!("Particle draft write failed on exit: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::prelude::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Resource)]
    struct SwitchOnExit {
        outgoing: Handle<ParticlesAsset>,
        incoming: Handle<ParticlesAsset>,
    }

    fn switch_after_update(
        request: Option<Res<SwitchOnExit>>,
        mut assets: ResMut<Assets<ParticlesAsset>>,
        mut state: ResMut<EditorState>,
        mut dirty: ResMut<DirtyState>,
        mut exits: MessageWriter<bevy::app::AppExit>,
        mut commands: Commands,
    ) {
        let Some(request) = request else { return };
        assets.get_mut(&request.outgoing).unwrap().name = "Outgoing modified".into();
        state.current_project = Some(request.incoming.clone());
        state.current_project_path = None;
        dirty.has_unsaved_changes = true;
        exits.write(bevy::app::AppExit::Success);
        commands.remove_resource::<SwitchOnExit>();
    }

    #[test]
    fn exit_after_same_frame_project_switch_preserves_both_projects() {
        let dir = std::env::temp_dir().join(format!(
            "theia-capture-switch-exit-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let store = DraftStore::new(dir.join("drafts"));
        let mut app = App::new();
        app.insert_non_send(DraftCapture::new(store.clone()))
            .init_resource::<crate::hosting::AuthoringHost>()
            .init_resource::<EditorState>()
            .init_resource::<DirtyState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .add_message::<bevy::app::AppExit>()
            .add_systems(Update, capture_drafts)
            .add_systems(PostUpdate, switch_after_update)
            .add_systems(Last, capture_on_exit);
        let mut assets = app.world_mut().resource_mut::<Assets<ParticlesAsset>>();
        let outgoing = assets.add(ParticlesAsset::new(
            "Outgoing".into(),
            ParticlesDimension::D3,
            default(),
            vec![],
            vec![],
            false,
            default(),
        ));
        let incoming = assets.add(ParticlesAsset::new(
            "Incoming".into(),
            ParticlesDimension::D3,
            default(),
            vec![],
            vec![],
            false,
            default(),
        ));
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = Some(outgoing.clone());
        app.update(); // Establish the clean outgoing baseline.
        app.insert_resource(SwitchOnExit { outgoing, incoming });
        app.update(); // Switch and request exit in PostUpdate, after periodic capture.
        let mut names: Vec<_> = store
            .list()
            .unwrap()
            .iter()
            .map(|id| store.recover(id).unwrap().asset.name)
            .collect();
        names.sort();
        assert_eq!(names, ["Incoming", "Outgoing modified"]);
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exit_retries_failed_outgoing_snapshot_after_storage_recovers() {
        use bevy::ecs::system::RunSystemOnce;
        let dir = std::env::temp_dir().join(format!(
            "theia-capture-retry-exit-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let draft_dir = dir.join("blocked");
        std::fs::write(&draft_dir, b"block directory creation").unwrap();
        let store = DraftStore::new(draft_dir.clone());
        let mut app = App::new();
        app.insert_non_send(DraftCapture::new(store.clone()))
            .init_resource::<crate::hosting::AuthoringHost>()
            .init_resource::<EditorState>()
            .init_resource::<DirtyState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .add_message::<bevy::app::AppExit>()
            .add_systems(Update, capture_drafts);
        let mut assets = app.world_mut().resource_mut::<Assets<ParticlesAsset>>();
        let outgoing = assets.add(ParticlesAsset::new(
            "Retry outgoing".into(),
            ParticlesDimension::D3,
            default(),
            vec![],
            vec![],
            false,
            default(),
        ));
        let incoming = assets.add(ParticlesAsset::new(
            "Incoming".into(),
            ParticlesDimension::D3,
            default(),
            vec![],
            vec![],
            false,
            default(),
        ));
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = Some(outgoing);
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = true;
        app.update(); // Queue a draft while its directory is blocked.
        app.world()
            .non_send::<DraftCapture>()
            .writer
            .as_ref()
            .unwrap()
            .flush(Duration::from_secs(10))
            .unwrap();
        std::fs::remove_file(&draft_dir).unwrap();
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = Some(incoming);
        app.world_mut().write_message(bevy::app::AppExit::Success);
        app.world_mut().run_system_once(capture_on_exit).unwrap();
        let mut names: Vec<_> = store
            .list()
            .unwrap()
            .iter()
            .map(|id| store.recover(id).unwrap().asset.name)
            .collect();
        names.sort();
        assert_eq!(names, ["Incoming", "Retry outgoing"]);
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }

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
            .add_message::<bevy::app::AppExit>()
            .add_systems(Update, capture_drafts)
            .add_systems(Last, capture_on_exit);
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
        let active = app
            .world()
            .resource::<EditorState>()
            .current_project
            .clone()
            .unwrap();
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&active)
            .unwrap()
            .name = "At exit".into();
        app.world_mut().write_message(bevy::app::AppExit::Success);
        app.update();
        assert!(
            store
                .list()
                .unwrap()
                .iter()
                .any(|id| store.recover(id).unwrap().asset.name == "At exit")
        );
        assert_eq!(std::fs::read(source).unwrap(), b"original");
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
