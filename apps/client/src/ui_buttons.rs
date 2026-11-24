//! Release recovery for standard Bevy buttons when panels move or hide on press.
use bevy::{input::touch::Touches, prelude::*, ui::Pressed, ui_widgets::Button};

pub(crate) fn clear_released_presses(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Option<Res<Touches>>,
    buttons: Query<Entity, (With<Button>, With<Pressed>)>,
) {
    // Pinned Bevy 0.20 sends release to the hovered widget. A modal, pending
    // action or bag reflow can move the original button before up arrives.
    // Retain real holds, including touch; recover only after every input is up.
    if mouse.get_pressed().next().is_none()
        && !touches.is_some_and(|touches| touches.iter().next().is_some())
    {
        for button in &buttons {
            commands.entity(button).remove::<Pressed>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        picking::{
            backend::HitData,
            events::{Pointer, PointerPress, PointerRelease},
            pointer::{Location, PointerButton, PointerId},
        },
        ui_widgets::{Activate, ActivateOnPress, ButtonPlugin},
    };

    #[derive(Resource, Default)]
    struct Activations(Vec<Entity>);

    #[test]
    fn ordinary_buttons_outside_inventory_recover_after_release_over_a_different_widget() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Activations>()
            .add_plugins(ButtonPlugin)
            .add_observer(|event: On<Activate>, mut calls: ResMut<Activations>| {
                calls.0.push(event.entity);
            })
            .add_systems(PostUpdate, clear_released_presses);
        let button = app.world_mut().spawn((Button, ActivateOnPress)).id();
        let elsewhere = app.world_mut().spawn_empty().id();
        let pointer = Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(
                    elsewhere,
                ))
                .normalize(Some(elsewhere))
                .unwrap(),
                position: Vec2::ZERO,
            },
        );
        let press = PointerPress {
            entity: button,
            pointer: pointer.clone(),
            button: PointerButton::Primary,
            hit: HitData::new(elsewhere, 0., None, None),
            count: 1,
        };
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut().trigger(press.clone());
        app.update();
        assert!(app.world().get::<Pressed>(button).is_some());
        assert_eq!(app.world().resource::<Activations>().0, vec![button]);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.world_mut().trigger(PointerRelease {
            entity: elsewhere,
            pointer,
            button: PointerButton::Primary,
            hit: HitData::new(elsewhere, 0., None, None),
        });
        app.update();
        assert!(app.world().get::<Pressed>(button).is_none());
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut().trigger(press);
        app.update();
        assert_eq!(
            app.world().resource::<Activations>().0,
            vec![button, button]
        );
    }
}
