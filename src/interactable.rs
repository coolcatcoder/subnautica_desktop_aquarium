use crate::prelude::*;

pub mod prelude {
    pub use super::Interactable;
}

/// Sets whether you can click on this window, or if it goes through.
#[init]
#[derive(Resource)]
pub struct Interactable(pub bool);
impl Default for Interactable {
    fn default() -> Self {
        Self(true)
    }
}

#[system(Update)]
fn interactable(
    mut previous: Local<bool>,
    interactable: Res<Interactable>,
    winit_windows: NonSend<WinitWindows>,
) {
    if *previous != interactable.0 {
        *previous = interactable.0;
        info!("Changed interactable status.");
        winit_windows.windows.values().for_each(|window| {
            if let Err(error) = window.set_cursor_hittest(interactable.0) {
                error!("Tried to set hit test. Encountered error: {error}");
            }
        });
    }
}
