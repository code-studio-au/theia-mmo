use std::{
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};
use theia_zone::{Command, Editor, MAX_DOCUMENT_BYTES, Result, Zone};

#[derive(Default)]
pub struct Options {
    pub zone: Option<PathBuf>,
    pub commands: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub headless: bool,
    pub capture: Option<PathBuf>,
    pub report: Option<PathBuf>,
    pub mailbox: Option<PathBuf>,
}
impl Options {
    pub fn parse() -> Result<Self> {
        let mut result = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--zone" => result.zone = Some(args.next().ok_or("--zone needs a path")?.into()),
                "--commands" => {
                    result.commands =
                        Some(args.next().ok_or("--commands needs a JSONL path")?.into())
                }
                "--output" => {
                    result.output = Some(args.next().ok_or("--output needs a path")?.into())
                }
                "--capture" => {
                    result.capture = Some(args.next().ok_or("--capture needs a PNG path")?.into())
                }
                "--report" => {
                    result.report = Some(args.next().ok_or("--report needs a path")?.into())
                }
                "--mailbox" => {
                    result.mailbox = Some(args.next().ok_or("--mailbox needs a directory")?.into())
                }
                "--headless" => result.headless = true,
                "--help" => {
                    println!(
                        "theia-zone-editor [--zone FILE] [--commands JSONL] [--output FILE] [--headless] [--capture PNG] [--report JSON] [--mailbox DIR]\n\nCommands use the same transaction/undo engine as the viewport. Headless mode validates and prints the final revision without opening a window. Output writes only after every command succeeds."
                    );
                    std::process::exit(0);
                }
                _ => return Err(format!("Unknown argument: {arg}")),
            }
        }
        if result.headless
            && (result.capture.is_some() || result.report.is_some() || result.mailbox.is_some())
        {
            return Err("--capture, --report and --mailbox require a window".into());
        }
        Ok(result)
    }
    pub fn prepare(&self) -> Result<Editor> {
        let zone = if let Some(path) = &self.zone {
            Zone::load(path)?
        } else {
            Zone::default()
        };
        let mut editor = Editor::new(zone)?;
        if let Some(path) = &self.commands {
            let bytes = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
            if bytes > MAX_DOCUMENT_BYTES {
                return Err("Command file too large".into());
            }
            let reader = BufReader::new(
                std::fs::File::open(path)
                    .map_err(|e| e.to_string())?
                    .take(MAX_DOCUMENT_BYTES + 1),
            );
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
