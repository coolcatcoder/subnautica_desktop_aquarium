// Almost everything comes from https://shahriyarshahrabi.medium.com/gentle-introduction-to-fluid-simulation-for-programmers-and-technical-artists-7c0045c40bac

use crate::prelude::*;

pub mod prelude {
    pub use super::Fluid;
}

#[derive(Component, Default)]
#[require(Velocity, Pressure)]
pub struct Fluid;

#[system(Update)]
fn debug(
    cells: Query<(&Cell, &Velocity, &VelocityDivergence, &Pressure)>,
    grids: Query<&RenderLayers>,
    mut gizmos: Gizmos,
) {
    cells
        .iter()
        .for_each(|(cell, velocity, velocity_divergence, pressure)| {
            let Ok(grid) = grids.get(cell.grid) else {
                return;
            };

            if *grid != gizmos.config.render_layers {
                return;
            }

            gizmos.rect_2d(
                cell.translation,
                Vec2::splat(Cell::SIZE * 0.9),
                Srgba::new(velocity_divergence.0 * 0.25, pressure.0 * 0.1, 0., 1.),
            );

            gizmos.arrow_2d(cell.translation, cell.translation + velocity.0, Srgba::BLUE);
        });
}

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

#[derive(Component, Default)]
#[require(PressureUpdate)]
struct Pressure(f32);

#[derive(Component, Default)]
struct PressureUpdate(f32);

#[system(Update::Fluid::Solve)]
fn solve(
    mut calculate_and_update: ParamSet<(
        (
            Query<(&Cell, &mut PressureUpdate, &VelocityDivergence, &Pressure)>,
            Query<&Pressure, Without<Solid>>,
        ),
        Query<(&mut Pressure, &PressureUpdate)>,
    )>,
) {
    (0..30).for_each(|_| {
        let (mut pressure_update, pressure) = calculate_and_update.p0();

        pressure_update.par_iter_mut().for_each(
            |(cell, mut pressure_update, velocity_divergence, center_pressure)| {
                let mut sum_of_neighbours = 0.;
                cell.nearest_4.iter().for_each(|entity| {
                    let pressure = entity
                        .and_then(|entity| pressure.get(entity).ok())
                        .map(|pressure| pressure.0)
                        .unwrap_or(center_pressure.0);
                    sum_of_neighbours += pressure;
                });

                pressure_update.0 = (sum_of_neighbours - velocity_divergence.0) / 4.;
            },
        );

        calculate_and_update
            .p1()
            .par_iter_mut()
            .for_each(|(mut pressure, pressure_update)| {
                pressure.0 = pressure_update.0;
            });
    });
}

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

#[system(Update::Fluid::Forces)]
fn gravity(mut velocity: Query<&mut Velocity>, time: Res<Time>) {
    let time_delta_seconds = time.delta_secs();
    velocity.par_iter_mut().for_each(|mut velocity| {
        velocity.0.y -= 5. * time_delta_seconds;

        let velocity_delta = velocity.0.abs() * velocity.0 * 0.005 * time_delta_seconds;
        velocity.0 -= velocity_delta;
    });
}
