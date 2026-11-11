use std::path::PathBuf;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 6 {
        return Err("Usage: draft_particle_chunks SOURCE_WORLD CHUNK_WORLD NEW_OUTPUT ORIGIN_X ORIGIN_Y ORIGIN_Z".into());
    }
    let mut origin = [0.; 3];
    for (out, value) in origin.iter_mut().zip(&args[3..]) {
        *out = value
            .to_str()
            .ok_or("Origin must be UTF-8")?
            .parse()
            .map_err(|_| "Origin must be numeric")?;
    }
    theia_vfx::chunk_overlay::draft::run(
        &PathBuf::from(&args[0]),
        &PathBuf::from(&args[1]),
        &PathBuf::from(&args[2]),
        origin,
    )
}
