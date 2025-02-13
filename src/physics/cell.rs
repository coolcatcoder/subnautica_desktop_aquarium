use crate::prelude::*;

pub mod prelude {
    pub use super::Cell;
}

/// A grid cell.
// TODO: Consider using just an entity, that then could have Solid, Fluid, etc.
// My intuition is that it would be slower, but it should be profiled.
pub struct Cell {
    pub solid: Option<Solid>,
    // TODO: Perhaps we can just store a vec of rigidbody entities to advect along the fluid velocities, and use avian?
    // Probably easier to take their translations, convert it to an index, and then advect using that cell's velocity.
}

impl Cell {
    pub const SIZE: f32 = 30.;
    pub const EMPTY: Self = Self { solid: None };
}
