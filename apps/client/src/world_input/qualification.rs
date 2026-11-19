//! Native qualification helpers emit ordinary Bevy pointer messages, never requests.
use bevy::{camera::primitives::Aabb, prelude::*};
pub(crate) fn guide_screen(world: &mut World) -> Option<Vec2> {
    let point = world
        .query_filtered::<&GlobalTransform, With<crate::dialogue::GuideVisual>>()
        .iter(world)
        .next()?
        .translation();
    project(world, point)
}
pub(crate) fn enemy_screen(
    world: &mut World,
    key: theia_core::identity::EntityKey,
) -> Option<Vec2> {
    let root = world
        .query::<(Entity, &crate::targeting::TargetVisual)>()
        .iter(world)
        .find(|(_, v)| v.key == key)?
        .0;
    let point = world
        .query_filtered::<(Entity, &Aabb, &GlobalTransform, &InheritedVisibility), With<Mesh3d>>()
        .iter(world)
        .find(|(entity, _, _, visible)| {
            if !visible.get() {
                return false;
            }
            let mut e = *entity;
            for _ in 0..32 {
                if e == root {
                    return true;
                }
                let Some(p) = world.get::<ChildOf>(e) else {
                    return false;
                };
                e = p.parent();
            }
            false
        })
        .map(|(_, aabb, t, _)| t.transform_point(Vec3::from(aabb.center)))?;
    project(world, point)
}
fn project(world: &mut World, point: Vec3) -> Option<Vec2> {
    let mut query = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    let (camera, transform) = query.single(world).ok()?;
    let screen = camera.world_to_viewport(transform, point).ok()?;
    let size = camera.logical_viewport_size()?;
    (screen.x >= 0. && screen.y >= 0. && screen.x < size.x && screen.y < size.y).then_some(screen)
}
pub(crate) fn mouse(
    world: &mut World,
    position: Vec2,
    button: bevy::picking::pointer::PointerButton,
    release: bool,
) {
    use bevy::picking::pointer::{Location, PointerAction, PointerId, PointerInput};
    let Some(window) = world
        .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
        .iter(world)
        .next()
    else {
        return;
    };
    let location = Location {
        target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
            .normalize(Some(window))
            .unwrap(),
        position,
    };
    world.write_message(PointerInput::new(
        PointerId::Mouse,
        location.clone(),
        PointerAction::Move { delta: Vec2::ZERO },
    ));
    world.write_message(PointerInput::new(
        PointerId::Mouse,
        location,
        if release {
            PointerAction::Release(button)
        } else {
            PointerAction::Press(button)
        },
    ));
}

