use crate::prelude::*;
use bevy::{
    window::{CompositeAlphaMode, Monitor, PrimaryWindow, WindowMode},
    winit::WinitWindows,
};

pub mod prelude {
    pub use super::Interactable;
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    #[derive(Resource)]
    pub struct Wayland(pub bool);

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

        let is_wayland = if window.inner_position().is_err() {
            true
        } else {
            false
        };

        info!("Wayland: {is_wayland}");
        commands.insert_resource(Wayland(is_wayland));
    }
}

#[cfg(target_os = "linux")]
#[system(Update)]
fn create_windows(
    primary_window: Option<Single<Entity, With<PrimaryWindow>>>,
    monitors: Query<(Entity, &Monitor)>,
    mut commands: Commands,
    wayland: Option<Res<linux::Wayland>>,
    mut setup_windows: EventWriter<SetupWindows>,
    mut finished: Local<bool>,
) {
    if *finished {
        return;
    }

    let Some(wayland) = wayland else {
        return;
    };

    let Some(primary_window) = primary_window else {
        return;
    };

    commands.entity(*primary_window).despawn();
    *finished = true;

    if wayland.0 {
        // Because we cannot position windows in wayland, we have to make do with a single window.
        let window_entity = commands
            .spawn(Window {
                mode: WindowMode::Windowed,
                transparent: true,
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                ..default()
            })
            .id();

        commands.spawn((UiCamera, Camera2d, Camera {
            target: RenderTarget::Window(WindowRef::Entity(window_entity)),
            ..default()
        }));

        setup_windows.send(SetupWindows(1));
    } else {
        monitors
            .iter()
            .enumerate()
            .for_each(|(index, (monitor_entity, monitor))| {
                let window_entity = commands
                    .spawn(Window {
                        mode: WindowMode::Windowed,
                        position: WindowPosition::Centered(MonitorSelection::Entity(
                            monitor_entity,
                        )),
                        transparent: true,
                        ..default()
                    })
                    .id();

                let mut camera = commands.spawn((Camera2d, Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                    ..default()
                }));

                if index == 0 {
                    camera.insert(UiCamera);
                }
            });

        setup_windows.send(SetupWindows(monitors.iter().len()));
    }
}

#[init]
#[derive(Event)]
struct SetupWindows(usize);

#[cfg(target_os = "linux")]
#[system(Update)]
fn setup_windows(
    mut setup_windows: EventReader<SetupWindows>,
    winit_windows: NonSend<WinitWindows>,
    mut maybe_quantity: Local<Option<usize>>,
    wayland: Option<Res<linux::Wayland>>,
) {
    if let Some(setup_windows) = setup_windows.read().next() {
        *maybe_quantity = Some(setup_windows.0);
        return;
    }

    let Some(wayland) = wayland else {
        return;
    };

    let Some(quantity) = *maybe_quantity else {
        return;
    };

    if winit_windows.windows.len() != quantity {
        return;
    }

    if wayland.0 {
        let window = winit_windows.windows.values().next().unwrap();

        window.set_maximized(true);
    } else {
        winit_windows.windows.values().for_each(|window| {
            window.set_maximized(true);
        });
    }

    *maybe_quantity = None;
    info!("Windows are setup!");
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
