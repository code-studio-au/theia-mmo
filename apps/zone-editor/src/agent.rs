//! Explicit local agent handoff; receipt success means an editor edit, not a save
//! or authoritative server publication. All I/O stays outside the fixed update.
use super::*;
use theia_zone::mailbox::Mailbox;
#[derive(Resource)]
pub(super) struct AgentMailbox {
    inner: Mailbox,
    elapsed: f32,
}
impl AgentMailbox {
    pub fn open(path: &std::path::Path) -> Result<Self, String> {
        Ok(Self {
            inner: Mailbox::open(path)?,
            elapsed: 0.2,
        })
    }
}
pub(super) fn poll(
    mailbox: Option<ResMut<AgentMailbox>>,
    time: Res<Time>,
    mut session: ResMut<Session>,
) {
    let Some(mut mailbox) = mailbox else {
        return;
    };
    mailbox.elapsed += time.delta_secs();
    if mailbox.elapsed < 0.2 {
        return;
    }
    mailbox.elapsed = 0.;
    let busy = session.drag.is_some()
        || session.marquee.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
        || session.play.is_some()
        || session.has_modal();
    let previous = session.editor.revision();
    let points_before = session.selected_path_len();
    match mailbox.inner.poll(&mut session.editor, busy) {
        Ok(Some(receipt)) => {
            session.status = if receipt.accepted {
                format!(
                    "Agent request {:?} accepted at revision {}",
                    receipt.request_id, receipt.revision
                )
            } else {
                format!(
                    "Agent request refused: {}",
                    receipt
                        .error
                        .unwrap_or_default()
                        .chars()
                        .take(240)
                        .collect::<String>()
                )
            };
        }
        Ok(None) => {}
        Err(error) => {
            session.status = format!(
                "Agent mailbox: {}",
                error.chars().take(240).collect::<String>()
            )
        }
    }
    if session.editor.revision() != previous {
        session.reconcile_after_edit(points_before);
    }
}
