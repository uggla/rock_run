use bevy::prelude::*;
use enum_iterator::{Sequence, all};

/// Component to tag an entity as only needed in some of the states
#[derive(Component, Debug)]
pub struct ForState<T> {
    pub states: Vec<T>,
}

// Main state enum
#[derive(States, Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Sequence)]
pub enum AppState {
    #[default]
    Loading,
    StartMenu,
    GameCreate,
    GameMessage,
    GameRunning,
    GamePaused,
    GameOver,
    FinishLevel,
    NextLevel,
    GameFinished,
}

#[allow(dead_code)]
impl AppState {
    const ANY_GAME_MENU: [AppState; 4] = [
        AppState::StartMenu,
        AppState::GameMessage,
        AppState::GamePaused,
        AppState::GameOver,
    ];
    pub fn is_any_game_menu(current_state: &AppState) -> bool {
        AppState::ANY_GAME_MENU.contains(current_state)
    }
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        for state in all::<AppState>() {
            app.add_systems(OnEnter(state), state_enter_despawn::<AppState>);
        }
    }
}

fn state_enter_despawn<T: States>(
    mut commands: Commands,
    state: ResMut<State<T>>,
    query: Query<(Entity, &ForState<T>)>,
) {
    for (entity, for_state) in &mut query.iter() {
        if !for_state.states.contains(state.get()) {
            commands.entity(entity).despawn();
        }
    }
}
