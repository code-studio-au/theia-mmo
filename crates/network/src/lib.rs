//! Small local walking protocol, not an MMO service stack.
pub mod appearance_wire;
pub mod chunk_world;
pub mod chunks;
pub mod classic;
pub mod classic_wire;
pub mod client;
pub mod combat_membership_wire;
pub mod combat_wire;
pub mod commands;
pub mod content;
pub mod dialogue;
pub mod dialogue_wire;
pub mod dodge_input;
pub mod dodge_prediction;
pub mod dodge_prediction_wire;
pub mod dodging;
pub mod inventory_wire;
pub mod kinematic;
pub mod material_profile;
pub mod navigation;
pub mod physics;
pub mod player_life_wire;
pub mod prediction;
pub mod prediction_driver;
pub mod prediction_wire;
pub mod procedural;
pub mod renet_runtime;
pub mod renet_transport;
pub mod reward_wire;
pub mod saved_terrain;
pub mod server;
pub mod shared_images;
pub mod simulation;
mod static_shapes;
pub mod status_wire;
pub mod targeting;
pub mod targeting_wire;
pub mod terrain;
pub mod tls;
pub mod wire;
pub mod world;
pub mod world_bounds;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod progression_wire;

pub mod quest_journal;
pub mod quest_journal_wire;
pub mod quest_lifecycle_wire;
pub mod quest_wire;

pub mod vendor_wire;

#[cfg(test)]
#[path = "../tests/support/relay.rs"]
#[allow(dead_code)]
pub(crate) mod test_relay;
