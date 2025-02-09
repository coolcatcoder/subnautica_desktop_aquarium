use crate::prelude::*;

pub mod prelude {
    pub use super::{CursorTranslation, InnerCursorTranslation};
}

/// The world space cursor translation and the camera it is on.
#[init]
#[derive(Resource, Default)]
pub struct CursorTranslation(pub Option<InnerCursorTranslation>);

pub struct InnerCursorTranslation {
    pub translation: Vec2,
    pub window: Entity,
}

#[system(Update)]
fn cursor_translation(
    mut cursor_translation: ResMut<CursorTranslation>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    cursor_translation.0 = None;
    cameras.iter().any(|(camera, global_transform)| {
        let RenderTarget::Window(WindowRef::Entity(window_entity)) = camera.target else {
            return false;
        };
        let Ok(window) = windows.get(window_entity) else {
            return false;
        };

        let Some(translation) = window.cursor_position().and_then(|translation| {
            camera
                .viewport_to_world_2d(global_transform, translation)
                .ok()
        }) else {
            return false;
        };

        cursor_translation.0 = Some(InnerCursorTranslation {
            translation,
            window: window_entity,
        });
        // Short circuit once we have found the translation.
        true
    });
    //info!("{:?}", cursor_translation.0);
}
