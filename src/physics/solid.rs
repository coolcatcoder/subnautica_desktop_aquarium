use crate::prelude::*;

pub mod prelude {
    pub use super::{SetSolid, Solid};
}

/// A wall.
/// No fluid can flow into it, and any trapped fluid will immediately flow out.
#[derive(Component)]
pub struct Solid;

#[init]
#[derive(Event)]
pub struct SetSolid {
    pub window: Entity,
    pub translation: Vec2,
    pub colour: Srgba,
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
    mut set_solid: EventReader<SetSolid>,
    grids: Query<(&Grid, &RenderLayers)>,
    cells: Query<&Cell, Without<Solid>>,
    mut meshes_and_materials: ResMut<MeshesAndMaterials>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    set_solid.read().for_each(|set_solid| {
        let Ok((grid, render_layers)) = grids.get(set_solid.window) else {
            return;
        };

        let Some(cell_entity) = grid.get(set_solid.translation) else {
            return;
        };

        let Ok(cell) = cells.get(cell_entity) else {
            return;
        };

        let colour_not_nan = [
            ordered_float::NotNan::new(set_solid.colour.red).unwrap(),
            ordered_float::NotNan::new(set_solid.colour.green).unwrap(),
            ordered_float::NotNan::new(set_solid.colour.blue).unwrap(),
            ordered_float::NotNan::new(set_solid.colour.alpha).unwrap(),
        ];

        if !meshes_and_materials
            .colour_materials
            .contains_key(&colour_not_nan)
        {
            meshes_and_materials.colour_materials.insert(
                colour_not_nan,
                asset_server.add(ColorMaterial::from_color(set_solid.colour)),
            );
            info!("New colour material.");
        }

        commands
            .entity(cell_entity)
            .insert((
                Solid,
                render_layers.clone(),
                Mesh2d(meshes_and_materials.square_mesh.clone()),
                MeshMaterial2d(
                    meshes_and_materials
                        .colour_materials
                        .get(&colour_not_nan)
                        .unwrap()
                        .clone(),
                ),
                Transform::from_translation(Vec3::new(cell.translation.x, cell.translation.y, 0.)),
            ))
            .id();
    });
}
