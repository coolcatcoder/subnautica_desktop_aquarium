use crate::prelude::*;

pub mod prelude {
    pub use super::{Tool, ToolBarHovered, UiCamera};
}

#[derive(Component)]
pub struct UiCamera;

/// The selected tool.
/// This does not include their settings. That is stored separately, so that settings can persist between tool changes.
#[init]
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub enum Tool {
    None,
    #[default]
    Draw,
    Water,
}

#[init]
#[derive(Resource, Default)]
pub struct ToolBarHovered(pub bool);

#[derive(Component)]
struct Root;

#[derive(Component)]
struct ToolButton(Tool);

#[system(Update)]
fn tool_bar_setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut finished: Local<bool>,
    camera: Option<Single<Entity, With<UiCamera>>>,
) {
    if *finished {
        return;
    }

    let Some(camera) = camera else {
        return;
    };

    *finished = true;

    let mut root = commands.spawn((Root, TargetCamera(*camera), Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Start,
        justify_content: JustifyContent::Center,
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        ..default()
    }));

    [("Draw", Tool::Draw), ("Water", Tool::Water)]
        .into_iter()
        .for_each(|(text, tool)| {
            root.with_child((
                Text::new(text),
                ToolButton(tool),
                Button,
                Outline::new(Val::Percent(5.), Val::Percent(0.), Color::BLACK),
                TextFont {
                    font: asset_server.load("fonts/domine.ttf"),
                    font_size: 25.,
                    ..default()
                },
            ));
        });
}

#[system(Update)]
fn tool_bar(
    mut tool: ResMut<Tool>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor, &ToolButton), With<Button>>,
    mut tool_bar_hovered: ResMut<ToolBarHovered>,
) {
    tool_bar_hovered.0 = false;
    buttons
        .iter_mut()
        .for_each(|(interaction, mut colour, tool_button)| match interaction {
            Interaction::Pressed => {
                tool_bar_hovered.0 = true;
                colour.0 = Srgba::gray(0.1).into();
                *tool = tool_button.0;
            }
            Interaction::Hovered => {
                tool_bar_hovered.0 = true;
                colour.0 = Srgba::gray(0.2).into();
            }
            Interaction::None => {
                if *tool == tool_button.0 {
                    colour.0 = Srgba::gray(0.1).into();
                } else {
                    colour.0 = Srgba::gray(0.4).into();
                }
            }
        });
}

#[system(Update)]
fn tool_bar_visibility(
    mut interactable: ResMut<Interactable>,
    actions: Actions,
    mut tool: ResMut<Tool>,
    visibility: Option<Single<&mut Visibility, With<Root>>>,
) {
    if !actions.just_pressed(&Action::ToggleEditor) {
        return;
    }

    let Some(mut visibility) = visibility else {
        return;
    };

    if matches!(*tool, Tool::None) {
        interactable.0 = true;
        *tool = Tool::Draw;
        **visibility = Visibility::Visible;
    } else {
        interactable.0 = false;
        *tool = Tool::None;
        **visibility = Visibility::Hidden;
    }
}
