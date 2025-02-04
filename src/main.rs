mod prelude {
    pub use crate::{
        actions::prelude::*, cursor_translation::prelude::*, tools::prelude::*,
        windowing::prelude::*,
    };
    pub use avian2d::prelude::*;
    pub use bevy::{prelude::*, render::camera::RenderTarget, window::WindowRef};
    pub use bevy_registration::prelude::*;
    pub use leafwing_input_manager::prelude::*;
}
use std::time::Duration;

use prelude::*;

mod actions;
mod cursor_translation;
mod draw_terrain;
mod tools;
mod water;
mod windowing;

schedule! {
    Update (
        [run_every(Duration::from_secs_f32(1. / 30.))]
        Fluid (
            Pressure,
            Forces,
        )
    )
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                #[cfg(not(target_os = "linux"))]
                primary_window: None,
                ..default()
            }),
            RegistrationPlugin,
            PhysicsPlugins::default().with_length_unit(30.),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Gravity(Vec2::new(0., -100.)))
        .run();
}
