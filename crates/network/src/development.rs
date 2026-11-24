//! Explicit local-owner terrain walkthrough controls, never player wire commands.
use crate::{Result, prediction_wire::Corrected, renet_runtime::WorldControl};
use std::{io::Read, path::Path};
pub const SPEEDS: [u8; 5] = [1, 2, 4, 8, 16];
pub const CORRECTION_TAG: &[u8; 4] = b"DT03";
pub const ENV: &str = "THEIA_DEVELOPER_SPEED_FILE";
pub fn valid(speed: u8) -> bool {
    SPEEDS.contains(&speed)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: u8,
    pub multiplier: u8,
    #[serde(default)]
    pub reset_nonce: u64,
}
pub fn read(path: &Path) -> Result<Request> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(129)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 128 {
        return Err("Developer request exceeds 128 bytes".into());
    }
    let value: Request = serde_json::from_slice(&bytes)?;
    if value.schema != 1 || !valid(value.multiplier) {
        return Err("Invalid developer speed".into());
    }
    Ok(value)
}
pub fn encode(mut value: Corrected) -> Result<Vec<u8>> {
    let speed = value.speed;
    if !valid(speed) {
        return Err("Invalid developer speed correction".into());
    }
    value.speed = 1;
    let mut bytes = crate::classic_wire::encode_correction(value)?;
    bytes[..4].copy_from_slice(CORRECTION_TAG);
    bytes.push(speed);
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Corrected> {
    if bytes.len() != crate::classic_wire::CORRECTION_BYTES + 1
        || !bytes.starts_with(CORRECTION_TAG)
    {
        return Err("Invalid developer correction envelope".into());
    }
    let speed = *bytes.last().unwrap();
    if !valid(speed) {
        return Err("Invalid developer speed correction".into());
    }
    let mut base = bytes[..bytes.len() - 1].to_vec();
    base[..4].copy_from_slice(crate::classic_wire::CORRECTION_TAG);
    let mut value = crate::classic_wire::decode_correction(&base)?;
    value.speed = speed;
    Ok(value)
}
/// One bounded read in flight, outside the simulation and render threads.
pub async fn watch(
    path: std::path::PathBuf,
    control: WorldControl,
    mut stop: tokio::sync::watch::Receiver<bool>,
) {
    let mut last = None;
    let mut last_reset = 0;
    loop {
        tokio::select! {
            _=stop.changed()=>break,
            _=tokio::time::sleep(std::time::Duration::from_millis(100))=>{}
        }
        let path = path.clone();
        if let Ok(Ok(request)) = tokio::task::spawn_blocking(move || read(&path)).await {
            if last != Some(request.multiplier) {
                match control.development_speed(request.multiplier).await {
                    Ok(true) => last = Some(request.multiplier),
                    _ => break,
                }
            }
            if request.reset_nonce > last_reset
                && matches!(control.development_reset().await, Ok(true))
            {
                last_reset = request.reset_nonce;
            }
        }
    }
}
