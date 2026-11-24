//! Native validation host for the reused editor; no production UI is replaced.
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{Screenshot, save_to_disk},
    },
    text::EditableText,
    ui_widgets::{Button, TextInput},
};
use bevy_sprinkles::prelude::*;
use bevy_sprinkles_editor::{
    state::{DirtyState, EditorState},
    ui::components::{
        binding::{FieldAccessor, FieldBinding},
        playback_controls::{LoopButton, PlayPauseButton, StopButton},
    },
    ui::widgets::{
        color_picker::EditorColorPicker,
        curve_edit::EditorCurveEdit,
        gradient_edit::EditorGradientEdit,
        popover::{EditorPopover, PopoverCloseButton},
    },
};
use std::path::PathBuf;

#[derive(Resource)]
struct Report(PathBuf);

fn main() {
    let path = std::env::var_os("THEIA_PARTICLE_REPORT").expect("validation report destination");
    let mut app = bevy_sprinkles_editor::standalone_app();
    app.insert_resource(Report(path.into()))
        .add_systems(Update, capture)
        .add_systems(Last, report)
        .run();
}

fn capture(keys: Res<ButtonInput<KeyCode>>, path: Res<Report>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::F12) {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0.with_extension("png")));
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn report(
    path: Res<Report>,
    time: Res<Time>,
    state: Res<EditorState>,
    dirty: Res<DirtyState>,
    assets: Res<Assets<ParticlesAsset>>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    gpu: Option<Res<RenderAdapterInfo>>,
    runtime: Query<&ParticleSystemRuntime>,
    emitters: Query<&EmitterRuntime>,
    buttons: Query<
        (
            Entity,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Has<PlayPauseButton>,
            Has<StopButton>,
            Has<LoopButton>,
            Option<&Hovered>,
        ),
        With<Button>,
    >,
    fields: Query<
        (
            Entity,
            &EditableText,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
        ),
        With<TextInput>,
    >,
    hierarchy: Query<&Children>,
    parents: Query<&ChildOf>,
    texts: Query<&Text>,
    (bindings, hover, scrolls, widgets): (
        Query<&FieldBinding>,
        Res<bevy::picking::hover::HoverMap>,
        Query<(Entity, &ScrollPosition, &ComputedNode, &Node)>,
        Query<(
            Has<EditorColorPicker>,
            Has<EditorGradientEdit>,
            Has<EditorCurveEdit>,
            Has<EditorPopover>,
            Has<PopoverCloseButton>,
        )>,
    ),
) {
    let Ok(window) = windows.single() else { return };
    let asset = state
        .current_project
        .as_ref()
        .and_then(|handle| assets.get(handle));
    let button_data: Vec<_> = buttons.iter().filter(|(_, node, ..)| node.size().min_element() > 0.).map(|(entity, node, pose, play, stop, repeat, hovered)| {
        let label = hierarchy.iter_descendants(entity).filter_map(|child| texts.get(child).ok()).map(|text| text.0.as_str()).collect::<Vec<_>>().join(" ");
        let ancestry:Vec<_> = std::iter::once(entity).chain(parents.iter_ancestors(entity)).collect();
        let widget = ancestry.iter().find_map(|e| widgets.get(*e).ok().and_then(|(color,gradient,curve,_,_)| if color {Some("color")} else if gradient {Some("gradient")} else if curve {Some("curve")} else {None}));
        let field = ancestry.iter().find_map(|e| bindings.get(*e).ok()).map(binding_path);
        let close = widgets.get(entity).is_ok_and(|(_,_,_,_,close)|close);
        serde_json::json!({"entity": entity.to_bits(), "widget":widget, "field":field, "kind": if play { "play" } else if stop { "stop" } else if repeat { "loop" } else if close {"close"} else { "button" },
            "text": label, "hovered": hovered.is_some_and(|h| h.0), "centre": pose.affine().translation.to_array(), "size": node.size().to_array()})
    }).collect();
    let field_data: Vec<_> = fields.iter().filter(|(_, _, node, _)| node.size().min_element() > 0.).map(|(entity, text, node, pose)| {
        let binding = parents.iter_ancestors(entity).find_map(|ancestor| bindings.get(ancestor).ok());
        let field = binding.map(|binding| match &binding.accessor {
            FieldAccessor::Direct(path) => path.clone(),
            FieldAccessor::VariantField { path, field_name } => format!("{path}.{field_name}"),
        });
        serde_json::json!({"entity": entity.to_bits(), "field": field, "value": text.value().to_string(), "centre": pose.affine().translation.to_array(), "size": node.size().to_array()})
    }).collect();
    let value = serde_json::json!({
        "elapsed":time.elapsed_secs_f64(), "focused":window.focused, "viewport":[window.width(),window.height()],
        "cursor":window.cursor_position().map(|p|p.to_array()), "left_mouse_down":mouse.pressed(MouseButton::Left),
        "gpu":gpu.map(|g|serde_json::json!({"name":g.name,"backend":format!("{:?}",g.backend)})),
        "dirty":dirty.has_unsaved_changes,"project":asset.and_then(|asset|serde_json::to_value(asset).ok()),
        "runtime":runtime.iter().map(|r|serde_json::json!({"paused":r.paused,"loop":r.force_loop})).collect::<Vec<_>>(),
        "emitter_count":emitters.iter().count(),"buttons":button_data,"fields":field_data,
        "popovers":widgets.iter().filter(|(_,_,_,popover,_)|*popover).count(),
        "hover":hover.values().flat_map(|h|h.keys()).map(|e|e.to_bits()).collect::<Vec<_>>(),
        "scrolls":scrolls.iter().filter(|(_,_,_,n)|n.overflow.y==OverflowAxis::Scroll).map(|(e,s,c,_)|serde_json::json!({"entity":e.to_bits(),"y":s.y,"size":c.size().to_array(),"content":c.content_size().to_array()})).collect::<Vec<_>>(),
    });
    if let Ok(bytes) = serde_json::to_vec(&value) {
        let temporary = path.0.with_extension("tmp");
        if std::fs::write(&temporary, bytes).is_ok() {
            let _ = std::fs::rename(temporary, &path.0);
        }
    }
}

fn binding_path(binding: &FieldBinding) -> String {
    match &binding.accessor {
        FieldAccessor::Direct(path) => path.clone(),
        FieldAccessor::VariantField { path, field_name } => format!("{path}.{field_name}"),
    }
}
