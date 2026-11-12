//! Offline gameplay validation CLI; shared implementation is also used by the editor.
use std::path::PathBuf;
use theia_network::{Result, gameplay_bundle::validate};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or("Expected gameplay manifest JSON path")?);
    if args.next().is_some() {
        return Err("Unexpected gameplay validator arguments".into());
    }
    println!("{}", serde_json::to_string_pretty(&validate(&path)?)?);
    Ok(())
}
