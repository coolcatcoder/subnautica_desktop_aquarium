use crate::prelude::*;

#[derive(Resource)]
struct DrawSettings {
    brush: Handle<Image>,
    colour: Color,
    // Scales the dimensions of the brush by this much.
    scale: f32,
    // How deep in the scene the terrain will be.
    depth: f32,
    // Squishes the brush strokes together. I'm not sure how.
    squish: f32,
    collision: bool,
}
#[system(Startup)]
fn draw_settings(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(DrawSettings {
        brush: asset_server.load("brushes/circle.png"),
        colour: Color::BLACK,
        scale: 0.1,
        depth: 0.,
        squish: 0.3,
        collision: true,
    });
}

#[init]
#[derive(Resource, Default)]
struct DrawnPointPreviousTranslation(Vec2);

#[system(Update)]
fn draw_terrain(
    tool: Res<Tool>,
    actions: Actions,
    cursor_translation: Res<CursorTranslation>,
    mut previous_translation: ResMut<DrawnPointPreviousTranslation>,
    mut commands: Commands,
    settings: Res<DrawSettings>,
    images: Res<Assets<Image>>,
    tool_bar_hovered: Res<ToolBarHovered>,
    mut set_tile: EventWriter<SetTile>,
) {
    if !matches!(*tool, Tool::Draw) {
        return;
    }

    if tool_bar_hovered.0 {
        return;
    }

    if !actions.pressed(&Action::Use) {
        return;
    }

    let Some(cursor_translation) = &cursor_translation.0 else {
        return;
    };

    let Some(image) = images.get(settings.brush.id()) else {
        return;
    };

    let size = image.size_f32();

    // If we just clicked somewhere, we spawn a terrain point, and set the previous translation to be at the cursor.
    // If we don't do this, then we get a cool straight line effect.
    if actions.just_pressed(&Action::Use) {
        spawn_terrain(
            &mut commands,
            cursor_translation.translation,
            &settings,
            size,
        );
        set_tile.send(SetTile {
            window: cursor_translation.window,

            translation: cursor_translation.translation,
            tile_config: TileConfig::Solid {
                colour: Color::BLACK,
            },
        });
        previous_translation.0 = cursor_translation.translation;
    }

    let radius_average_squished = (size.x + size.y) / 2. * settings.scale * settings.squish;

    // Create points until we reach the cursor translation.
    loop {
        let distance_squared = cursor_translation
            .translation
            .distance_squared(previous_translation.0);

        if distance_squared < (radius_average_squished * radius_average_squished) {
            break;
        }

        let distance = distance_squared.sqrt();

        // Gets, and normalises the direction.
        let direction = (cursor_translation.translation - previous_translation.0) / distance;

        // Move the previous translation in the correct direction.
        previous_translation.0 += direction * radius_average_squished;

        spawn_terrain(&mut commands, previous_translation.0, &settings, size);
    }
}

fn spawn_terrain(commands: &mut Commands, translation: Vec2, settings: &DrawSettings, size: Vec2) {
    let mut terrain = commands.spawn((
        Transform::from_translation(Vec3::new(translation.x, translation.y, settings.depth)),
        Sprite {
            image: settings.brush.clone(),
            color: settings.colour,
            custom_size: Some(size * settings.scale),
            ..default()
        },
    ));

    if settings.collision {
        terrain.insert((
            RigidBody::Static,
            Collider::circle((size.x + size.y) / 2. / 2. * settings.scale),
        ));
    }
}

#[derive(Component)]
struct Root;

#[system(Update)]
fn ui(
    cursor_translation: Res<CursorTranslation>,
    mut commands: Commands,
    mut finished: Local<bool>,
) {
    if *finished {
        return;
    }

    let Some(cursor_translation) = &cursor_translation.0 else {
        return;
    };

    *finished = true;

    let mut _root = commands.spawn((Root, TargetCamera(cursor_translation.window), Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Start,
        justify_content: JustifyContent::Center,
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        ..default()
    }));

    // TODO: Allow editing of settings via ui.
}

#[system(Update)]
fn ui_visibility(tool: Res<Tool>, visibility: Option<Single<&mut Visibility, With<Root>>>) {
    let Some(mut visibility) = visibility else {
        return;
    };

    if matches!(*tool, Tool::Draw) {
        **visibility = Visibility::Visible;
    } else {
        **visibility = Visibility::Hidden;
    }
}
