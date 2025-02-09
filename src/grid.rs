use crate::prelude::*;

pub mod prelude {
    pub use super::{SetTile, TileConfig};
}

#[derive(Clone, Default)]
enum Tile {
    #[default]
    Empty,
    Fluid,
    Solid,
}

/// Handles needed for rendering tiles.
#[derive(Resource)]
struct TileMeshesAndMaterials {
    square_mesh: Handle<Mesh>,
    colour_materials: HashMap<[ordered_float::NotNan<f32>; 4], Handle<ColorMaterial>>,
}

#[system(Startup)]
fn create_tile_meshes(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(TileMeshesAndMaterials {
        square_mesh: meshes.add(Rectangle::new(Grid::CELL_SIZE, Grid::CELL_SIZE)),
        colour_materials: default(),
    });
}

pub enum TileConfig {
    Solid { colour: Color },
}

#[init]
#[derive(Event)]
pub struct SetTile {
    pub window: Entity,

    pub translation: Vec2,
    pub tile_config: TileConfig,
}

#[system(Update)]
fn set_tile(mut set_tile: EventReader<SetTile>, mut grids: Query<&mut Grid>) {
    set_tile.read().for_each(|set_tile| {
        let Ok(mut grid) = grids.get_mut(set_tile.window) else {
            error!("Entity provided in SetTile did not have a Grid.");
            return;
        };

        let Some(index) = grid.translation_to_index(set_tile.translation) else {
            error!("Translation provided in SetTile did not return an index.");
            return;
        };

        grid.grid[index].tile = match set_tile.tile_config {
            TileConfig::Solid { .. } => Tile::Solid,
        };
    });
}

/// We replace (if it exists) the entity with a new one designed to render that specific tile.
#[system(Update)]
fn render_set_tile(
    mut tile_meshes_and_materials: ResMut<TileMeshesAndMaterials>,
    mut set_tile: EventReader<SetTile>,
    mut grids: Query<&mut Grid>,
    mut commands: Commands,
    mut colour_materials: ResMut<Assets<ColorMaterial>>,
    render_layers: Query<&RenderLayers>,
) {
    set_tile.read().for_each(|set_tile| {
        let Ok(mut grid) = grids.get_mut(set_tile.window) else {
            error!("Entity provided in SetTile did not have a Grid.");
            return;
        };

        let Some(index) = grid.translation_to_index(set_tile.translation) else {
            error!("Translation provided in SetTile did not return an index.");
            return;
        };
        // This will never panic, as we have made sure that the translation_to_index worked above.
        let cell_translation = grid.index_to_translation(index).unwrap();

        let cell = &mut grid.grid[index];

        if let Some(entity) = cell.entity.take() {
            if let Some(mut entity) = commands.get_entity(entity) {
                entity.despawn();
            }
        }

        match set_tile.tile_config {
            TileConfig::Solid { colour } => {
                let colour_srgba = colour.to_srgba();
                let colour_not_nan = [
                    ordered_float::NotNan::new(colour_srgba.red).unwrap(),
                    ordered_float::NotNan::new(colour_srgba.green).unwrap(),
                    ordered_float::NotNan::new(colour_srgba.blue).unwrap(),
                    ordered_float::NotNan::new(colour_srgba.alpha).unwrap(),
                ];

                if !tile_meshes_and_materials
                    .colour_materials
                    .contains_key(&colour_not_nan)
                {
                    tile_meshes_and_materials
                        .colour_materials
                        .insert(colour_not_nan, colour_materials.add(colour));
                }

                let Ok(render_layers) = render_layers.get(set_tile.window) else {
                    error!("Could not find correct render layers.");
                    return;
                };

                cell.entity = Some(
                    commands
                        .spawn((
                            render_layers.clone(),
                            Mesh2d(tile_meshes_and_materials.square_mesh.clone()),
                            MeshMaterial2d(
                                tile_meshes_and_materials
                                    .colour_materials
                                    .get(&colour_not_nan)
                                    .unwrap()
                                    .clone(),
                            ),
                            Transform::from_translation(Vec3::new(
                                cell_translation.x,
                                cell_translation.y,
                                0.,
                            )),
                        ))
                        .id(),
                );
            }
        }
    });
}

#[derive(Default, Clone)]
struct GridCell {
    tile: Tile,
    entity: Option<Entity>,
    // Perhaps we can just store a vec of rigidbody entities to advect along the fluid velocities, and use avian?
}

#[derive(Component)]
struct Grid {
    origin: Vec2,
    // size in cells
    grid_size: UVec2,

    grid: Box<[GridCell]>,
}

impl Grid {
    const CELL_SIZE: f32 = 30.;

    /// Convert from a translation in world space to an index in grid.
    fn translation_to_index(&self, translation: Vec2) -> Option<usize> {
        if translation.x < self.origin.x || translation.y < self.origin.y {
            return None;
        }

        // We remove origin so that (0,0) is the origin for our translation.
        let corrected_translation = (translation - self.origin) / Self::CELL_SIZE;
        //info!("{}", corrected_translation);

        if corrected_translation.x >= self.grid_size.x as f32
            || corrected_translation.y >= self.grid_size.y as f32
        {
            return None;
        }

        let float_index = corrected_translation.y.trunc() * self.grid_size.x as f32
            + corrected_translation.x.trunc();
        //info!(float_index);
        Some(float_index.trunc() as usize)
    }

    fn index_to_translation(&self, index: usize) -> Option<Vec2> {
        if index >= self.grid.len() {
            return None;
        }

        let float_index = index as f32;
        let grid_width = self.grid_size.x as f32;

        // We add origin at the end to put it back in world space.
        let translation =
            Vec2::new(float_index % grid_width, float_index / grid_width) * Grid::CELL_SIZE + self.origin;
        Some(translation)
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
        let grid_height = (height / Grid::CELL_SIZE).ceil() as usize;
        let grid_width = (width / Grid::CELL_SIZE).ceil() as usize;

        commands.entity(window_entity).insert(Grid {
            origin,
            grid_size: UVec2::new(grid_width as u32, grid_height as u32),

            grid: vec![default(); grid_height * grid_width].into_boxed_slice(),
        });
    });
}

#[system(Update)]
fn debug(grids: Query<&Grid>, mut gizmos: Gizmos) {
    grids.iter().for_each(|grid| {
        gizmos.circle_2d(grid.origin, 10., Srgba::RED);
    });
}
