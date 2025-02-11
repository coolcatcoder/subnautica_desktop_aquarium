use crate::prelude::*;

pub mod prelude {
    pub use super::{Cell};
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

    pub const fn new() -> Self {
        Self {
            solid: None,
        }
    }
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