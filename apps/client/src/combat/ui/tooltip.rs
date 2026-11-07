//! Authored eligibility/timing only: never predicts damage, success or readiness.
use theia_core::abilities::Ability;
fn number(value: f64) -> String {
    if value > 0. && value < 0.001 {
        format!("{value:.2e}")
    } else {
        value.to_string()
    }
}
pub(super) fn describe(ability: &Ability, key: &str, state: &str) -> String {
    let name = super::ability_name(ability.id().as_str());
    let maximum = number(ability.range_metres());
    let range = match ability.combat_rules().map(|r| r.minimum_range_metres()) {
        Some(minimum) if minimum > 0. => format!("{}–{maximum} m range", number(minimum)),
        _ => format!("{maximum} m range"),
    };
    let cost = if ability.mana_cost() > 0 {
        format!("{} mana · ", ability.mana_cost())
    } else {
        String::new()
    };
    let cast = ability.cast_duration();
    let cast = if cast.is_zero() {
        "Instant".to_owned()
    } else {
        format!("{} s cast", number(cast.as_secs_f64()))
    };
    let cooldown = ability.cooldown_duration();
    let cooldown = if cooldown.is_zero() {
        String::new()
    } else {
        format!(" · {} s cooldown", number(cooldown.as_secs_f64()))
    };
    let mut lines = vec![
        format!("[{key}] {name}"),
        format!("{cost}{range}"),
        format!("{cast}{cooldown}"),
    ];
    if ability.uses_global_cooldown() {
        lines.push("Uses global cooldown".to_owned());
    }
    if !ability.cast_duration().is_zero() && !ability.cast_while_moving() {
        lines.push("Stand still while casting".to_owned());
    }
    if ability
        .combat_rules()
        .is_some_and(|rules| rules.arc().is_some())
    {
        lines.push("Must face the target".to_owned());
    }
    lines.push(state.to_owned());
    lines.join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    const DATA: &[u8] =
        include_bytes!("../../../../../content/gameplay/development-classic-abilities.json");
    #[test]
    fn normal_spell_and_instant_melee_show_configured_metric_requirements() {
        let catalog = theia_core::abilities::Catalog::from_json(DATA).unwrap();
        let spell = describe(&catalog.abilities()[0], "1", "Disconnected / loading");
        assert_eq!(
            spell,
            "[1] Arcane Bolt\n25 mana · 30 m range\n0.5 s cast · 2 s cooldown\nUses global cooldown\nStand still while casting\nMust face the target\nDisconnected / loading"
        );
        let melee = describe(&catalog.abilities()[1], "2", "Face the target");
        assert_eq!(
            melee,
            "[2] Sword Slash\n3 m range\nInstant · 1.5 s cooldown\nUses global cooldown\nMust face the target\nFace the target"
        );
        assert!(!melee.contains("mana"));
        assert!(
            !spell.contains("damage"),
            "Additional server damage profiles can override authored base effects"
        );
    }
    #[test]
    fn minimum_range_and_fractional_durations_use_authored_data_not_tick_rounding() {
        let mut raw: serde_json::Value = serde_json::from_slice(DATA).unwrap();
        let ability = &mut raw["abilities"][0];
        ability["range_metres"] = 12.75.into();
        ability["combat"]["minimum_range_metres"] = 5.125.into();
        ability["cast_ms"] = 1001.into();
        ability["cooldown_ms"] = 0.into();
        ability["cast_while_moving"] = true.into();
        ability["uses_global_cooldown"] = false.into();
        ability["combat"]["facing"] = serde_json::json!({"kind":"any"});
        let catalog =
            theia_core::abilities::Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
        let ability = &catalog.abilities()[0];
        let text = describe(ability, "=", "Select a target with Tab");
        assert!(text.contains("5.125–12.75 m range\n1.001 s cast\n"));
        assert!(
            !text.contains("Stand still")
                && !text.contains("Must face")
                && !text.contains("cooldown")
        );
        assert_eq!(
            catalog
                .timing(
                    ability.id().as_str(),
                    theia_core::time::TickRate::from_hz(30).unwrap()
                )
                .unwrap()
                .cast_ticks,
            31
        );
        assert_eq!(number(0.000001), "1.00e-6");
    }
}
