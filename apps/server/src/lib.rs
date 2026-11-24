//! Headless server boundaries shared with owned qualification executables.
pub mod accounts;
pub mod admissions;
pub mod authenticated;
pub mod development;

#[cfg(test)]
mod quest_item_profiles;

#[cfg(test)]
mod quest_loot_profiles;

#[cfg(test)]
mod quest_document_profiles;

#[cfg(test)]
mod quest_item_reward_profiles;
