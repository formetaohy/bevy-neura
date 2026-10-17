use crate::board::CELLS;
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{Element, Graph, Init, Linear, Shape, Value};

pub const BOUND: u32 = 64;
pub const ROWS_ROLE: Role = "rows";
pub const STATES_ROLE: Role = "states";
pub const Q_ROLE: Role = "q";

const HIDDEN: u32 = 96;
const OUTPUT: f32 = 0.05;

pub struct QNet {
    dense: [Linear; 3],
}

impl QNet {
    pub fn build(graph: &Graph) -> Self {
        let spread = Init::Kaiming {
            gain: std::f32::consts::SQRT_2,
        };
        Self {
            dense: [
                Linear::new(
                    graph,
                    "trunk.first",
                    CELLS as u32,
                    HIDDEN,
                    spread,
                    Element::Single,
                ),
                Linear::new(
                    graph,
                    "trunk.second",
                    HIDDEN,
                    HIDDEN,
                    spread,
                    Element::Single,
                ),
                Linear::new(
                    graph,
                    "scores",
                    HIDDEN,
                    CELLS as u32,
                    Init::Uniform {
                        low: -OUTPUT,
                        high: OUTPUT,
                    },
                    Element::Single,
                ),
            ],
        }
    }

    pub fn forward(&self, graph: &Graph, states: Value) -> Value {
        let hidden = graph.relu(self.dense[0].forward(graph, states));
        let hidden = graph.relu(self.dense[1].forward(graph, hidden));
        self.dense[2].forward(graph, hidden)
    }

    pub fn parameters(&self) -> Vec<Value> {
        self.dense
            .iter()
            .flat_map(Linear::parameters)
            .collect::<Vec<_>>()
    }
}

pub fn plan(bound: u32) -> ModelPlan {
    ModelPlan::build(move |graph| {
        let rows = graph.free(bound);
        let states = graph.input(
            Shape::matrix(bound, CELLS as u32).freed(&[(2, rows)]),
            Element::Single,
        );
        let net = QNet::build(graph);
        let q = net.forward(graph, states);
        Roles::new()
            .axis(ROWS_ROLE, rows)
            .input(STATES_ROLE, states)
            .output(Q_ROLE, q)
    })
}

pub struct Player {
    model: Model,
}

impl Player {
    pub fn share(runtime: &NeuraRuntime, source: &Model) -> Self {
        Self {
            model: plan(BOUND).share(runtime, source),
        }
    }

    pub fn q(&self, runtime: &NeuraRuntime, states: &[f32], rows: u32) -> Vec<f32> {
        assert!(
            rows > 0 && rows <= BOUND,
            "a reading of {rows} boards outruns the {BOUND} a binding names",
        );
        assert_eq!(
            states.len(),
            rows as usize * CELLS,
            "a reading of {rows} boards walks {} numbers where each holds {CELLS}",
            states.len(),
        );
        self.model
            .bind(runtime, &Extents::new().axis(ROWS_ROLE, rows));
        self.model
            .run(runtime, &Inputs::new().write(STATES_ROLE, states));
        let q = self.model.read(runtime, Q_ROLE);
        assert_eq!(
            q.len(),
            rows as usize * CELLS,
            "a reading of {rows} boards answers {} scores where each holds {CELLS}",
            q.len(),
        );
        assert!(
            q.iter().all(|score| score.is_finite()),
            "a reading of {rows} boards answers a score that is not a number",
        );
        q
    }
}
