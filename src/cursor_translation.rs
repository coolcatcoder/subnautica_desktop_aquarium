use crate::prelude::*;

pub mod prelude {
    pub use super::CursorTranslation;
}

/// The world space cursor translation.
#[init]
#[derive(Resource, Default)]
pub struct CursorTranslation(pub Option<Vec2>);

#[system(Update)]
fn cursor_translation(
    mut cursor_translation: ResMut<CursorTranslation>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    cursor_translation.0 = None;
    cameras.iter().any(|(camera, global_transform)| {
        let RenderTarget::Window(WindowRef::Entity(window)) = camera.target else {
            return false;
        };
        let Ok(window) = windows.get(window) else {
            return false;
        };

        let Some(translation) = window
            .cursor_position()
            .and_then(|translation| camera.viewport_to_world(global_transform, translation).ok())
            .map(|ray| ray.origin.truncate())
        else {
            return false;
        };

        cursor_translation.0 = Some(translation);
        // Short circuit once we have found the translation.
        true
    });
}
