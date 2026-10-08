use crate::model::attention::{CrossAttention, SelfAttention};
use crate::model::dims::Dims;
use crate::model::parameters::{Layout, Parameters};
use neura::{Element, Embedding, Graph, Init, LayerNorm, Linear, Shape, Value};

pub struct Decoder {
    table: Embedding<'static>,
    positions: Value<'static>,
    blocks: Vec<Block>,
    norm: LayerNorm<'static>,
    head: Value<'static>,
    prefix: u32,
    state: u32,
}

struct Block {
    attention: SelfAttention,
    attention_norm: LayerNorm<'static>,
    cross: CrossAttention,
    cross_norm: LayerNorm<'static>,
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
        let mut norm = |what: &str| {
            let norm = LayerNorm::new(
                graph,
                &format!("{prefix}.{what}"),
                dims.state,
                Init::Zero,
                dims.floor,
                Element::Single,
            );
            parameters.adopt(
                &format!("{prefix}.{what}.weight"),
                Shape::vector(dims.state),
                Layout::Direct,
                norm.scale(),
            );
            parameters.adopt(
                &format!("{prefix}.{what}.bias"),
                Shape::vector(dims.state),
                Layout::Direct,
                norm.shift(),
            );
            norm
        };
        let attention_norm = norm("self_attn_layer_norm");
        let cross_norm = norm("encoder_attn_layer_norm");
        let mlp_norm = norm("final_layer_norm");
        let up = Linear::new(
            graph,
            &format!("{prefix}.fc1"),
            dims.state,
            dims.decoder_ffn,
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.fc1.weight"),
            Shape::matrix(dims.state, dims.decoder_ffn),
            Layout::Transposed,
            up.weight(),
        );
        parameters.adopt(
            &format!("{prefix}.fc1.bias"),
            Shape::vector(dims.decoder_ffn),
            Layout::Direct,
            up.bias(),
        );
        let down = Linear::new(
            graph,
            &format!("{prefix}.fc2"),
            dims.decoder_ffn,
            dims.state,
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.fc2.weight"),
            Shape::matrix(dims.decoder_ffn, dims.state),
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
            cross: CrossAttention::declare(
                graph,
                parameters,
                &format!("{prefix}.encoder_attn"),
                dims,
            ),
            cross_norm,
            up,
            down,
            mlp_norm,
        }
    }

    fn forward(
        &self,
        graph: &Graph<'static>,
        hidden: Value<'static>,
        keys: Value<'static>,
        values: Value<'static>,
    ) -> Value<'static> {
        let attended = graph.add(
            hidden,
            self.attention
                .forward(graph, self.attention_norm.forward(graph, hidden), true),
        );
        let crossed = graph.add(
            attended,
            self.cross.forward(
                graph,
                self.cross_norm.forward(graph, attended),
                keys,
                values,
            ),
        );
        let widened = graph.gelu(
            self.up
                .forward(graph, self.mlp_norm.forward(graph, crossed)),
        );
        graph.add(crossed, self.down.forward(graph, widened))
    }
}

impl Decoder {
    pub fn build(graph: &Graph<'static>, parameters: &mut Parameters, dims: &Dims) -> Self {
        let prefix = "model.decoder";
        let table = Embedding::new(
            graph,
            &format!("{prefix}.embed_tokens"),
            dims.vocab,
            dims.state,
            Init::Zero,
            Element::Single,
        );
        parameters.adopt(
            &format!("{prefix}.embed_tokens.weight"),
            Shape::matrix(dims.vocab, dims.state),
            Layout::Direct,
            table.table(),
        );
        let positions = parameters.declare(
            graph,
            &format!("{prefix}.embed_positions.weight"),
            Shape::matrix(dims.target_positions, dims.state),
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
        let head = parameters.declare_from(
            graph,
            "proj_out.weight",
            &format!("{prefix}.embed_tokens.weight"),
            Shape::of([1, 1, dims.state, dims.vocab]),
            Layout::Transposed,
        );
        let blocks = (0..dims.decoder_layers)
            .map(|index| {
                Block::declare(graph, parameters, &format!("{prefix}.layers.{index}"), dims)
            })
            .collect();
        Self {
            table,
            positions,
            blocks,
            norm,
            head,
            prefix: dims.prefix(),
            state: dims.state,
        }
    }

    pub fn forward(
        &self,
        graph: &Graph<'static>,
        tokens: Value<'static>,
        placements: Value<'static>,
        cursor: Value<'static>,
        cross: &[(Value<'static>, Value<'static>)],
    ) -> (Value<'static>, Value<'static>) {
        assert_eq!(
            cross.len(),
            self.blocks.len(),
            "a decoder of {} blocks reads {} cross attentions",
            self.blocks.len(),
            cross.len(),
        );
        let embedded = graph.add(
            self.table.forward(graph, tokens),
            graph.gather(self.positions, placements),
        );
        let mut hidden = embedded;
        for (block, (keys, values)) in self.blocks.iter().zip(cross) {
            hidden = block.forward(graph, hidden, *keys, *values);
        }
        let hidden = self.norm.forward(graph, hidden);
        let rows = graph.reshape(hidden, Shape::matrix(self.prefix, self.state));
        let row = graph.gather(rows, cursor);
        let logits = graph.matmul(row, self.head);
        let token = graph.argmax(logits);
        (logits, token)
    }
}
