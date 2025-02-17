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

    if !actions.pressed(&Action::Use) {
        return;
    }

    let Some(cursor_translation) = &cursor_translation.0 else {
        return;
    };
    let translation = cursor_translation.translation;

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
            Friction {
                dynamic_coefficient: 0.,
                static_coefficient: 0.,
                ..default()
            },
            LockedAxes::ROTATION_LOCKED,
            CollisionLayers::new(Layer::Fluid, Layer::Default),
        ))
        .with_child((Sensor, Collider::circle(H), CollidingEntities::default()));
}

#[derive(PhysicsLayer, Default)]
enum Layer {
    #[default]
    Default,
    Fluid,
}

// All the fluid physics is taken from https://www.cs.cornell.edu/~bindel/class/cs5220-f11/code/sph.pdf
/*
char* fname; /* File name */
int nframes; /* Number of frames */
int npframe; /* Steps per frame */
float h; /* Particle size */
float dt; /* Time step */
float rho0; /* Reference density */
float k; /* Bulk modulus */
float mu; /* Viscosity */
float g; /* Gravity strength */

int n; /* Number of particles */
float mass; /* Particle mass */
float* restrict rho; /* Densities */
float* restrict x; /* Positions */
float* restrict vh; /* Velocities (half step) */
float* restrict v; /* Velocities (full step) */
float* restrict a; /* Acceleration */

static void default_params(sim_param_t* params)
{
params->fname = "run.out";
params->nframes = 400;
params->npframe = 100;
params->dt = 1e-4;
params->h = 5e-2;
params->rho0 = 1000;
params->k = 1e3;
params->mu = 0.1;
params->g = 9.8;
}
*/

// Unsure.
const MASS: f32 = 10.;

// Unsure.
const H: f32 = 30.;
const H2: f32 = H * H;
const H8: f32 = (H2 * H2) * (H2 * H2);

const C: f32 = 4. * MASS / std::f32::consts::PI / H8;
const C0: f32 = MASS / std::f32::consts::PI / (H2 * H2);
const CP: f32 = 15. * K;
const CV: f32 = -40. * MU;

// Unsure.
const RHO0: f32 = 5.;
// Unsure.
const K: f32 = 1000.;
// Unsure.
const MU: f32 = 0.1;

#[derive(Component, Default)]
#[require(AccelerationAccumulator)]
struct Fluid {
    // Density.
    rho: f32,
}

#[derive(Component, Default)]
struct AccelerationAccumulator(Vec2);

//#[system(Update::Fluid::Pressure)]
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

            fluid.rho = 0.;
            collisions.0.iter().for_each(|collision| {
                let Ok(collider_transform) = colliders.get(*collision) else {
                    return;
                };

                let dx = transform.translation.x - collider_transform.translation.x;
                let dy = transform.translation.y - collider_transform.translation.y;

                let r2 = dx * dx + dy * dy;
                let z = H2 - r2;
                if z > 0. {
                    let rho_ij = C * z * z * z;
                    // We can only do one of the 2 operations. Consider doubling the value, to make up for the missed return stroke?
                    fluid.rho += rho_ij * 2.;
                }
            });
            //info!("rho: {}", fluid.rho)
        });
}

//#[system(Update::Fluid::GetAcceleration)]
fn get_acceleration(
    mut particles: Query<(
        &mut AccelerationAccumulator,
        &LinearVelocity,
        &Fluid,
        &Transform,
        &Children,
    )>,
    collisions: Query<&CollidingEntities>,
    colliders: Query<(&Fluid, &Transform, &LinearVelocity)>,
) {
    particles.par_iter_mut().for_each(
        |(mut acceleration_accumulator, velocity, fluid, transform, children)| {
            // All fluid particles must have 1 child containing the density sensor.
            let child = children.first().unwrap();

            let Ok(collisions) = collisions.get(*child) else {
                error!("Child could not be got.");
                return;
            };

            acceleration_accumulator.0 = Vec2::ZERO;
            collisions.0.iter().for_each(|collision| {
                let Ok((collider_fluid, collider_transform, collider_velocity)) =
                    colliders.get(*collision)
                else {
                    return;
                };

                let dx = transform.translation.x - collider_transform.translation.x;
                let dy = transform.translation.y - collider_transform.translation.y;

                let r2 = dx * dx + dy * dy;
                if r2 < H2 {
                    let q = r2.sqrt() / H;
                    let u = 1. - q;
                    let w0 = C0 * u / fluid.rho / collider_fluid.rho;
                    let wp = w0 * CP * (fluid.rho + collider_fluid.rho - (2. * RHO0)) * u / q;
                    let wv = w0 * CV;

                    let dvx = velocity.x - collider_velocity.x;
                    let dvy = velocity.y - collider_velocity.y;

                    let acceleration = Vec2::new(wp * dx + wv * dvx, wp * dy + wv * dvy);
                    acceleration_accumulator.0 +=
                        acceleration.clamp(Vec2::splat(-100.), Vec2::splat(100.));
                }
            });
            //info!("acceleration_delta: {}", acceleration_accumulator.0);
        },
    );
}

//#[system(Update::Fluid::ApplyAcceleration)]
fn apply_acceleration(
    mut particles: Query<(&mut LinearVelocity, &AccelerationAccumulator)>,
    time: Res<Time>,
) {
    let time_delta_seconds = time.delta_secs();
    particles
        .par_iter_mut()
        .for_each(|(mut velocity, acceleration_accumulator)| {
            velocity.0 += acceleration_accumulator.0 * time_delta_seconds;
        });
}

#[derive(Default)]
struct One {
    blah: f32,
}

#[derive(Default)]
struct Two {
    foo: f32,
}

fn experiment() {
    let mut foo = [
        (One::default(), Two::default()),
        (One::default(), Two::default()),
    ];

    let one = &mut foo[0].0;
    let two = &foo[1].1;

    info!("{}", one.blah);
    info!("{}", two.foo);
}
