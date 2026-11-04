//! Offline geometry check only; never admits a World or awards quest credit.
use std::{io::Read, path::PathBuf, process::ExitCode};
use theia_core::quests::areas::{Catalog, MAX_CATALOG_BYTES};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or("Use validate_quest_areas <areas.json>")?);
    if args.next().is_some() {
        return Err("Use validate_quest_areas <areas.json>".into());
    }
    let metadata = std::fs::symlink_metadata(&path)?;
    if !metadata.is_file() || metadata.len() > MAX_CATALOG_BYTES as u64 {
        return Err("Quest areas require a regular file within the 16-KiB budget".into());
    }
    let file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err("Quest area input changed from a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let catalog = Catalog::from_json(&bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema":1,"catalogue":catalog.id().as_str(),"revision":catalog.revision(),
            "areas":catalog.areas().map(|a|a.id().as_str()).collect::<Vec<_>>(),
            "input_bytes":bytes.len(),"geometry_validated":true,
            "world_binding_validated":false,"quest_credit_implemented":false
        })
    );
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Quest area validation failed: {error}");
            ExitCode::FAILURE
        }
    }
}
