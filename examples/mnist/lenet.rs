use crate::digit;
use neura::{Conv2d, Element, Free, Graph, Init, Linear, Pool, Shape, Value, Window};

pub const CLASSES: u32 = 10;
const CHANNELS: [u32; 2] = [6, 16];
const TAPS: [u32; 2] = [5, 5];
const POOL: [u32; 2] = [2, 2];
const DENSE: [u32; 2] = [120, 84];
const FLAT: u32 = CHANNELS[1] * 4 * 4;

pub struct Lenet {
    conv: [Conv2d; 2],
    dense: [Linear; 3],
}

impl Lenet {
    pub fn build(graph: &Graph) -> Self {
        let spread = Init::Kaiming {
            gain: std::f32::consts::SQRT_2,
        };
        Self {
            conv: [
                Conv2d::new(
                    graph,
                    "conv.first",
                    [1, CHANNELS[0]],
                    1,
                    Window::sliding(TAPS),
                    spread,
                    Element::Single,
                ),
                Conv2d::new(
                    graph,
                    "conv.second",
                    [CHANNELS[0], CHANNELS[1]],
                    1,
                    Window::sliding(TAPS),
                    spread,
                    Element::Single,
                ),
            ],
            dense: [
                Linear::new(
                    graph,
                    "dense.first",
                    FLAT,
                    DENSE[0],
                    spread,
                    Element::Single,
                ),
                Linear::new(
                    graph,
                    "dense.second",
                    DENSE[0],
                    DENSE[1],
                    spread,
                    Element::Single,
                ),
                Linear::new(
                    graph,
                    "dense.last",
                    DENSE[1],
                    CLASSES,
                    spread,
                    Element::Single,
                ),
            ],
        }
    }

    pub fn forward(&self, graph: &Graph, digits: Value, batch: Free) -> Value {
        let side = digit::SIDE as u32;
        let shape = graph.shape(digits);
        assert_eq!(
            shape.dims()[2..],
            [side, side],
            "the network reads digits of {side} by {side} pixels",
        );
        assert_eq!(
            shape.free(0),
            Some(batch.slot()),
            "a batch of {} digits walks the free extent the binding names",
            shape.dims()[0],
        );
        let window = Window::new(POOL, POOL, [0, 0], [1, 1]);
        let pooled = |hidden| graph.pool2d(hidden, window, Pool::Max);
        let hidden = pooled(graph.relu(self.conv[0].forward(graph, digits)));
        let hidden = pooled(graph.relu(self.conv[1].forward(graph, hidden)));
        let flat = graph.reshape(
            hidden,
            Shape::of([graph.shape(hidden).dims()[0], 1, 1, FLAT]).freed(&[(0, batch)]),
        );
        let hidden = graph.relu(self.dense[0].forward(graph, flat));
        let hidden = graph.relu(self.dense[1].forward(graph, hidden));
        self.dense[2].forward(graph, hidden)
    }

    pub fn parameters(&self) -> Vec<Value> {
        let mut parameters = Vec::new();
        for layer in &self.conv {
            parameters.extend(layer.parameters());
        }
        for layer in &self.dense {
            parameters.extend(layer.parameters());
        }
        parameters
    }
}
