mod cell;
mod fluid;
mod grid;
mod solid;

pub mod prelude {
    pub use super::{cell::prelude::*, fluid::prelude::*, grid::prelude::*, solid::prelude::*};
}
