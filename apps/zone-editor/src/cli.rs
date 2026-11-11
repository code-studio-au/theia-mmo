use std::{
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};
use theia_zone::{Command, Editor, MAX_DOCUMENT_BYTES, Result, Zone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureView {
    Spawn,
    PropertyActions,
    Particle,
    ParticlePreview,
}

#[derive(Default)]
pub struct Options {
    pub zone: Option<PathBuf>,
    pub commands: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub save_path: Option<PathBuf>,
    pub headless: bool,
    pub check_playtest: bool,
    pub capture: Option<PathBuf>,
    pub capture_view: Option<CaptureView>,
    pub report: Option<PathBuf>,
    pub mailbox: Option<PathBuf>,
    pub export: Option<PathBuf>,
    pub content_id: Option<String>,
    pub revision: Option<u32>,
}
impl Options {
    pub fn parse() -> Result<Self> {
        let mut result = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--zone" => result.zone = Some(args.next().ok_or("--zone needs a path")?.into()),
                "--commands" => {
                    result.commands = Some(
                        args.next()
                            .ok_or("--commands needs a JSONL path or - for stdin")?
                            .into(),
                    )
                }
                "--save-path" => {
                    result.save_path = Some(args.next().ok_or("--save-path needs a path")?.into());
                }
                "--output" => {
                    result.output = Some(args.next().ok_or("--output needs a path")?.into())
                }
                "--capture" => {
                    result.capture = Some(args.next().ok_or("--capture needs a PNG path")?.into())
                }
                "--capture-view" => {
                    result.capture_view = Some(match args.next().as_deref() {
                        Some("spawn") => CaptureView::Spawn,
                        Some("property-actions") => CaptureView::PropertyActions,
                        Some("particle") => CaptureView::Particle,
                        Some("particle-preview") => CaptureView::ParticlePreview,
                        _ => {
                            return Err(
                                "--capture-view requires spawn, property-actions, particle or particle-preview"
                                    .into(),
                            );
                        }
                    });
                }
                "--report" => {
                    result.report = Some(args.next().ok_or("--report needs a path")?.into())
                }
                "--mailbox" => {
                    result.mailbox = Some(args.next().ok_or("--mailbox needs a directory")?.into())
                }
                "--export" => {
                    result.export = Some(
                        args.next()
                            .ok_or("--export needs a fresh directory")?
                            .into(),
                    )
                }
                "--content-id" => {
                    result.content_id = Some(args.next().ok_or("--content-id needs a logical ID")?)
                }
                "--revision" => {
                    result.revision = Some(
                        args.next()
                            .ok_or("--revision needs a nonzero number")?
                            .parse()
                            .map_err(|_| "Invalid content revision")?,
                    )
                }
                "--headless" => result.headless = true,
                "--check-playtest" => {
                    result.check_playtest = true;
                    result.headless = true;
                }
                "--help" => {
                    println!(
                        "theia-zone-editor [--zone FILE] [--commands JSONL] [--output FILE] [--save-path FILE] [--headless] [--check-playtest] [--capture PNG] [--capture-view spawn|property-actions|particle|particle-preview] [--report JSON] [--mailbox DIR] [--export NEW_DIR --content-id ID --revision N]\n\nCommands use the same transaction/undo engine as the viewport. Use --commands - to read bounded JSONL from stdin through EOF, with the same byte/line limits as a command file. Headless mode validates and prints the final revision without opening a window. Check-playtest is read-only: it builds the actual offline collision scene, checks the requested capsule spawn and prints JSON without opening a window. It cannot be combined with --output or --export. Output writes only after every command succeeds. Save-path changes the later editor save target without writing at startup."
                    );
                    std::process::exit(0);
                }
                _ => return Err(format!("Unknown argument: {arg}")),
            }
        }
        if result.check_playtest && (result.output.is_some() || result.export.is_some()) {
            return Err("--check-playtest is read-only; omit --output and --export".into());
        }
        if result.export.is_some() {
            if result.content_id.is_none() || result.revision.is_none_or(|r| r == 0) {
                return Err("Export requires --content-id and positive --revision".into());
            }
            result.headless = true;
        } else if result.content_id.is_some() || result.revision.is_some() {
            return Err("Content identity options require --export".into());
        }
        if result.capture_view.is_some() && result.capture.is_none() {
            return Err("--capture-view requires --capture".into());
        }
        if result.headless
            && (result.capture.is_some() || result.report.is_some() || result.mailbox.is_some())
        {
            return Err("--capture, --report and --mailbox require a window".into());
        }
        Ok(result)
    }
    pub fn editor_path(&self) -> PathBuf {
        self.save_path
            .clone()
            .or_else(|| self.output.clone())
            .or_else(|| self.zone.clone())
            .unwrap_or_else(|| "build/zone-draft.json".into())
    }
    pub fn prepare(&self) -> Result<Editor> {
        let zone = if let Some(path) = &self.zone {
            Zone::load(path)?
        } else {
            Zone::default()
        };
        let mut editor = Editor::new(zone)?;
        if let Some(path) = &self.commands {
            let reader: Box<dyn BufRead> = if path == std::path::Path::new("-") {
                let mut bytes = Vec::new();
                std::io::stdin()
                    .take(MAX_DOCUMENT_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
                    return Err("Command stream too large".into());
                }
                Box::new(std::io::Cursor::new(bytes))
            } else {
                let bytes = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
                if bytes > MAX_DOCUMENT_BYTES {
                    return Err("Command file too large".into());
                }
                Box::new(BufReader::new(
                    std::fs::File::open(path)
                        .map_err(|e| e.to_string())?
                        .take(MAX_DOCUMENT_BYTES + 1),
                ))
            };
            let mut total = 0;
            for (index, line) in reader.lines().enumerate() {
                let line = line.map_err(|e| e.to_string())?;
                total += line.len() + 1;
                if total as u64 > MAX_DOCUMENT_BYTES || index >= 1024 {
                    return Err("Command budget exceeded".into());
                }
                let command: Command =
                    serde_json::from_str(&line).map_err(|e| format!("Line {}: {e}", index + 1))?;
                editor
                    .execute(command)
                    .map_err(|e| format!("Line {}: {e}", index + 1))?;
            }
        }
        if let Some(path) = &self.output {
            editor.zone().save(path)?;
        }
        Ok(editor)
    }
}

