use crate::prelude::*;

pub mod prelude {
    pub use super::Grid;
}

#[derive(Component)]
pub struct Grid {
    pub origin: Vec2,
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

    /// Convert from a translation in world space to an index in grid.
    fn translation_to_index(&self, translation: Vec2) -> Option<usize> {
        if translation.x < self.origin.x || translation.y < self.origin.y {
            return None;
        }

        // We remove origin so that (0,0) is the origin for our translation.
        let corrected_translation = (translation - self.origin) / Cell::SIZE;
        //info!("{}", corrected_translation);

        if corrected_translation.x >= self.grid_size.x as f32
            || corrected_translation.y >= self.grid_size.y as f32
        {
            return None;
        }

        let grid_translation = corrected_translation.round().as_uvec2();

        let index = grid_translation.y * self.grid_size.x + grid_translation.x;
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
    windows: Query<&Window>,
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
        let Ok(window) = windows.get(window_entity) else {
            return;
        };

        let Some(window_winit) = winit_windows.get_window(window_entity) else {
            return;
        };

        //let height = window.height();
        //let width = window.width();

        let size = window_winit
            .outer_size()
            .to_logical(window_winit.scale_factor());
        let height: f32 = size.height;
        let width = size.width;

        let origin = match camera.ndc_to_world(global_transform, Vec3::new(-1., -1., 0.)) {
            Some(origin) => origin.xy(),
            None => {
                //error!("{:?}", error);
                return;
            }
        };
        //let origin = Vec2::new(width, height)/2;

        // Divide the height and width by the size, to get the number of cells needed.
        let grid_height = (height / Cell::SIZE).ceil() as usize;
        let grid_width = (width / Cell::SIZE).ceil() as usize;

        let cells = (0..(grid_height * grid_width))
            .map(|index| {
                // SAFETY: Index is within the grid.
                let translation = unsafe {
                    Grid::index_to_translation_unchecked(grid_width as f32, origin, index as f32)
                };

                commands.spawn(Cell { index, translation }).id()
            })
            .collect();

        commands.entity(window_entity).insert(Grid {
            origin,
            grid_size: UVec2::new(grid_width as u32, grid_height as u32),

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
