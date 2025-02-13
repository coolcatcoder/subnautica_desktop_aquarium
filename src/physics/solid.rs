use crate::prelude::*;

pub mod prelude {
    pub use super::Solid;
}

/// A wall.
/// No fluid can flow into it, and any trapped fluid will immediately flow out.
pub struct Solid {
    pub colour: Srgba,

    /// The entity that has the  square mesh, material, and transform on it.
    /// If None, then it needs to be set up.
    /// Perhaps temporary?
    pub render_entity: Option<Entity>,
}

/// Handles needed for rendering.
#[derive(Resource)]
struct MeshesAndMaterials {
    square_mesh: Handle<Mesh>,
    colour_materials: HashMap<[ordered_float::NotNan<f32>; 4], Handle<ColorMaterial>>,
}

#[system(Startup)]
fn create_meshes_and_materials(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    commands.insert_resource(MeshesAndMaterials {
        square_mesh: meshes.add(Rectangle::new(Cell::SIZE, Cell::SIZE)),
        colour_materials: default(),
    });
}

#[system(Update)]
fn render(
    mut grids: Query<(&mut Grid, &RenderLayers)>,
    mut meshes_and_materials: ResMut<MeshesAndMaterials>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // TODO: Consider running in parallel. This may be difficult due to the hashmap in meshes_and_materials.
    grids.iter_mut().for_each(|(mut grid, render_layers)| {
        // We can't access &grid when iterating the cells, so we instead store the required information in variables.
        let grid_width = grid.grid_size.x as f32;
        let grid_origin = grid.origin;

        grid.grid.iter_mut().enumerate().for_each(|(index, cell)| {
            let Some(solid) = &mut cell.solid else {
                return;
            };

            if solid.render_entity.is_some() {
                return;
            }

            // SAFETY: Index is always valid because we are getting it from enumerating the iteration of the grid.
            let translation = unsafe {
                Grid::index_to_translation_unchecked(grid_width, grid_origin, index as f32)
            };

            let colour_not_nan = [
                ordered_float::NotNan::new(solid.colour.red).unwrap(),
                ordered_float::NotNan::new(solid.colour.green).unwrap(),
                ordered_float::NotNan::new(solid.colour.blue).unwrap(),
                ordered_float::NotNan::new(solid.colour.alpha).unwrap(),
            ];

            if !meshes_and_materials
                .colour_materials
                .contains_key(&colour_not_nan)
            {
                meshes_and_materials.colour_materials.insert(
                    colour_not_nan,
                    asset_server.add(ColorMaterial::from_color(solid.colour)),
                );
                info!("New colour material.");
            }

            solid.render_entity = Some(
                commands
                    .spawn((
                        render_layers.clone(),
                        Mesh2d(meshes_and_materials.square_mesh.clone()),
                        MeshMaterial2d(
                            meshes_and_materials
                                .colour_materials
                                .get(&colour_not_nan)
                                .unwrap()
                                .clone(),
                        ),
                        Transform::from_translation(Vec3::new(translation.x, translation.y, 0.)),
                    ))
                    .id(),
            );
        });
    });
}
