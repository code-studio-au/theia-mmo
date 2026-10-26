use std::path::PathBuf;
fn main() -> theia_network::Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("Expected bake|verify directory [revision]")?;
    let root = PathBuf::from(args.next().ok_or("Missing terrain directory")?);
    let revision = args.next().map(|v| v.parse()).transpose()?.unwrap_or(1);
    if args.next().is_some() {
        return Err("Unexpected terrain tool arguments".into());
    }
    match command.as_str() {
        "bake" => {
            if let Some(parent) = root.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            theia_network::saved_terrain::bake(&root, "theia-graybox-saved", revision)?;
        }
        "verify" => {}
        _ => return Err("Expected bake|verify".into()),
    }
    let store = theia_network::saved_terrain::Store::open(&root)?;
    store.verify()?;
    println!(
        "saved_terrain_verified identity={:?} stats={:?}",
        store.identity(),
        store.stats()
    );
    Ok(())
}
