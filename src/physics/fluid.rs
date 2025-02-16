use crate::prelude::*;

pub mod prelude {
    pub use super::Fluid;
}

pub struct Fluid {
    pub velocity: Vec2,

    pub divergence: f32,
    /// Used for updating divergence.
    pub divergence_transfer: f32,

    pub pressure: f32,
}

impl Fluid {
    pub const EMPTY: Self = Self {
        velocity: Vec2::ZERO,

        divergence: 0.,
        divergence_transfer: 0.,

        pressure: 0.,
    };
}

fn divergence(mut grids: Query<&mut Grid>) {
    grids.par_iter_mut().for_each(|mut grid| {});
}
