use crate::prelude::*;

#[derive(Resource)]
struct Settings {
    colour: Color,
}
app!(|app| {
    app.insert_resource(Settings {
        colour: Srgba::new(0., 0., 1., 0.3).into(),
    });
});

#[system(Update)]
fn spawn(
    tool: Res<Tool>,
    actions: Actions,
    cursor_translation: Res<CursorTranslation>,
    mut commands: Commands,
    settings: Res<Settings>,
    tool_bar_hovered: Res<ToolBarHovered>,
    asset_server: Res<AssetServer>,
) {
    if !matches!(*tool, Tool::Water) {
        return;
    }

    if tool_bar_hovered.0 {
        return;
    }

    if !actions.just_pressed(&Action::Use) {
        return;
    }

    let Some(translation) = cursor_translation.0 else {
        return;
    };

    commands
        .spawn((
            Fluid::default(),
            RigidBody::Dynamic,
            Collider::circle(10.),
            Transform::from_translation(Vec3::new(translation.x, translation.y, 0.)),
            Sprite {
                image: asset_server.load("brushes/circle.png"),
                color: settings.colour,
                custom_size: Some(Vec2::splat(20.)),
                ..default()
            },
        ))
        .with_child((
            Sensor,
            Collider::circle(PRESSURE_RADIUS),
            CollidingEntities::default(),
        ));
}

const PRESSURE_RADIUS: f32 = 80.;
const PRESSURE_RADIUS_SQUARED: f32 = PRESSURE_RADIUS * PRESSURE_RADIUS;
const POLY6: f32 = 4.
    / (std::f32::consts::PI
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS);
const GAS: f32 = 2000.;
const REST_DENS: f32 = 300.;
// Changed -10 to -30 chatgpt
const SPIKY_GRAD: f32 = -30.
    / (std::f32::consts::PI
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS
        * PRESSURE_RADIUS);
const VISC: f32 = 200.0;

#[derive(Component, Default)]
struct Fluid {
    rho: f32,
    pressure: f32,
}

#[system(Update::Fluid::Pressure)]
fn pressure(
    mut particles: Query<(&mut Fluid, &Transform, &Children)>,
    collisions: Query<&CollidingEntities>,
    colliders: Query<&Transform, With<Fluid>>,
) {
    particles
        .par_iter_mut()
        .for_each(|(mut fluid, transform, children)| {
            // All fluid particles must have 1 child containing the density sensor.
            let child = children.first().unwrap();

            let Ok(collisions) = collisions.get(*child) else {
                error!("Child could not be got.");
                return;
            };

            fluid.rho = 0.001;
            collisions.0.iter().for_each(|collision| {
                let Ok(collider) = colliders.get(*collision) else {
                    return;
                };

                let distance_squared = collider
                    .translation
                    .xy()
                    .distance_squared(transform.translation.xy());
                let precomputation = PRESSURE_RADIUS_SQUARED - distance_squared;

                fluid.rho += POLY6 * (precomputation * precomputation * precomputation);
            });
            fluid.pressure = GAS * (fluid.rho - REST_DENS);
        });
}

#[system(Update::Fluid::Forces)]
fn forces(
    mut particles: Query<(&mut LinearVelocity, &Fluid, &Transform, &Children)>,
    collisions: Query<&CollidingEntities>,
    colliders: Query<(&Fluid, &Transform)>,
    time: Res<Time>,
) {
    let time_delta_seconds = time.delta_secs();

    particles
        .par_iter_mut()
        .for_each(|(mut velocity, fluid, transform, children)| {
            // All fluid particles must have 1 child containing the density sensor.
            let child = children.first().unwrap();

            let Ok(collisions) = collisions.get(*child) else {
                error!("Child could not be got.");
                return;
            };

            let mut fpress = Vec2::ZERO;
            let mut fvisc = Vec2::ZERO;
            collisions.0.iter().for_each(|collision| {
                let Ok((collider_fluid, collider_transform)) = colliders.get(*collision) else {
                    return;
                };

                let direction_unnormalised =
                    collider_transform.translation.xy() - transform.translation.xy();
                let distance = direction_unnormalised.length();

                let precomputation = PRESSURE_RADIUS - distance;

                fpress += -direction_unnormalised.normalize_or_zero()
                    * (fluid.pressure + collider_fluid.pressure)
                    / (2. * collider_fluid.rho)
                    * SPIKY_GRAD
                    * (precomputation * precomputation); // CHANGED FROM to the power of 3, instead of 2, chatgpt, make sure this is right.

                //fvisc += VISC;
            });

            info!("fpress: {:.2}", fpress);
            info!("fvisc: {}", fvisc);
            info!("rho: {}", fluid.rho);
            info!("time_delta_seconds: {}", time_delta_seconds);
            info!("something: {}", time_delta_seconds * fpress / fluid.rho);

            //velocity.0 += time_delta_seconds * (fpress + fvisc) / fluid.rho;
        });
}
