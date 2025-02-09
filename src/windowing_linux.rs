use crate::prelude::*;
use bevy::window::{Monitor, PrimaryWindow, WindowMode};
use winit::{dpi::PhysicalSize, window::WindowId};

pub mod prelude {
    pub use super::WindowingDone;
}

/// Wayland needs very different setup to x11.
/// This lets us know at run time if we are using wayland.
#[derive(Resource)]
struct Wayland(bool);

#[system(Update)]
fn wayland(
    winit_windows: NonSend<WinitWindows>,
    mut commands: Commands,
    wayland: Option<Res<Wayland>>,
) {
    // If we already know if we are using wayland, return.
    if wayland.is_some() {
        return;
    }

    let Some(window) = winit_windows.windows.values().next() else {
        return;
    };

    // Wayland does not let us get inner_position, but x11 does, so we can use it to work out which one is being used.
    let is_wayland = if window.inner_position().is_err() {
        true
    } else {
        false
    };

    info!("Wayland: {is_wayland}");
    commands.insert_resource(Wayland(is_wayland));
}

#[init]
#[derive(Resource, Default)]
enum WindowingState {
    #[default]
    CreateWindows,
    ConfigureWindows,
    WaitUntilStableWindowSize(HashMap<WindowId, (u8, PhysicalSize<u32>)>),
    Done,
}

/// Sent out when all windows are fully finished being setup.
#[init]
#[derive(Event)]
pub struct WindowingDone;

#[system(Update)]
fn create_windows_x11(
    primary_window: Option<Single<Entity, With<PrimaryWindow>>>,
    monitors: Query<Entity, With<Monitor>>,
    mut commands: Commands,
    wayland: Option<Res<Wayland>>,
    mut windowing_state: ResMut<WindowingState>,
) {
    if !matches!(*windowing_state, WindowingState::CreateWindows) {
        return;
    }

    let Some(wayland) = wayland else {
        return;
    };
    if wayland.0 {
        return;
    }

    let Some(primary_window) = primary_window else {
        error!("Primary window could not be found.");
        return;
    };
    commands.entity(*primary_window).despawn();

    monitors.iter().enumerate().for_each(|(index, monitor)| {
        let mut window = commands.spawn_empty();
        let window_entity = window.id();

        // Putting the window and the camera together is useful for the grid.
        window.insert((
            Window {
                mode: WindowMode::Windowed,
                position: WindowPosition::Centered(MonitorSelection::Entity(monitor)),
                transparent: true,
                ..default()
            },
            Camera2d,
            RenderLayers::layer(index),
            Camera {
                target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                ..default()
            },
        ));
    });

    *windowing_state = WindowingState::ConfigureWindows;
}

#[system(Update)]
fn configure_windows_x11(
    mut windowing_state: ResMut<WindowingState>,
    winit_windows: NonSend<WinitWindows>,
    wayland: Option<Res<Wayland>>,
) {
    if !matches!(*windowing_state, WindowingState::ConfigureWindows) {
        return;
    }

    let Some(wayland) = wayland else {
        return;
    };
    if wayland.0 {
        return;
    }

    // If there are 0 or 1 windows, then we know something has gone wrong.
    if winit_windows.windows.len() <= 1 {
        return;
    }

    winit_windows.windows.values().for_each(|window| {
        window.set_maximized(true);
    });

    *windowing_state = WindowingState::WaitUntilStableWindowSize(default());
}

// When you maximise a window, an animation of it growing will play. We have to wait for that to finish. There is no good way to detect this.
#[system(Update)]
fn wait_until_stable_window_size(
    mut windowing_state: ResMut<WindowingState>,
    mut windowing_done: EventWriter<WindowingDone>,
    // WindowResized events do not detect maximising, so we have to use this.
    winit_windows: NonSend<WinitWindows>,
) {
    let WindowingState::WaitUntilStableWindowSize(window_resize_map) = &mut *windowing_state else {
        return;
    };

    let mut all_windows_stable = true;

    winit_windows.windows.iter().for_each(|(id, window)| {
        let (frames_since_last_resize, previous_size) = match window_resize_map.get_mut(id) {
            Some((frames_since_last_resize, previous_size)) => {
                (frames_since_last_resize, previous_size)
            }
            None => {
                window_resize_map.insert(*id, default());
                let (frames_since_last_resize, previous_size) =
                    window_resize_map.get_mut(id).unwrap();
                (frames_since_last_resize, previous_size)
            }
        };

        let size = window.inner_size();

        if size != *previous_size {
            *frames_since_last_resize = 0;
            *previous_size = size;
        } else {
            *frames_since_last_resize += 1;
        }

        if *frames_since_last_resize < 3 {
            all_windows_stable = false;
        }
    });

    if all_windows_stable {
        *windowing_state = WindowingState::Done;
        windowing_done.send(WindowingDone);
        info!("Windowing done!");
    }
}
