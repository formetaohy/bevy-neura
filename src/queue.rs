use crate::runtime::NeuraRuntime;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::{App, Commands, First, Res, ResMut, Resource};
use bevy::render::extract_plugin::ExtractSchedule;
use bevy::render::{Extract, Render, RenderApp, RenderSystems};
use neura::{Queue, QueueLock};

#[derive(Resource)]
struct NeuraQueue(Queue);

#[derive(Resource, Default)]
struct QueueTurn(Option<QueueLock>);

pub(crate) fn hold(app: &mut App) {
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    if render.world().contains_resource::<QueueTurn>() {
        return;
    }
    render.init_resource::<QueueTurn>();
    render.add_systems(ExtractSchedule, pass_queue);
    render.add_systems(First, take_turn);
    render.add_systems(Render, release_turn.after(RenderSystems::PostCleanup));
}

fn pass_queue(mut commands: Commands, runtime: Extract<Res<NeuraRuntime>>) {
    commands.insert_resource(NeuraQueue(runtime.context().queue().clone()));
}

fn take_turn(queue: Option<Res<NeuraQueue>>, mut turn: ResMut<QueueTurn>) {
    let Some(queue) = queue else {
        return;
    };
    if turn.0.is_none() {
        turn.0 = Some(queue.0.lock());
    }
}

fn release_turn(mut turn: ResMut<QueueTurn>) {
    turn.0 = None;
}
