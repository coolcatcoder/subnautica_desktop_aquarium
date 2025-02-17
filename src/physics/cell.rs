use crate::prelude::*;

pub mod prelude {
    pub use super::Cell;
}

/// A grid cell.
/// I don't fully know how this is going to work.
#[derive(Component)]
#[require(Fluid)]
pub struct Cell {
    pub grid: Entity,

    pub index: usize,
    pub translation: Vec2,

    /// The 4 nearest cells.
    /// Ordered top, left, right, bottom.
    pub nearest_4: [Option<Entity>; 4],
}

impl Cell {
    pub const SIZE: f32 = 30.;
}

/// FIXME: Abandoned, for now. Consider fixing it and profiling it.
/// Allows you to iterate grid.cells mutably, while still having access to the immutable grid.
/// To do this, we define a mutable and immutable version of Cell, that have no aliasing.
/// This macro will cause undefined behaviour if you input the same field in both the mutable case and the immutable case.
/// Example usage: (&mut (fluid.divergence, foo.bar.cake), &(gaggle.of.geese, etc.etc, etcetera.etcetera))
/// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=d098cc5ca164259bc6cee8fda57b5665
#[macro_export]
macro_rules! disjoint_iterator {
    (
        &mut ($($($mutable_field_segment: ident).+: $mutable_field_type: ty),+),
        &($($($immutable_field_segment: ident).+: $immutable_field_type: ty),+)
    ) => {
        use disjoint_iterator::DisjointIterator;
        paste::paste! {
            mod disjoint_iterator {
                /// A cell that contains only mutable fields.
                pub struct CellMutable<'a> {
                    $(
                        pub [<$($mutable_field_segment)_+>]: &'a mut $mutable_field_type
                    ),+
                }

                /// A cell that contains only immutable fields.
                pub struct CellImmutable<'a> {
                    $(
                        pub [<$($immutable_field_segment)_+>]: &'a $immutable_field_type
                    ),+
                }

                /// Gives immutable access to the grid, even while mutably iterating cells.
                pub struct GridImmutable<'a> {
                    pub origin: &'a super::Vec2,
                    /// size in cells
                    pub grid_size: &'a super::UVec2,

                    pub cells: CellsImmutable<'a>,
                }

                /// Provides immutable access to all cells.
                pub struct CellsImmutable<'a>(*const [super::Cell], std::marker::PhantomData<&'a ()>);

                impl<'a> CellsImmutable<'a> {
                    pub fn get(&self, index: usize) -> Option<CellImmutable<'a>> {
                        unsafe {
                            //let cell = &raw const *(*self.0).get(index)?;
                            let start: *const super::Cell = self.0 as *const super::Cell;

                            Some(CellImmutable {
                                $(
                                    //[<$($immutable_field_segment)_+>]: &*(&raw const (*cell).$($mutable_field_segment).+)
                                    [<$($immutable_field_segment)_+>]: &*(start.add(index).byte_add(std::mem::offset_of!(super::Cell, $($mutable_field_segment).+)) as *const $mutable_field_type)
                                ),+
                            })
                        }
                    }
                }

                pub struct DisjointIterator<'a> {
                    origin: &'a super::Vec2,
                    grid_size: &'a super::UVec2,

                    /// Cells left in the iterator.
                    cells_remaining: *mut [super::Cell],
                    /// All cells in the grid.
                    cells: *const [super::Cell],
                }

                impl<'a> DisjointIterator<'a> {
                    /// Creates a new disjointed iterator.
                    /// This is safe, if you followed the macro's safety requirements.
                    /// Otherwise, it is undefined behaviour.
                    pub unsafe fn new(grid: &'a mut super::Grid) -> Self {
                        Self {
                            origin: &grid.origin,
                            grid_size: &grid.grid_size,

                            cells: &*grid.cells,
                            cells_remaining: &mut grid.cells,
                        }
                    }
                }

                impl<'a> Iterator for DisjointIterator<'a> {
                    type Item = (CellMutable<'a>, GridImmutable<'a>);

                    fn next(&mut self) -> Option<Self::Item> {
                        // We gain ownership of the &mut, which allows us to remove the first &mut Cell and then give it back.
                        let cells_remaining = std::mem::replace(&mut self.cells_remaining, &mut []);

                        let cell = match cells_remaining {
                            [] => None,
                            [cell, cells_remaining @ ..] => {
                                self.cells_remaining = cells_remaining;
                                Some(cell)
                            }
                        }?;

                        let item = (
                            CellMutable {
                                $(
                                    [<$($mutable_field_segment)_+>]: &mut cell.$($mutable_field_segment).+
                                ),+
                            },
                            GridImmutable {
                                origin: self.origin,
                                grid_size: self.grid_size,

                                cells: CellsImmutable(self.cells, std::marker::PhantomData),
                            },
                        );

                        Some(item)
                    }
                }
            }
        }
    };
}
