use crate::prelude::*;

pub mod prelude {
    pub use super::{Action, Actions};
}

app!(|app| {
    app.add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(input_map());
});

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Use,
    ToggleEditor,
}

fn input_map() -> InputMap<Action> {
    InputMap::new([(Action::Use, MouseButton::Left)]).with(Action::ToggleEditor, KeyCode::KeyE)
}

pub type Actions<'w> = Res<'w, ActionState<Action>>;