#[derive(Resource)]
struct OsTrace {
    output: std::path::PathBuf,
    written: std::time::Instant,
}
pub(crate) fn install(app: &mut App) {
    if let Some(output) = std::env::var_os("THEIA_WORLD_INPUT_OS_QUALIFY") {
        app.insert_resource(OsTrace {
            output: output.into(),
            written: std::time::Instant::now(),
        })
        .add_systems(Last, os_trace);
    }
}
fn widget_position(world: &mut World, equip: bool, item: &str) -> Option<Vec2> {
    if !world
        .get_resource::<crate::inventory::Ui>()
        .is_some_and(|ui| ui.open)
    {
        return None;
    }
    if equip {
        world.query_filtered::<(&ComputedNode,&bevy::ui::UiGlobalTransform),With<crate::inventory::EquipButton>>()
            .iter(world).next().map(|(node,t)|t.affine().translation*node.inverse_scale_factor())
    } else {
        let state = world.resource::<crate::network::Network>().state();
        let slot = state
            .inventory
            .as_ref()?
            .bag
            .slots()
            .iter()
            .position(|stack| {
                stack
                    .as_ref()
                    .is_some_and(|stack| stack.item().as_str() == item)
            })?;
        world
            .query::<(
                &crate::inventory::SlotButton,
                &ComputedNode,
                &bevy::ui::UiGlobalTransform,
            )>()
            .iter(world)
            .find(|(button, _, _)| usize::from(button.0) == slot)
            .map(|(_, node, t)| t.affine().translation * node.inverse_scale_factor())
    }
}
fn inventory_widget<C: Component>(world: &mut World) -> Option<[f32; 2]> {
    world
        .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<C>>()
        .iter(world)
        .next()
        .map(|(node, t)| (t.affine().translation * node.inverse_scale_factor()).to_array())
}
fn inventory_layout(world: &mut World) -> serde_json::Value {
    if !world
        .get_resource::<crate::inventory::Ui>()
        .is_some_and(|ui| ui.open)
    {
        return serde_json::Value::Null;
    }
    let mut slots = vec![None; theia_core::inventory::SLOT_COUNT];
    for (button, node, t) in world
        .query::<(
            &crate::inventory::SlotButton,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
        )>()
        .iter(world)
    {
        slots[usize::from(button.0)] =
            Some((t.affine().translation * node.inverse_scale_factor()).to_array());
    }
    serde_json::json!({"slots":slots,"split":inventory_widget::<crate::inventory::SplitButton>(world),
        "destroy":inventory_widget::<crate::inventory::DestroyButton>(world),"cancel":inventory_widget::<crate::inventory::CancelButton>(world)})
}
fn os_trace(world: &mut World) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let mut run = world.remove_resource::<OsTrace>().unwrap();
    if run.written.elapsed() >= std::time::Duration::from_millis(100) {
        let state = world.resource::<crate::network::Network>().state();
        let guide = guide_screen(world);
        let enemy = state
            .targeting
            .as_ref()
            .and_then(|v| v.targets.first())
            .and_then(|t| enemy_screen(world, t.key));
        let angles = world.resource::<crate::camera::Orbit>().angles();
        let camera_mode = world
            .get_resource::<crate::camera::mode::Mode>()
            .map(|m| serde_json::json!({"requested":m.requested,"active":m.active}));
        let cursor = world
            .query::<&bevy::window::CursorOptions>()
            .single(world)
            .ok()
            .map(|c| serde_json::json!({"visible":c.visible,"grab":format!("{:?}",c.grab_mode)}));
        let intent = world.resource::<crate::movement::MoveIntent>();
        let movement =
            serde_json::json!({"direction":intent.direction.to_array(),"sprint":intent.sprint});
        let size = world
            .query::<&Window>()
            .single(world)
            .ok()
            .map(|w| (w.width(), w.height(), w.scale_factor()));
        let inventory_open = world
            .get_resource::<crate::inventory::Ui>()
            .is_some_and(|ui| ui.open);
        let adapter = world
            .get_resource::<bevy::render::renderer::RenderAdapterInfo>()
            .map(|a| serde_json::json!({"name":a.name.clone(),"driver":a.driver.clone(),"driver_info":a.driver_info.clone(),"backend":format!("{:?}",a.backend)}));
        let dodge=state.correction.map(|c|serde_json::json!({"tick":c.state.input.tick,"rate":c.state.rate.hz(),"snapshot":c.state.pose.dodge,"authority_feet":c.state.pose.feet.metres(),"input":c.state.input.intent}));
        let rendered = world
            .resource::<crate::network::Network>()
            .rendered_global()
            .map(|p| p.metres());
        let dodge_animation=world.get_resource::<crate::dodge::animation::State>().map(|s|serde_json::json!({"available":s.available,"starts":s.starts,"resumes":s.resumes,"maximum_pose_changes":s.maximum_pose_changes,"current":s.current.map(|c|serde_json::json!({"slot":c.slot,"started_at":c.key.first_start,"elapsed":c.elapsed}))}));
        let grip = world
            .get_resource::<crate::reference_scene::grip::Metrics>()
            .map(|m| serde_json::to_value(m).unwrap());
        let sword = state.correction.is_some_and(|c| {
            crate::equipment::sword_equipped(state.appearance.as_ref(), c.state.actor)
        });
        let sword_slot = widget_position(world, false, "theia.dev.training-sword");
        let wand_slot = widget_position(world, false, "theia.dev.training-wand");
        let equip = widget_position(world, true, "");
        let pawn_heading = world
            .query_filtered::<&Transform, With<crate::movement::LocalPawn>>()
            .single(world)
            .ok()
            .map(|t| t.rotation.to_array());
        let autoattack=state.autoattack.as_ref().map(|a|serde_json::json!({"phase":format!("{:?}",a.phase),"target":a.target.map(|t|t.entity.value()),"stream":a.stream_epoch,"tick":a.tick,"ready_at":a.ready_at,"swings":a.swing_sequence,"reply":a.reply.map(|r|format!("{:?}",r.outcome))}));
        let combat = world
            .get_resource::<crate::combat::Metrics>()
            .map(|m| serde_json::to_value(m).unwrap());
        let attack_animation = world
            .get_resource::<crate::reference_scene::attack::Metrics>()
            .map(|m| serde_json::to_value(m).unwrap());
        let melee_labels = world
            .get_resource::<crate::combat_feedback::Pending>()
            .map(|p| serde_json::to_value(&p.metrics).unwrap());
        let journal = crate::quest_journal::qualification(world, &state);
        let selection = world.resource::<super::Selection>();
        let mut data = serde_json::json!({"enemy_health":state.combat.as_ref().and_then(|c|c.targets.first()).map(|t|t.current),"quest_journal":journal,"quest_marker":world.get_resource::<crate::quest_marker::Metrics>(),"main_hand_item":state.inventory.as_ref().and_then(|i|i.bag.main_hand()).map(|stack|stack.item().as_str()),"wand_slot_screen":wand_slot.map(|p|p.to_array()),"ability_reply":state.combat.as_ref().and_then(|c|c.reply).map(|r|format!("{:?}",r.outcome)),"active_cast":state.combat.as_ref().and_then(|c|c.cast.as_ref()).map(|c|serde_json::json!({"started":c.started,"completes":c.completes,"ability":c.ability.as_str()})),"enemy_global":state.targeting.as_ref().and_then(|v|v.targets.first()).map(|t|t.position.metres()),"autoattack":autoattack,"combat":combat,"attack_animation":attack_animation,"melee_labels":melee_labels,"combat_ui":world.get_resource::<crate::combat::ui::Model>(),"combat_layout":world.get_resource::<crate::combat::ui::Metrics>(),"selected_hostile":state.targeting.as_ref().and_then(|v|v.selected).map(|id|id.value()),"classic_facing":state.correction.and_then(|c|c.state.pose.facing),"classic_enabled":world.contains_resource::<crate::classic_controls::Controller>(),"pawn_heading":pawn_heading,"sword_equipped":sword,"sword_slot_screen":sword_slot.map(|v|v.to_array()),"equip_screen":equip.map(|v|v.to_array()),"dodge_animation":dodge_animation,"grip":grip,"dodge":dodge,"rendered_body":rendered,"pid":std::process::id(),"connected":state.error.is_none()&&state.welcome.is_some(),"guide_screen":guide.map(|p|p.to_array()),"enemy_screen":enemy.map(|p|p.to_array()),"window":size,"camera_angles":angles.to_array(),"camera_mode":camera_mode,"cursor":cursor,"movement":movement,"inventory_open":inventory_open,"dialogue_open":state.dialogue.as_ref().is_some_and(|v|v.open.is_some()),"friendly_selected":selection.friendly.is_some(),"controls":selection,"adapter":adapter});
        data["dialogue_pending"] = serde_json::json!(state.dialogue_pending);
        data["dialogue_ui"] = world
            .get_resource::<crate::dialogue::Ui>()
            .map(|ui| serde_json::to_value(ui).unwrap())
            .unwrap_or_default();
        data["inventory_ui"] = world
            .get_resource::<crate::inventory::Ui>()
            .map(|ui| serde_json::to_value(ui).unwrap())
            .unwrap_or_default();
        data["inventory_bag"] = state
            .inventory
            .as_ref()
            .map(|v| serde_json::from_slice::<serde_json::Value>(&v.bag.to_json()).unwrap())
            .unwrap_or_default();
        data["inventory_layout"] = inventory_layout(world);
        let remote_centres: Vec<_> = world
            .query::<(&crate::network::RemotePawn, &GlobalTransform)>()
            .iter(world)
            .map(|(p, t)| (p.actor_id(), t.translation() + Vec3::Y * 0.3))
            .collect();
        data["remote_player_screens"]=serde_json::json!(remote_centres.into_iter().map(|(actor,point)|serde_json::json!({"actor":actor,"screen":project(world,point).map(|p|p.to_array())})).collect::<Vec<_>>());
        data["avatar_pose_resets"] = serde_json::json!(
            world
                .get_resource::<crate::network::Network>()
                .map(|n| n.avatar_pose_resets)
        );
        data["player_life_animation"] =
            serde_json::json!(world.get_resource::<crate::reference_scene::LifeAnimationMetrics>());
        data["minimap"] = serde_json::json!(world.get_resource::<crate::minimap::Metrics>());
        data["player_locomotion"] = crate::reference_scene::locomotion::qualification(world);
        data["render_frame"] = serde_json::json!({
            "revision":world.resource::<crate::floating_origin::Frame>().revision,
            "origin":world.resource::<crate::floating_origin::Frame>().render.origin().metres()
        });
        data["camera_distance"] =
            serde_json::json!(world.resource::<crate::camera::Orbit>().distance());
        data["minimap_streaming"] = serde_json::json!(world.get_resource::<crate::graybox::Graybox>().map(|g|
            serde_json::json!({"loaded":g.loaded,"unloaded":g.unloaded,"patches":g.patches,"saved":g.saved_stats()})));
        data["public_avatar_life"]=serde_json::json!(state.public_avatar_life.as_ref().and_then(|p|p.current()).map(|v|serde_json::json!({"tick":v.tick,"actors":v.actors.iter().map(|a|serde_json::json!({"actor":a.key.actor.entity.value(),"generation":a.key.generation.get(),"dead":a.dead})).collect::<Vec<_>>()})));
        data["actor"] = serde_json::json!(state.id);
        data["snapshot_players"] =
            serde_json::json!(state.snapshot.as_ref().map(|s| s.players.len()));
        data["creature_visual"] = serde_json::json!(
            world
                .get_resource::<crate::creature_visual::Pending>()
                .map(|p| &p.metrics)
        );
        data["public_creature_attack"] = serde_json::json!(state.public_creature_attack.as_ref().and_then(|p|p.current()).map(|o|serde_json::json!({"creature":o.creature.entity.value(),"tick":o.tick,"windup":o.windup.map(|w|serde_json::json!({"target":w.target.entity.value(),"starts":w.starts,"completes":w.completes}))})));
        std::fs::create_dir_all(&run.output).unwrap();
        std::fs::write(
            run.output.join("input-os.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
        let capture = run.output.join("capture.request");
        if capture.is_file() {
            std::fs::remove_file(capture).unwrap();
            world
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(run.output.join("input-os.png")));
        }
        run.written = std::time::Instant::now();
    }
    world.insert_resource(run);
}
