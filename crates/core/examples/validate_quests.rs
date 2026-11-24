//! Offline structural validation, not live quest admission or target existence.
use std::{fs::File, io::Read};
use theia_core::quests::catalog::{Catalog, MAX_CATALOG_BYTES};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("Expected QUESTS.json")?;
    if args.next().is_some() {
        return Err("Expected QUESTS.json".into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let catalog = Catalog::from_json(&bytes)?;
    let quests:Vec<_>=catalog.quests().map(|q|serde_json::json!({"id":q.id().as_str(),"version":q.version(),"stages":q.stages().count(),"objectives":q.stages().map(|s|s.objectives().len()).sum::<usize>()})).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"validated":true,"scope":"offline structure only; no world target or persistence admission","schema":catalog.schema(),"id":catalog.id().as_str(),"revision":catalog.revision(),"quests":quests})
        )?
    );
    Ok(())
}
