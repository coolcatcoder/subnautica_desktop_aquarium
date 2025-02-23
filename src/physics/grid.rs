use crate::prelude::*;

pub mod prelude {
    pub use super::Grid;
}

/// The region that a grid takes up.
pub struct Region {
    /// The bottom left corner world translation.
    origin: Vec2,
    /// Size in cells.
    pub size: UVec2,
}

impl Region {
    /// Convert from a translation in world space to an index for a flattened array that represents the grid.
    /// Returns None if the translation is outside the grid.
    pub fn translation_to_index(&self, translation: Vec2) -> Option<usize> {
        if translation.x < self.origin.x || translation.y < self.origin.y {
            return None;
        }

        // We remove origin so that (0,0) is the origin for our translation.
        // We then divide by Cell::SIZE so that 1 unit is 1 cell.
        // Rounding makes sure that the closest cell is found.
        let grid_translation = ((translation - self.origin) / Cell::SIZE)
            .round()
            .as_uvec2();

        if grid_translation.x >= self.size.x || grid_translation.y >= self.size.y {
            return None;
        }

        let index = grid_translation.y * self.size.x + grid_translation.x;
        Some(index as usize)
    }

    /// Converts the grid index to a translation.
    /// Returns None if the index is outside the grid.
    pub fn index_to_translation(&self, index: usize) -> Option<Vec2> {
        if index >= self.size.x as usize * self.size.y as usize {
            return None;
        }

        let index = index as f32;
        let width = self.size.x as f32;

        // We add origin at the end to put it back in world space.
        let translation = Vec2::new(
                // If you imagine every index in 1 line, then if you wrap the index back to 0 every time it reaches width, you will have x.
                index % width,
                // TODO: Explain how this works.
                (index / width).floor()
            )
            // Converts 1 unit back into being Cell::SIZE.
            * Cell::SIZE
            // Converts the origin from [0,0] to the actual origin.
            + self.origin;

        Some(translation)
    }
}

/// A grid for fluids.
#[derive(Component)]
pub struct Grid {
    region: Region,

    /// The grid cells are stored as components on an entity.
    /// This means we don't have to deal with unsafety. Potentially at the cost of performance.
    cells: Box<[Entity]>,
}

impl Grid {
    /// The bottom left corner of the grid.
    pub fn origin(&self) -> Vec2 {
        self.region.origin
    }

    /// Gets the cell that the translation is inside.
    /// Returns None if the translation is outside the grid.
    pub fn get(&self, translation: Vec2) -> Option<Entity> {
        let index = self.region.translation_to_index(translation)?;
        self.cells.get(index).copied()
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

        let region = Region {
            origin,
            size: grid_size,
        };

        let cell_entities: Box<[Entity]> = (0..(grid_height * grid_width))
            .map(|_| commands.spawn_empty().id())
            .collect();

        let cells = cell_entities
            .iter()
            .enumerate()
            .map(|(index, cell_entity)| {
                let cell_entity = *cell_entity;

                // Index is part of the grid, so this will not panic.
                let translation = region.index_to_translation(index).unwrap();

                let top = region
                    .translation_to_index(translation + Vec2::new(0., Cell::SIZE))
                    .and_then(|index| cell_entities.get(index).copied());
                let left = region
                    .translation_to_index(translation + Vec2::new(-Cell::SIZE, 0.))
                    .and_then(|index| cell_entities.get(index).copied());
                let right = region
                    .translation_to_index(translation + Vec2::new(Cell::SIZE, 0.))
                    .and_then(|index| cell_entities.get(index).copied());
                let bottom = region
                    .translation_to_index(translation + Vec2::new(0., -Cell::SIZE))
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

        commands
            .entity(window_entity)
            .insert(Grid { region, cells });
    });
}

#[system(Update)]
fn debug(grids: Query<&Grid>, mut gizmos: Gizmos) {
    grids.iter().for_each(|grid| {
        gizmos.circle_2d(grid.origin(), 10., Srgba::RED);
    });
}
