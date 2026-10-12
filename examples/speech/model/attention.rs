use crate::model::dims::Dims;
use crate::model::parameters::{Layout, Parameters};
use neura::{AttentionOptions, Graph, Shape, Value};

pub struct Projection {
    weight: Value,
    bias: Option<Value>,
}

impl Projection {
    fn declare(
        graph: &Graph,
        parameters: &mut Parameters,
        prefix: &str,
        dims: &Dims,
        bias: bool,
    ) -> Self {
        let weight = parameters.declare(
            graph,
            &format!("{prefix}.weight"),
            Shape::of([dims.heads, 1, dims.state, dims.width()]),
            Layout::Projected,
        );
        let bias = bias.then(|| {
            parameters.declare(
                graph,
                &format!("{prefix}.bias"),
                Shape::of([dims.heads, 1, 1, dims.width()]),
                Layout::Direct,
            )
        });
        Self { weight, bias }
    }

    pub fn forward(&self, graph: &Graph, input: Value) -> Value {
        let projected = graph.matmul(input, self.weight);
        match self.bias {
            Some(bias) => graph.add(projected, bias),
            None => projected,
        }
    }
}

pub struct Merge {
    weight: Value,
    bias: Value,
}

impl Merge {
    fn declare(graph: &Graph, parameters: &mut Parameters, prefix: &str, dims: &Dims) -> Self {
        Self {
            weight: parameters.declare(
                graph,
                &format!("{prefix}.weight"),
                Shape::of([dims.heads, 1, dims.width(), dims.state]),
                Layout::Merged,
            ),
            bias: parameters.declare(
                graph,
                &format!("{prefix}.bias"),
                Shape::vector(dims.state),
                Layout::Direct,
            ),
        }
    }

    fn forward(&self, graph: &Graph, attended: Value) -> Value {
        let merged = graph.sum_axis(graph.matmul(attended, self.weight), 0);
        graph.add(merged, self.bias)
    }
}

pub struct SelfAttention {
    query: Projection,
    key: Projection,
    value: Projection,
    merge: Merge,
    scale: Value,
}

impl SelfAttention {
    pub fn declare(graph: &Graph, parameters: &mut Parameters, prefix: &str, dims: &Dims) -> Self {
        Self {
            query: Projection::declare(graph, parameters, &format!("{prefix}.q_proj"), dims, true),
            key: Projection::declare(graph, parameters, &format!("{prefix}.k_proj"), dims, false),
            value: Projection::declare(graph, parameters, &format!("{prefix}.v_proj"), dims, true),
            merge: Merge::declare(graph, parameters, &format!("{prefix}.out_proj"), dims),
            scale: graph.knob(dims.scale()),
        }
    }

    pub fn forward(&self, graph: &Graph, hidden: Value, causal: bool) -> Value {
        let attended = graph.attention(
            self.query.forward(graph, hidden),
            self.key.forward(graph, hidden),
            self.value.forward(graph, hidden),
            AttentionOptions {
                scale: Some(self.scale),
                causal,
                origin: None,
                segments: None,
                reach: None,
                query_segments: None,
                bias: None,
            },
        );
        self.merge.forward(graph, attended)
    }
}

pub struct CrossAttention {
    query: Projection,
    merge: Merge,
    scale: Value,
}

impl CrossAttention {
    pub fn declare(graph: &Graph, parameters: &mut Parameters, prefix: &str, dims: &Dims) -> Self {
        Self {
            query: Projection::declare(graph, parameters, &format!("{prefix}.q_proj"), dims, true),
            merge: Merge::declare(graph, parameters, &format!("{prefix}.out_proj"), dims),
            scale: graph.knob(dims.scale()),
        }
    }

    pub fn forward(&self, graph: &Graph, hidden: Value, keys: Value, values: Value) -> Value {
        let attended = graph.attention(
            self.query.forward(graph, hidden),
            keys,
            values,
            AttentionOptions {
                scale: Some(self.scale),
                causal: false,
                origin: None,
                segments: None,
                reach: None,
                query_segments: None,
                bias: None,
            },
        );
        self.merge.forward(graph, attended)
    }
}

pub struct CrossProjection {
    key: Projection,
    value: Projection,
}

impl CrossProjection {
    pub fn declare(graph: &Graph, parameters: &mut Parameters, prefix: &str, dims: &Dims) -> Self {
        Self {
            key: Projection::declare(graph, parameters, &format!("{prefix}.k_proj"), dims, false),
            value: Projection::declare(graph, parameters, &format!("{prefix}.v_proj"), dims, true),
        }
    }

    pub fn forward(&self, graph: &Graph, states: Value) -> (Value, Value) {
        (
            self.key.forward(graph, states),
            self.value.forward(graph, states),
        )
    }
}
