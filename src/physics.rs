mod cell;
mod grid;
mod solid;
mod fluid;

pub mod prelude {
    pub use super::{cell::prelude::*, grid::prelude::*, solid::prelude::*};
}
