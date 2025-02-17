use crate::prelude::*;

pub mod prelude {
    pub use super::Grid;
}

#[derive(Component)]
pub struct Grid {
    origin: Vec2,
    /// size in cells
    pub grid_size: UVec2,

    /// The grid cells are stored as components on an entity.
    /// This means we don't have to deal with unsafety. Potentially at the cost of performance.
    cells: Box<[Entity]>,
}

impl Grid {
    pub fn get(&self, translation: Vec2) -> Option<Entity> {
        let index = self.translation_to_index(translation)?;
        self.cells.get(index).copied()
    }

    /// Convert from a translation in world space to an index in cells.
    fn translation_to_index(&self, translation: Vec2) -> Option<usize> {
        Self::translation_to_index_no_self(self.origin, self.grid_size, translation)
    }

    /// Convert from a translation in world space to an index.
    /// No Self.
    fn translation_to_index_no_self(
        grid_origin: Vec2,
        grid_size: UVec2,
        translation: Vec2,
    ) -> Option<usize> {
        if translation.x < grid_origin.x || translation.y < grid_origin.y {
            return None;
        }

        // We remove origin so that (0,0) is the origin for our translation.
        // We then divide by Cell::SIZE so that 1 unit is 1 cell.
        // Rounding makes sure that the closest cell is found.
        let grid_translation = ((translation - grid_origin) / Cell::SIZE)
            .round()
            .as_uvec2();

        if grid_translation.x >= grid_size.x || grid_translation.y >= grid_size.y {
            return None;
        }

        let index = grid_translation.y * grid_size.x + grid_translation.x;
        Some(index as usize)
    }

    /// Converts the grid index to a translation.
    /// Returns None if the index is >= self.grid.len().
    fn index_to_translation(&self, index: usize) -> Option<Vec2> {
        if index >= self.cells.len() {
            return None;
        }

        let float_index = index as f32;
        let grid_width = self.grid_size.x as f32;

        Some(unsafe { Grid::index_to_translation_unchecked(grid_width, self.origin, float_index) })
    }

    /// Converts the grid index to a translation.
    /// This does not check if the index is < len().
    /// Does not ask for &self, so that this can be used while iterating the grid.
    pub unsafe fn index_to_translation_unchecked(
        grid_width: f32,
        grid_origin: Vec2,
        float_index: f32,
    ) -> Vec2 {
        // We add origin at the end to put it back in world space.
        Vec2::new(float_index % grid_width, (float_index / grid_width).floor()) * Cell::SIZE
            + grid_origin
    }
}

#[system(Update)]
fn setup(
    mut commands: Commands,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut finished: Local<bool>,
    mut windowing_done: EventReader<WindowingDone>,
    winit_windows: NonSend<WinitWindows>,
) {
    if *finished {
        return;
    }

    if windowing_done.read().next().is_none() {
        return;
    }

    *finished = true;

    cameras.iter().for_each(|(camera, global_transform)| {
        let RenderTarget::Window(WindowRef::Entity(window_entity)) = camera.target else {
            return;
        };

        let Some(window_winit) = winit_windows.get_window(window_entity) else {
            return;
        };

        let size = window_winit
            .outer_size()
            .to_logical(window_winit.scale_factor());
        let height: f32 = size.height;
        let width = size.width;

        let origin = match camera.ndc_to_world(global_transform, Vec3::new(-1., -1., 0.)) {
            Some(origin) => origin.xy(),
            None => {
                error!("Something contained NaN.");
                return;
            }
        };

        // Divide the height and width by the size, to get the number of cells needed.
        let grid_height = (height / Cell::SIZE).ceil() as usize;
        let grid_width = (width / Cell::SIZE).ceil() as usize;

        let grid_size = UVec2::new(grid_width as u32, grid_height as u32);

        let cell_entities: Box<[Entity]> = (0..(grid_height * grid_width))
            .map(|_| commands.spawn_empty().id())
            .collect();

        let cells = cell_entities
            .iter()
            .enumerate()
            .map(|(index, cell_entity)| {
                let cell_entity = *cell_entity;

                // SAFETY: Index is within the grid.
                let translation = unsafe {
                    Grid::index_to_translation_unchecked(grid_width as f32, origin, index as f32)
                };

                let top = Grid::translation_to_index_no_self(
                    origin,
                    grid_size,
                    translation + Vec2::new(0., Cell::SIZE),
                )
                .and_then(|index| cell_entities.get(index).copied());
                let left = Grid::translation_to_index_no_self(
                    origin,
                    grid_size,
                    translation + Vec2::new(-Cell::SIZE, 0.),
                )
                .and_then(|index| cell_entities.get(index).copied());
                let right = Grid::translation_to_index_no_self(
                    origin,
                    grid_size,
                    translation + Vec2::new(Cell::SIZE, 0.),
                )
                .and_then(|index| cell_entities.get(index).copied());
                let bottom = Grid::translation_to_index_no_self(
                    origin,
                    grid_size,
                    translation + Vec2::new(0., -Cell::SIZE),
                )
                .and_then(|index| cell_entities.get(index).copied());

                commands
                    .entity(cell_entity)
                    .insert(Cell {
                        grid: window_entity,

                        index,
                        translation,

                        nearest_4: [top, left, right, bottom],
                    })
                    .id()
            })
            .collect();

        commands.entity(window_entity).insert(Grid {
            origin,
            grid_size,

            cells,
        });
    });
}

#[system(Update)]
fn debug(grids: Query<&Grid>, mut gizmos: Gizmos) {
    grids.iter().for_each(|grid| {
        gizmos.circle_2d(grid.origin, 10., Srgba::RED);
    });
}
