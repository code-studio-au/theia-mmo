//! Offline admission/inspection of the first development ability-data contract.
use std::{fs::File, io::Read};
use theia_core::{
    abilities::{Catalog, MAX_CATALOG_BYTES},
    time::TickRate,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("Expected CATALOG.json [20|30|60]")?;
    let hz = args
        .next()
        .map(|v| {
            v.to_str()
                .ok_or("Invalid tick rate")?
                .parse::<u32>()
                .map_err(|_| "Invalid tick rate")
        })
        .transpose()?
        .unwrap_or(30);
    let rate = TickRate::from_hz(hz).ok_or("Tick rate must be 20, 30 or 60")?;
    if args.next().is_some() {
        return Err("Expected CATALOG.json [20|30|60]".into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let catalog = Catalog::from_json(&bytes)?;
    let abilities: Vec<_> = catalog.abilities().iter().map(|ability| {
        let timing = catalog.timing(ability.id().as_str(), rate).unwrap();
        serde_json::json!({"id":ability.id().as_str(),"range_metres":ability.range_metres(),
            "cast_while_moving":ability.cast_while_moving(),"uses_global_cooldown":ability.uses_global_cooldown(),
            "cast_ticks":timing.cast_ticks,"cooldown_ticks":timing.cooldown_ticks,
            "global_cooldown_ticks":timing.global_cooldown_ticks,"queue_window_ticks":timing.queue_window_ticks})
    }).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"schema":1,"tick_hz":hz,"abilities":abilities})
        )?
    );
    Ok(())
}
