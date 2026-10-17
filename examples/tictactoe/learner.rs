use crate::board::CELLS;
use crate::net::{Q_ROLE, QNet, ROWS_ROLE, STATES_ROLE, plan};
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{AdamW, Element, Shape, mse_loss};

pub const BATCH: u32 = 128;
const SYNC: u32 = 200;
const RATE: f32 = 5e-4;
const ACTIONS_ROLE: Role = "actions";
const TARGETS_ROLE: Role = "targets";
const RATE_ROLE: Role = "rate";

pub struct Learner {
    model: Model,
    target: Model,
    updates: u32,
}

impl Learner {
    pub fn attach(runtime: &NeuraRuntime) -> Self {
        let model = update_plan().attach(runtime);
        let target = plan(BATCH).attach(runtime);
        let checkpoint = model.checkpoint(runtime);
        target.restore(runtime, &checkpoint);
        Self {
            model,
            target,
            updates: 0,
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn target_q(&self, runtime: &NeuraRuntime, states: &[f32]) -> Vec<f32> {
        assert_eq!(
            states.len(),
            BATCH as usize * CELLS,
            "a target reading of {BATCH} boards walks {} numbers",
            states.len(),
        );
        self.target
            .bind(runtime, &Extents::new().axis(ROWS_ROLE, BATCH));
        self.target
            .run(runtime, &Inputs::new().write(STATES_ROLE, states));
        let q = self.target.read(runtime, Q_ROLE);
        assert_eq!(
            q.len(),
            BATCH as usize * CELLS,
            "a target reading of {BATCH} boards answers {} scores",
            q.len(),
        );
        q
    }

    pub fn update(
        &mut self,
        runtime: &NeuraRuntime,
        states: &[f32],
        actions: &[f32],
        targets: &[f32],
        rate: f32,
    ) {
        assert!(
            rate > 0.0 && rate.is_finite(),
            "a descent rate of {rate} moves nothing",
        );
        assert_eq!(
            states.len(),
            BATCH as usize * CELLS,
            "a batch of {BATCH} boards walks {} numbers",
            states.len(),
        );
        assert_eq!(
            (actions.len(), targets.len()),
            (BATCH as usize, BATCH as usize),
            "a batch of {BATCH} boards takes {} actions and {} targets",
            actions.len(),
            targets.len(),
        );
        self.model
            .bind(runtime, &Extents::new().axis(ROWS_ROLE, BATCH));
        self.model.run(
            runtime,
            &Inputs::new()
                .write(STATES_ROLE, states)
                .write(ACTIONS_ROLE, actions)
                .write(TARGETS_ROLE, targets)
                .write(RATE_ROLE, [rate]),
        );
        self.updates += 1;
        if self.updates.is_multiple_of(SYNC) {
            let checkpoint = self.model.checkpoint(runtime);
            self.target.restore(runtime, &checkpoint);
        }
    }
}

fn update_plan() -> ModelPlan {
    ModelPlan::build(|graph| {
        let rows = graph.free(BATCH);
        let states = graph.input(
            Shape::matrix(BATCH, CELLS as u32).freed(&[(2, rows)]),
            Element::Single,
        );
        let actions = graph.input(Shape::matrix(BATCH, 1).freed(&[(2, rows)]), Element::Single);
        let targets = graph.input(Shape::matrix(BATCH, 1).freed(&[(2, rows)]), Element::Single);
        let net = QNet::build(graph);
        let q = net.forward(graph, states);
        let taken = graph.sum_rows(graph.mul(q, graph.one_hot(actions, CELLS as u32)));
        let loss = mse_loss(graph, taken, targets);
        let gradients = graph.backward(loss);
        let mut optimizer = AdamW::new(graph, "optimizer", RATE, 0.9, 0.999, 1e-8, 0.0);
        optimizer.track_all(graph, &net.parameters());
        optimizer.step(graph, &gradients);
        Roles::new()
            .axis(ROWS_ROLE, rows)
            .input(STATES_ROLE, states)
            .input(ACTIONS_ROLE, actions)
            .input(TARGETS_ROLE, targets)
            .input(RATE_ROLE, optimizer.rate())
    })
}
