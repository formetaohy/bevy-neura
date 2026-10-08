use crate::env::{Action, OBSERVATION};
use neura::{Element, Graph, Init, Linear, Value};

pub const HIDDEN: u32 = 64;
const TRUNK_BOUND: f32 = 0.35;
const POLICY_BOUND: f32 = 0.01;
const VALUE_BOUND: f32 = 0.5;

pub struct Net<'g> {
    trunk: [Linear<'g>; 2],
    policy: Linear<'g>,
    value: Linear<'g>,
}

impl<'g> Net<'g> {
    pub fn build(graph: &Graph<'g>) -> Self {
        let trunk = Init::Uniform {
            low: -TRUNK_BOUND,
            high: TRUNK_BOUND,
        };
        Self {
            trunk: [
                Linear::new(
                    graph,
                    "trunk.first",
                    OBSERVATION as u32,
                    HIDDEN,
                    trunk,
                    Element::Single,
                ),
                Linear::new(
                    graph,
                    "trunk.second",
                    HIDDEN,
                    HIDDEN,
                    trunk,
                    Element::Single,
                ),
            ],
            policy: Linear::new(
                graph,
                "policy",
                HIDDEN,
                Action::ALL.len() as u32,
                Init::Uniform {
                    low: -POLICY_BOUND,
                    high: POLICY_BOUND,
                },
                Element::Single,
            ),
            value: Linear::new(
                graph,
                "value",
                HIDDEN,
                1,
                Init::Uniform {
                    low: -VALUE_BOUND,
                    high: VALUE_BOUND,
                },
                Element::Single,
            ),
        }
    }

    pub fn forward(&self, graph: &Graph<'g>, observation: Value<'g>) -> (Value<'g>, Value<'g>) {
        let hidden = graph.tanh(self.trunk[0].forward(graph, observation));
        let hidden = graph.tanh(self.trunk[1].forward(graph, hidden));
        (
            self.policy.forward(graph, hidden),
            self.value.forward(graph, hidden),
        )
    }

    pub fn parameters(&self) -> Vec<Value<'g>> {
        let mut parameters = Vec::new();
        for layer in &self.trunk {
            parameters.extend(layer.parameters());
        }
        parameters.extend(self.policy.parameters());
        parameters.extend(self.value.parameters());
        parameters
    }
}
