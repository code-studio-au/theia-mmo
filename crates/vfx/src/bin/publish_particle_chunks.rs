use std::path::PathBuf;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: publish_particle_chunks CHUNK_WORLD OVERLAY_DRAFT NEW_OUTPUT".into());
    }
    theia_vfx::chunk_overlay::publish::run(
        &PathBuf::from(&args[0]),
        &PathBuf::from(&args[1]),
        &PathBuf::from(&args[2]),
    )
}
