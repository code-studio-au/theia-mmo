use std::path::PathBuf;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "--replace-all")) {
        return Err(
            "Usage: publish_particle_chunks CHUNK_WORLD OVERLAY_DRAFT NEW_OUTPUT [--replace-all]"
                .into(),
        );
    }
    let publish = if args.len() == 4 {
        theia_vfx::chunk_overlay::publish::replace_all
    } else {
        theia_vfx::chunk_overlay::publish::run
    };
    publish(
        &PathBuf::from(&args[0]),
        &PathBuf::from(&args[1]),
        &PathBuf::from(&args[2]),
    )
}
