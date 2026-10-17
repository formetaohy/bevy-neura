use crate::game::Game;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;
use bevy_neura_example::app::Phase;
use bevy_neura_example::loading::Loading;

const PLAN: [(&str, f32); 2] = [("building the players", 0.85), ("opening the board", 0.15)];

#[derive(Resource)]
pub(crate) struct Pending {
    game: Option<Game>,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(Loading::of(&PLAN));
    commands.insert_resource(Pending { game: None });
}

pub fn drive(
    runtime: Res<NeuraRuntime>,
    mut commands: Commands,
    mut page: ResMut<Loading>,
    mut pending: ResMut<Pending>,
    mut phase: ResMut<NextState<Phase>>,
) {
    if !page.shown() {
        return;
    }
    match page.index() {
        0 => {
            pending.game = Some(Game::attach(&runtime));
            page.report(1.0);
        }
        1 => {
            let game = pending
                .game
                .take()
                .expect("the page of tictactoe holds the game it builds");
            commands.insert_resource(game);
            commands.remove_resource::<Pending>();
            page.report(1.0);
            phase.set(Phase::Ready);
        }
        steps => panic!(
            "the page of tictactoe walks {} steps, not {steps}",
            PLAN.len()
        ),
    }
}
