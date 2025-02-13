mod prelude {
    #[cfg(target_os = "linux")]
    pub use crate::windowing_linux::prelude::*;
    pub use crate::{
        actions::prelude::*, cursor_translation::prelude::*, interactable::prelude::*,
        physics::prelude::*, tools::prelude::*,
    };
    pub use avian2d::prelude::*;
    pub use bevy::{
        prelude::*,
        render::{camera::RenderTarget, view::RenderLayers},
        window::WindowRef,
        winit::WinitWindows,
    };
    pub use bevy_registration::prelude::*;
    pub use foldhash::HashMap;
    pub use leafwing_input_manager::prelude::*;
}
use std::time::Duration;

use prelude::*;

mod actions;
mod cursor_translation;
mod draw_terrain;
mod interactable;
mod physics;
mod tools;
mod water;

//mod windowing;
#[cfg(target_os = "linux")]
mod windowing_linux;

schedule! {
    Update (
        [run_every(Duration::from_secs_f32(1. / 60.))]
        Fluid (
            Pressure,
            GetAcceleration,
            ApplyAcceleration
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
            //PhysicsDebugPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Gravity(Vec2::new(0., -100.)))
        .insert_resource(SubstepCount(1))
        .run();
}

// trait UnwrapTo<T> {
//     fn unwrap_to(self) -> T;
// }

// impl<T> UnwrapTo<T> for Option<T> {
//     fn unwrap_to(self) -> T {
//         self.unwrap()
//     }
// }

// struct RunIf<'world, 'state, T: SystemParam> {
//     item: T::Item<'world, 'state>,
// }

// unsafe impl<'struct_world, 'struct_state, 'where_world, 'where_state, T: SystemParam> SystemParam for RunIf<'struct_world, 'struct_state, T>
// where
//     Option<T>: SystemParam,
//     <Option<T> as SystemParam>::Item<'where_world, 'where_state>: UnwrapTo<<T as SystemParam>::Item<'where_world, 'where_state>>,
// {
//     type Item<'world, 'state> = RunIf<'world, 'state, T>;
//     type State = <Option<T> as SystemParam>::State;

//     fn init_state(
//         world: &mut World,
//         system_meta: &mut bevy::ecs::system::SystemMeta,
//     ) -> Self::State {
//         Option::<T>::init_state(world, system_meta)
//     }

//     unsafe fn get_param<'world, 'state>(
//         state: &'state mut Self::State,
//         system_meta: &bevy::ecs::system::SystemMeta,
//         world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
//         change_tick: bevy::ecs::component::Tick,
//     ) -> Self::Item<'world, 'state> {
//         let option = unsafe { Option::<T>::get_param(state, system_meta, world, change_tick) };

//         // TODO: Somehow stop the system if this is None, and otherwise return the contained item?

//         let item: <T as SystemParam>::Item<'world, 'state> = option.unwrap_to();

//         RunIf {
//             item,
//         }
//     }

//     fn apply(
//         state: &mut Self::State,
//         system_meta: &bevy::ecs::system::SystemMeta,
//         world: &mut World,
//     ) {
//         Option::<T>::apply(state, system_meta, world);
//     }

//     unsafe fn new_archetype(
//         state: &mut Self::State,
//         archetype: &bevy::ecs::archetype::Archetype,
//         system_meta: &mut bevy::ecs::system::SystemMeta,
//     ) {
//         unsafe {
//             Option::<T>::new_archetype(state, archetype, system_meta);
//         }
//     }

//     fn queue(
//         state: &mut Self::State,
//         system_meta: &bevy::ecs::system::SystemMeta,
//         world: bevy::ecs::world::DeferredWorld,
//     ) {
//         Option::<T>::queue(state, system_meta, world);
//     }

//     unsafe fn validate_param(
//         state: &Self::State,
//         system_meta: &bevy::ecs::system::SystemMeta,
//         world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
//     ) -> bool {
//         unsafe { Option::<T>::validate_param(state, system_meta, world) }
//     }
// }
