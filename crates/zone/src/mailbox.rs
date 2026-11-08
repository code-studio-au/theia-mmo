//! Opt-in, single-slot local agent handoff. Session matching rejects stale files;
//! it is not authentication or a gameplay/network protocol. Call outside simulation.
use super::{Command, Editor, MAX_DOCUMENT_BYTES, Result, require};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub session: String,
    pub request_id: u64,
    pub command: Command,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub session: String,
    pub request_id: Option<u64>,
    pub accepted: bool,
    /// The revision when this request was decided. Exact retries retain it.
    pub revision: u64,
    pub error: Option<String>,
}

pub struct Mailbox {
    directory: PathBuf,
    session: String,
    last: Option<(Request, Receipt)>,
    pending_receipt: Option<Receipt>,
}
impl Mailbox {
    pub fn open(directory: &Path) -> Result<Self> {
        fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        // Unique per process opening, without introducing random/auth dependencies.
        let session = format!("{}-{nonce}", std::process::id());
        Ok(Self {
            directory: directory.into(),
            session,
            last: None,
            pending_receipt: None,
        })
    }
    pub fn session(&self) -> &str {
        &self.session
    }
    fn receipt(&self, id: Option<u64>, editor: &Editor, result: Result<()>) -> Receipt {
        Receipt {
            session: self.session.clone(),
            request_id: id,
            accepted: result.is_ok(),
            revision: editor.revision(),
            error: result.err(),
        }
    }
    fn decide(&mut self, bytes: &[u8], editor: &mut Editor, busy: bool) -> Receipt {
        let parsed = (|| {
            require(
                bytes.len() as u64 <= MAX_DOCUMENT_BYTES,
                "Request file too large",
            )?;
            serde_json::from_slice::<Request>(bytes).map_err(|e| e.to_string())
        })();
        let request = match parsed {
            Ok(request) => request,
            Err(error) => return self.receipt(None, editor, Err(error)),
        };
        if request.session != self.session {
            return self.receipt(
                Some(request.request_id),
                editor,
                Err("Wrong editor session".into()),
            );
        }
        if request.request_id == 0 {
            return self.receipt(Some(0), editor, Err("Request ID must be positive".into()));
        }
        if let Some((previous, receipt)) = &self.last {
            if request == *previous {
                return receipt.clone();
            }
            if request.request_id <= previous.request_id {
                return self.receipt(
                    Some(request.request_id),
                    editor,
                    Err("Reused or older request ID".into()),
                );
            }
        }
        let result = if busy {
            Err("Editor busy: finish or cancel the authoring draft, or leave playtest".into())
        } else {
            editor.execute(request.command.clone())
        };
        let receipt = self.receipt(Some(request.request_id), editor, result);
        self.last = Some((request, receipt.clone()));
        receipt
    }
    fn write_json(&self, name: &str, value: &impl Serialize) -> Result<()> {
        let temp = self.directory.join(format!(".{name}.{}.tmp", self.session));
        let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
        fs::write(&temp, bytes).map_err(|e| e.to_string())?;
        fs::rename(&temp, self.directory.join(name)).map_err(|e| e.to_string())
    }
    fn publish(&mut self, editor: &Editor, busy: bool) -> Result<()> {
        self.write_json(
            "state.json",
            &serde_json::json!({
                "schema":1,"session":self.session,"revision":editor.revision(),
                "history":editor.history(),"busy":busy,"document":editor.zone(),
            }),
        )?;
        if let Some(receipt) = &self.pending_receipt {
            self.write_json("receipt.json", receipt)?;
        }
        self.pending_receipt = None;
        Ok(())
    }
    /// Poll at most one request, and publish current state. A pending publication
    /// failure blocks consuming another request until its receipt can be written.
    /// Producer writes a temporary file then renames it to request.json, and waits
    /// for the matching session/request receipt before submitting another request.
    pub fn poll(&mut self, editor: &mut Editor, busy: bool) -> Result<Option<Receipt>> {
        self.publish(editor, busy)?;
        let request_path = self.directory.join("request.json");
        let processing = self
            .directory
            .join(format!(".processing-{}.json", self.session));
        match fs::rename(&request_path, &processing) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
        }
        let read = (|| {
            let mut bytes = Vec::new();
            fs::File::open(&processing)
                .map_err(|e| e.to_string())?
                .take(MAX_DOCUMENT_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            Ok(bytes)
        })();
        let receipt = match read {
            Ok(bytes) => self.decide(&bytes, editor, busy),
            Err(error) => self.receipt(None, editor, Err(error)),
        };
        // Only our claimed file is removed; a newly submitted request is untouched.
        self.pending_receipt = Some(receipt.clone());
        fs::remove_file(processing).map_err(|e| e.to_string())?;
        self.publish(editor, busy)?;
        Ok(Some(receipt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edit, Zone};
    fn request(mailbox: &Mailbox, id: u64, revision: u64) -> Request {
        Request {
            session: mailbox.session().into(),
            request_id: id,
            command: Command::Apply {
                expected_revision: revision,
                edits: vec![Edit::SetSpawn {
                    position: [2., 0.1, 3.],
                }],
            },
        }
    }
    fn submit(
        mailbox: &mut Mailbox,
        editor: &mut Editor,
        request: &Request,
        busy: bool,
    ) -> Receipt {
        mailbox.decide(&serde_json::to_vec(request).unwrap(), editor, busy)
    }
    #[test]
    fn exact_retry_does_not_reapply_or_undo_later_visual_edits() {
        let dir = std::env::temp_dir().join(format!("theia-mailbox-retry-{}", std::process::id()));
        let mut mailbox = Mailbox::open(&dir).unwrap();
        let mut editor = Editor::new(Zone::default()).unwrap();
        let req = request(&mailbox, 1, 0);
        let receipt = submit(&mut mailbox, &mut editor, &req, false);
        assert!(receipt.accepted);
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        assert_eq!(submit(&mut mailbox, &mut editor, &req, false), receipt);
        assert_eq!(editor.revision(), 2);
        assert_eq!(editor.zone(), &Zone::default());
        let mut reused = req.clone();
        reused.command = Command::Redo {
            expected_revision: 2,
        };
        assert!(!submit(&mut mailbox, &mut editor, &reused, false).accepted);
        let mut next = request(&mailbox, 2, 2);
        next.command = Command::Redo {
            expected_revision: 2,
        };
        assert!(submit(&mut mailbox, &mut editor, &next, false).accepted);
        assert_eq!(editor.revision(), 3);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn wrong_session_stale_busy_and_bad_payloads_preserve_state_and_history() {
        let dir =
            std::env::temp_dir().join(format!("theia-mailbox-refusal-{}", std::process::id()));
        let mut mailbox = Mailbox::open(&dir).unwrap();
        let mut editor = Editor::new(Zone::default()).unwrap();
        let req = request(&mailbox, 1, 0);
        let mut wrong = req.clone();
        wrong.session = "previous-run".into();
        assert!(!submit(&mut mailbox, &mut editor, &wrong, false).accepted);
        assert!(!submit(&mut mailbox, &mut editor, &req, true).accepted);
        assert!(
            !submit(&mut mailbox, &mut editor, &req, false).accepted,
            "Busy retry returns original refusal"
        );
        let stale = request(&mailbox, 2, 9);
        assert!(!submit(&mut mailbox, &mut editor, &stale, false).accepted);
        assert!(!mailbox.decide(b"{}", &mut editor, false).accepted);
        assert!(
            !mailbox
                .decide(
                    &vec![b' '; MAX_DOCUMENT_BYTES as usize + 1],
                    &mut editor,
                    false
                )
                .accepted
        );
        assert_eq!(editor.revision(), 0);
        assert_eq!(editor.history(), (0, 0));
        let valid = request(&mailbox, 3, 0);
        assert!(submit(&mut mailbox, &mut editor, &valid, false).accepted);
        assert_eq!(editor.history(), (1, 0));
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn real_file_handoff_publishes_matching_state_and_does_not_reconsume() {
        let dir = std::env::temp_dir().join(format!("theia-mailbox-file-{}", std::process::id()));
        let mut mailbox = Mailbox::open(&dir).unwrap();
        let mut editor = Editor::new(Zone::default()).unwrap();
        assert!(mailbox.poll(&mut editor, false).unwrap().is_none());
        let state: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("state.json")).unwrap()).unwrap();
        assert_eq!(state["revision"], 0);
        let req = request(&mailbox, 1, 0);
        fs::write(dir.join("agent.tmp"), serde_json::to_vec(&req).unwrap()).unwrap();
        fs::rename(dir.join("agent.tmp"), dir.join("request.json")).unwrap();
        assert!(mailbox.poll(&mut editor, false).unwrap().unwrap().accepted);
        let receipt: Receipt =
            serde_json::from_slice(&fs::read(dir.join("receipt.json")).unwrap()).unwrap();
        assert_eq!(receipt.revision, 1);
        assert_eq!(receipt.session, mailbox.session());
        assert!(!dir.join("request.json").exists());
        assert!(mailbox.poll(&mut editor, true).unwrap().is_none());
        assert_eq!(editor.revision(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn receipt_publication_failure_retries_without_reapplying() {
        let dir =
            std::env::temp_dir().join(format!("theia-mailbox-publication-{}", std::process::id()));
        let mut mailbox = Mailbox::open(&dir).unwrap();
        let mut editor = Editor::new(Zone::default()).unwrap();
        let req = request(&mailbox, 1, 0);
        fs::write(dir.join("request.json"), serde_json::to_vec(&req).unwrap()).unwrap();
        fs::create_dir(dir.join("receipt.json")).unwrap();
        assert!(mailbox.poll(&mut editor, false).is_err());
        assert_eq!(editor.revision(), 1);
        assert_eq!(editor.history(), (1, 0));
        fs::remove_dir(dir.join("receipt.json")).unwrap();
        assert!(mailbox.poll(&mut editor, false).unwrap().is_none());
        let receipt: Receipt =
            serde_json::from_slice(&fs::read(dir.join("receipt.json")).unwrap()).unwrap();
        assert!(receipt.accepted);
        assert_eq!(receipt.revision, 1);
        assert_eq!(editor.revision(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
}
