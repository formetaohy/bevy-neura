use crate::plan::{ModelPlan, Roles};
use crate::report::ModelGeometry;
use crate::role::Role;
use crate::sample::{Loss, Sample};
use neura::{Checkpoint, Graph, Profile, Program, Readout, Runtime, Value, Weights};
use std::path::Path;

pub(crate) struct Model<'r> {
    graph: Graph<'static>,
    roles: Roles,
    weights: Weights<'r>,
    program: Program<'r>,
}

impl<'r> Model<'r> {
    pub(crate) fn attach(runtime: &'r Runtime, plan: ModelPlan) -> Self {
        let ModelPlan { graph, roles, .. } = plan;
        for value in roles.retains() {
            graph.retain(value);
        }
        let weights = runtime.weights(&graph);
        let program = runtime.compile(&graph, &weights);
        Self {
            graph,
            roles,
            weights,
            program,
        }
    }

    pub(crate) fn share(runtime: &'r Runtime, weights: Weights<'r>, plan: ModelPlan) -> Self {
        let ModelPlan { graph, roles, .. } = plan;
        for value in roles.retains() {
            graph.retain(value);
        }
        runtime.rebind(&weights, &graph);
        let program = runtime.compile(&graph, &weights);
        Self {
            graph,
            roles,
            weights,
            program,
        }
    }

    pub(crate) fn weights(&self) -> Weights<'r> {
        self.weights.clone()
    }

    pub(crate) fn geometry(&self) -> ModelGeometry {
        ModelGeometry {
            tasks: self.program.task_count(),
            waves: self.program.wave_count(),
            workgroups: self.program.workgroups(),
            tensors: self.program.tensor_bytes(),
            weights: self.program.weights().bytes(),
            arena: self.program.arena_bytes(),
            device: self.program.device_bytes(),
            updates: self.program.updates_weights(),
            dynamic: self.program.dynamic(),
            bounds: self.roles.bounds(),
        }
    }

    pub(crate) fn train(
        &self,
        runtime: &Runtime,
        lengths: &[u32],
        samples: &[Sample],
        loss: Loss,
    ) -> (Vec<f32>, f64) {
        self.bind(runtime, lengths);
        let reported = self.roles.loss_value();
        assert!(
            loss == Loss::None || reported.is_some(),
            "a training reads the loss a model declares, and this model declares none",
        );
        let mut losses = Vec::new();
        let mut pending: Option<Readout<'_>> = None;
        let mut seconds = 0.0;
        for (index, sample) in samples.iter().enumerate() {
            for (role, data) in sample.writes() {
                runtime.write(&self.program, self.roles.input_value(role), data);
            }
            let last = index + 1 == samples.len();
            let run = runtime.run(&self.program);
            if last {
                seconds = run.seconds();
            }
            let reporting = match (loss, reported) {
                (Loss::Every, Some(value)) => Some(value),
                (Loss::Final, Some(value)) if last => Some(value),
                _ => None,
            };
            if let Some(value) = reporting {
                let readout = runtime.pull(&self.program, &[value]);
                if let Some(previous) = pending.replace(readout) {
                    losses.push(loss_of(runtime, previous));
                }
            }
        }
        if let Some(pending) = pending {
            losses.push(loss_of(runtime, pending));
        }
        (losses, seconds)
    }

    pub(crate) fn infer(
        &self,
        runtime: &Runtime,
        lengths: &[u32],
        sample: &Sample,
    ) -> (Vec<(Role, Vec<f32>)>, f64) {
        self.bind(runtime, lengths);
        for (role, data) in sample.writes() {
            runtime.write(&self.program, self.roles.input_value(role), data);
        }
        let seconds = runtime.run(&self.program).seconds();
        let roles = self.roles.output_roles();
        (self.read_roles(runtime, &roles), seconds)
    }

    pub(crate) fn read(
        &self,
        runtime: &Runtime,
        lengths: &[u32],
        roles: &[Role],
    ) -> Vec<(Role, Vec<f32>)> {
        self.bind(runtime, lengths);
        self.read_roles(runtime, roles)
    }

    pub(crate) fn tune(&mut self, runtime: &'r Runtime, lengths: &[u32]) -> (Profile, f64) {
        self.bind(runtime, lengths);
        self.program = runtime.tune(&self.graph, &self.weights);
        (self.program.profile(), runtime.measure(&self.program))
    }

    pub(crate) fn save(&self, runtime: &Runtime, path: &Path) -> u64 {
        let checkpoint = runtime.checkpoint(&self.weights);
        let bytes = checkpoint.bytes();
        std::fs::write(path, bytes).unwrap_or_else(|error| {
            panic!(
                "a checkpoint of {} bytes reaches no file at {}: {error}",
                bytes.len(),
                path.display(),
            )
        });
        bytes.len() as u64
    }

    pub(crate) fn restore(&self, runtime: &Runtime, path: &Path) {
        let bytes = std::fs::read(path)
            .unwrap_or_else(|error| panic!("no checkpoint reads from {}: {error}", path.display()));
        runtime.restore(&self.weights, &Checkpoint::decode(&bytes));
    }

    fn bind(&self, runtime: &Runtime, lengths: &[u32]) {
        if !lengths.is_empty() {
            runtime.bind(&self.program, lengths);
        }
    }

    fn read_roles(&self, runtime: &Runtime, roles: &[Role]) -> Vec<(Role, Vec<f32>)> {
        if roles.is_empty() {
            return Vec::new();
        }
        let values = roles
            .iter()
            .map(|role| self.roles.output_value(role))
            .collect::<Vec<Value<'static>>>();
        runtime
            .read_many(&self.program, &values)
            .into_iter()
            .zip(roles.iter().copied())
            .map(|(data, role)| (role, data))
            .collect()
    }
}

fn loss_of(runtime: &Runtime, readout: Readout<'_>) -> f32 {
    let mut values = runtime.collect(readout);
    let loss = values
        .pop()
        .expect("a pull of one tensor answers one tensor");
    assert_eq!(
        loss.len(),
        1,
        "a training reports the one number a loss holds, and this loss holds {}",
        loss.len(),
    );
    loss[0]
}
