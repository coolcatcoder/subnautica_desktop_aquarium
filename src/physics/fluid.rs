// Almost everything comes from https://shahriyarshahrabi.medium.com/gentle-introduction-to-fluid-simulation-for-programmers-and-technical-artists-7c0045c40bac

use crate::prelude::*;

pub mod prelude {
    pub use super::Fluid;
}

#[derive(Component, Default)]
#[require(Velocity)]
pub struct Fluid;

#[derive(Component, Default)]
#[require(VelocityDivergence)]
pub struct Velocity(Vec2);

#[derive(Component, Default)]
pub struct VelocityDivergence(f32);

#[system(Update::Fluid::VelocityDivergence)]
fn velocity_divergence(
    mut velocity_divergence: Query<(&Cell, &mut VelocityDivergence)>,
    velocity: Query<&Velocity>,
) {
    velocity_divergence
        .par_iter_mut()
        .for_each(|(cell, mut velocity_divergence)| {
            // This mess gets the nearest 4 velocities.
            // If it can't get one (Edge of grid.), then it instead gives a default value.
            let mut velocities = cell.nearest_4.iter().map(|entity| {
                entity
                    .and_then(|entity| velocity.get(entity).ok().map(|velocity| velocity.0))
                    .unwrap_or(Vec2::ZERO)
            });
            let velocities: [Vec2; 4] = std::array::from_fn(|_| velocities.next().unwrap());

            velocity_divergence.0 = divergence(velocities);
        });
}

// pub struct Fluid {
//     pub velocity: Vec2,

//     pub divergence: f32,
//     /// Used for updating divergence.
//     pub divergence_transfer: f32,

//     pub pressure: f32,
// }

// impl Fluid {
//     pub const EMPTY: Self = Self {
//         velocity: Vec2::ZERO,

//         divergence: 0.,
//         divergence_transfer: 0.,

//         pressure: 0.,
//     };
// }

/// Calculates the divergence.
/// Nearest 4 is ordered top, left, right, bottom.
fn divergence(nearest_4: [Vec2; 4]) -> f32 {
    (nearest_4[2].x - nearest_4[1].x) + (nearest_4[0].y - nearest_4[3].y)
}

/// Calculates the gradient.
/// Nearest 4 is ordered top, left, right, bottom.
fn gradient(nearest_4: [f32; 4]) -> Vec2 {
    Vec2::new(nearest_4[2] - nearest_4[1], nearest_4[0] - nearest_4[3])
}