pub(super) fn summary(editor: &Editor) -> serde_json::Value {
    serde_json::json!({"revision":editor.revision(),"objects":editor.zone().objects.len(),"paths":editor.zone().paths.len(),"markers":editor.zone().markers.len(),"plants":editor.zone().plants.len(),"history":editor.history(),"valid":true})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_or_redirecting_editor_save_path_does_not_write_source_or_destination() {
        let root = std::env::temp_dir().join(format!(
            "theia-zone-cli-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let input = root.join("input.json");
        let target = root.join("draft.json");
        let blank = Options {
            save_path: Some(target.clone()),
            ..Options::default()
        };
        blank.prepare().unwrap();
        assert!(!target.exists());
        assert_eq!(blank.editor_path(), target);
        let bytes = serde_json::to_vec(&Zone::default()).unwrap();
        std::fs::write(&input, &bytes).unwrap();
        let options = Options {
            zone: Some(input.clone()),
            save_path: Some(target.clone()),
            ..Options::default()
        };
        options.prepare().unwrap();
        assert_eq!(std::fs::read(&input).unwrap(), bytes);
        assert!(!target.exists());
        assert_eq!(options.editor_path(), target);
        let ordinary = Options {
            zone: Some(input.clone()),
            ..Options::default()
        };
        assert_eq!(ordinary.editor_path(), input);
        std::fs::remove_dir_all(root).unwrap();
    }
}
