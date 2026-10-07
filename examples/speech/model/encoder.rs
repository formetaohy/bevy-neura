use crate::model::attention::{CrossProjection, SelfAttention};
use crate::model::dims::Dims;
use crate::model::parameters::{Layout, Parameters};
use neura::{Conv2d, Element, Graph, Init, LayerNorm, Linear, Shape, Value, Window};

pub struct Encoder {
    conv1: Conv2d<'static>,
    conv2: Conv2d<'static>,
    positions: Value<'static>,
    blocks: Vec<Block>,
    norm: LayerNorm<'static>,
    cross: Vec<CrossProjection>,
}

struct Block {
    attention: SelfAttention,
    attention_norm: LayerNorm<'static>,
    up: Linear<'static>,
    down: Linear<'static>,
    mlp_norm: LayerNorm<'static>,
}

impl Block {
    fn declare(
        graph: &Graph<'static>,
        parameters: &mut Parameters,
        prefix: &str,
        dims: &Dims,
    ) -> Self {
        let attention_norm = LayerNorm::new(
            graph,
            &format!("{prefix}.self_attn_layer_norm"),
            dims.state,
            Init::Zero,
            dims.floor,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.self_attn_layer_norm.weight"),
            Shape::vector(dims.state),
            Layout::Direct,
            attention_norm.scale(),
        );
        parameters.adopt(
            &format!("{prefix}.self_attn_layer_norm.bias"),
            Shape::vector(dims.state),
            Layout::Direct,
            attention_norm.shift(),
        );
        let mlp_norm = LayerNorm::new(
            graph,
            &format!("{prefix}.final_layer_norm"),
            dims.state,
            Init::Zero,
            dims.floor,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.final_layer_norm.weight"),
            Shape::vector(dims.state),
            Layout::Direct,
            mlp_norm.scale(),
        );
        parameters.adopt(
            &format!("{prefix}.final_layer_norm.bias"),
            Shape::vector(dims.state),
            Layout::Direct,
            mlp_norm.shift(),
        );
        let up = Linear::new(
            graph,
            &format!("{prefix}.fc1"),
            dims.state,
            dims.encoder_ffn,
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.fc1.weight"),
            Shape::matrix(dims.state, dims.encoder_ffn),
            Layout::Transposed,
            up.weight(),
        );
        parameters.adopt(
            &format!("{prefix}.fc1.bias"),
            Shape::vector(dims.encoder_ffn),
            Layout::Direct,
            up.bias(),
        );
        let down = Linear::new(
            graph,
            &format!("{prefix}.fc2"),
            dims.encoder_ffn,
            dims.state,
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.fc2.weight"),
            Shape::matrix(dims.encoder_ffn, dims.state),
            Layout::Transposed,
            down.weight(),
        );
        parameters.adopt(
            &format!("{prefix}.fc2.bias"),
            Shape::vector(dims.state),
            Layout::Direct,
            down.bias(),
        );
        Self {
            attention: SelfAttention::declare(
                graph,
                parameters,
                &format!("{prefix}.self_attn"),
                dims,
            ),
            attention_norm,
            up,
            down,
            mlp_norm,
        }
    }

    fn forward(&self, graph: &Graph<'static>, hidden: Value<'static>) -> Value<'static> {
        let attended = graph.add(
            hidden,
            self.attention
                .forward(graph, self.attention_norm.forward(graph, hidden), false),
        );
        let widened = graph.gelu(
            self.up
                .forward(graph, self.mlp_norm.forward(graph, attended)),
        );
        graph.add(attended, self.down.forward(graph, widened))
    }
}

impl Encoder {
    pub fn build(graph: &Graph<'static>, parameters: &mut Parameters, dims: &Dims) -> Self {
        let prefix = "model.encoder";
        let conv1 = Conv2d::new(
            graph,
            &format!("{prefix}.conv1"),
            [dims.mel_bins, dims.state],
            1,
            Window::new([1, 3], [1, 1], [0, 1]),
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.conv1.weight"),
            Shape::of([dims.state, dims.mel_bins, 1, 3]),
            Layout::Direct,
            conv1.filter(),
        );
        parameters.adopt(
            &format!("{prefix}.conv1.bias"),
            Shape::of([1, dims.state, 1, 1]),
            Layout::Direct,
            conv1.bias(),
        );
        let conv2 = Conv2d::new(
            graph,
            &format!("{prefix}.conv2"),
            [dims.state, dims.state],
            1,
            Window::new([1, 3], [1, 2], [0, 1]),
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.conv2.weight"),
            Shape::of([dims.state, dims.state, 1, 3]),
            Layout::Direct,
            conv2.filter(),
        );
        parameters.adopt(
            &format!("{prefix}.conv2.bias"),
            Shape::of([1, dims.state, 1, 1]),
            Layout::Direct,
            conv2.bias(),
        );
        let positions = parameters.declare(
            graph,
            &format!("{prefix}.embed_positions.weight"),
            Shape::of([1, 1, dims.positions, dims.state]),
            Layout::Direct,
        );
        let norm = LayerNorm::new(
            graph,
            &format!("{prefix}.layer_norm"),
            dims.state,
            Init::Zero,
            dims.floor,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.layer_norm.weight"),
            Shape::vector(dims.state),
            Layout::Direct,
            norm.scale(),
        );
        parameters.adopt(
            &format!("{prefix}.layer_norm.bias"),
            Shape::vector(dims.state),
            Layout::Direct,
            norm.shift(),
        );
        let blocks = (0..dims.encoder_layers)
            .map(|index| {
                Block::declare(graph, parameters, &format!("{prefix}.layers.{index}"), dims)
            })
            .collect();
        let cross = (0..dims.decoder_layers)
            .map(|index| {
                CrossProjection::declare(
                    graph,
                    parameters,
                    &format!("model.decoder.layers.{index}.encoder_attn"),
                    dims,
                )
            })
            .collect();
        Self {
            conv1,
            conv2,
            positions,
            blocks,
            norm,
            cross,
        }
    }

    pub fn forward(&self, graph: &Graph<'static>, mel: Value<'static>) -> Value<'static> {
        let first = graph.gelu(self.conv1.forward(graph, mel));
        let second = graph.gelu(self.conv2.forward(graph, first));
        let mut hidden = graph.add(graph.permute(second, [0, 2, 3, 1]), self.positions);
        for block in &self.blocks {
            hidden = block.forward(graph, hidden);
        }
        self.norm.forward(graph, hidden)
    }

    pub fn project(
        &self,
        graph: &Graph<'static>,
        states: Value<'static>,
    ) -> Vec<(Value<'static>, Value<'static>)> {
        self.cross
            .iter()
            .map(|projection| projection.forward(graph, states))
            .collect()
    }
}
