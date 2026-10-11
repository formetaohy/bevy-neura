use crate::Phase;
use crate::common::Loading;
use crate::display::Display;
use crate::learner::Learner;
use crate::policy::Sampler;
use crate::rollout::ENVS;
use crate::train::Training;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const PLAN: [(&str, f32); 3] = [
    ("compiling the policy", 0.5),
    ("sharing the policy", 0.3),
    ("waking the showcase", 0.2),
];

#[derive(Resource)]
pub(crate) struct Pending {
    learner: Option<Learner>,
    training: Option<Training>,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(Loading::of(&PLAN));
    commands.insert_resource(Pending {
        learner: None,
        training: None,
    });
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
            pending.learner = Some(Learner::attach(&runtime));
            page.report(1.0);
        }
        1 => {
            let learner = pending
                .learner
                .take()
                .expect("the page of lunar compiles the policy it holds");
            let sampler = Sampler::share(&runtime, learner.model(), ENVS);
            pending.training = Some(Training::of(learner, sampler));
            page.report(1.0);
        }
        2 => {
            let training = pending
                .training
                .take()
                .expect("the page of lunar shows the training it holds");
            commands.insert_resource(Display::build(&runtime, &training));
            commands.insert_resource(training);
            commands.remove_resource::<Pending>();
            page.report(1.0);
            phase.set(Phase::Ready);
        }
        steps => panic!("the page of lunar walks {} steps, not {steps}", PLAN.len()),
    }
}
