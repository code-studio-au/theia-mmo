//! Test-only standard Bevy CPU font/layout harness; no renderer or OS backend.
use super::*;

pub(super) fn add_layout(app: &mut App) {
    use bevy::app::{HierarchyPropagatePlugin, PropagateSet, TaskPoolPlugin};
    use bevy::camera::{ComputedCameraValues, RenderTargetInfo, Viewport};
    use bevy::text::{FontCx, LayoutCx, RemSize, ScaleCx, TextIterScratch, TextPipeline};
    use bevy::ui::update::{propagate_ui_target_cameras, update_clipping_system};
    use bevy::ui::{
        ComputedUiRenderTargetInfo, ComputedUiTargetCamera, UiScale, ui_surface::UiSurface,
    };
    use bevy::ui::{sync_font_size_to_em_size, ui_layout_system, widget::measure_text_system};
    app.add_plugins(TaskPoolPlugin::default())
        .add_plugins(HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(
            PostUpdate,
        ))
        .add_plugins(HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(
            PostUpdate,
        ))
        .init_resource::<UiScale>()
        .init_resource::<UiSurface>()
        .init_resource::<Assets<Font>>()
        .init_resource::<TextPipeline>()
        .init_resource::<FontCx>()
        .init_resource::<LayoutCx>()
        .init_resource::<ScaleCx>()
        .init_resource::<TextIterScratch>()
        .init_resource::<RemSize>()
        .add_systems(
            PostUpdate,
            (
                propagate_ui_target_cameras,
                bevy::text::load_font_assets_into_font_collection,
                bevy::text::detect_text_needs_rerender,
                sync_font_size_to_em_size,
                bevy::ui::widget::update_editable_text_styles,
                bevy::ui::widget::update_editable_text_content_size,
                measure_text_system,
                ui_layout_system,
                update_clipping_system,
            )
                .chain(),
        )
        .configure_sets(
            PostUpdate,
            PropagateSet::<ComputedUiTargetCamera>::default()
                .after(propagate_ui_target_cameras)
                .before(ui_layout_system),
        )
        .configure_sets(
            PostUpdate,
            PropagateSet::<ComputedUiRenderTargetInfo>::default()
                .after(propagate_ui_target_cameras)
                .before(sync_font_size_to_em_size),
        );
    app.world_mut()
        .resource_mut::<Assets<Font>>()
        .insert(
            bevy::asset::AssetId::default(),
            Font::from_bytes(bevy::text::DEFAULT_FONT_DATA.to_vec()),
        )
        .unwrap();
    app.world_mut().spawn((
        Camera2d,
        Camera {
            computed: ComputedCameraValues {
                target_info: Some(RenderTargetInfo {
                    physical_size: UVec2::new(3440, 1440),
                    scale_factor: 1.,
                }),
                ..default()
            },
            viewport: Some(Viewport {
                physical_size: UVec2::new(3440, 1440),
                ..default()
            }),
            ..default()
        },
    ));
    // UiPlugin normally supplies this required state. These data-only apps
    // already spawned their fixed fields before adding the CPU layout systems.
    let fields: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<bevy::text::EditableText>>()
        .iter(app.world())
        .collect();
    for entity in fields {
        app.world_mut()
            .entity_mut(entity)
            .insert(bevy::ui::widget::EditableTextContentSizeState::default());
    }
}
