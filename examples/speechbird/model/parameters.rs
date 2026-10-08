use neura::{Element, Graph, Init, Program, Runtime, Shape, Value};
use safetensors::SafeTensors;
use safetensors::tensor::TensorView;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    Direct,
    Transposed,
    Projected,
    Merged,
}

pub struct Parameter {
    name: String,
    source: String,
    value: Value<'static>,
    shape: [u32; 4],
    layout: Layout,
}

impl Parameter {
    fn elements(&self) -> u32 {
        self.shape.iter().product::<u32>()
    }
}

#[derive(Default)]
pub struct Parameters {
    entries: Vec<Parameter>,
}

impl Parameters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(
        &mut self,
        graph: &Graph<'static>,
        name: &str,
        shape: Shape,
        layout: Layout,
    ) -> Value<'static> {
        self.declare_from(graph, name, name, shape, layout)
    }

    pub fn declare_from(
        &mut self,
        graph: &Graph<'static>,
        name: &str,
        source: &str,
        shape: Shape,
        layout: Layout,
    ) -> Value<'static> {
        assert!(
            !self.entries.iter().any(|entry| entry.name == name),
            "two parameters of a model answer to the name {name}",
        );
        let value = graph.named_parameter(name, shape, Init::Zero, Element::Single);
        self.entries.push(Parameter {
            name: name.to_string(),
            source: source.to_string(),
            value,
            shape: shape.dims(),
            layout,
        });
        value
    }

    pub fn adopt(&mut self, name: &str, shape: Shape, layout: Layout, value: Value<'static>) {
        assert!(
            !self.entries.iter().any(|entry| entry.name == name),
            "two parameters of a model answer to the name {name}",
        );
        self.entries.push(Parameter {
            name: name.to_string(),
            source: name.to_string(),
            value,
            shape: shape.dims(),
            layout,
        });
    }

    pub fn pour(&self, runtime: &Runtime, program: &Program, bytes: &[u8]) -> usize {
        let checkpoint = SafeTensors::deserialize(bytes).expect("a checkpoint holds safetensors");
        let mut written = 0;
        for parameter in &self.entries {
            let view = checkpoint.tensor(&parameter.source).unwrap_or_else(|_| {
                panic!(
                    "this checkpoint holds no tensor named {}, and the graph declares it",
                    parameter.source,
                )
            });
            let source = floats(&view);
            let shape = view.shape();
            let elements = parameter.elements() as usize;
            assert_eq!(
                source.len(),
                elements,
                "this checkpoint holds {} numbers of {} where the graph reads {elements}",
                source.len(),
                parameter.source,
            );
            let laid = match parameter.layout {
                Layout::Direct => source,
                Layout::Transposed => {
                    assert_eq!(
                        shape.len(),
                        2,
                        "a transposed parameter reads two axes, and {} holds {}",
                        parameter.source,
                        shape.len(),
                    );
                    assert_eq!(
                        shape,
                        [parameter.shape[3] as usize, parameter.shape[2] as usize],
                        "a transposed parameter of {} reads {:?} where its source holds {shape:?}",
                        parameter.name,
                        [parameter.shape[3], parameter.shape[2]],
                    );
                    transposed(&source, shape[0])
                }
                Layout::Projected => {
                    let (heads, width) = (parameter.shape[0], parameter.shape[3]);
                    assert_eq!(
                        shape,
                        [(heads * width) as usize, parameter.shape[2] as usize],
                        "a projected parameter of {heads} heads over {width} numbers reads {:?} where its source holds {shape:?}",
                        [(heads * width), parameter.shape[2]],
                    );
                    projected(&source, parameter.shape[2], heads, width)
                }
                Layout::Merged => {
                    let (heads, width, state) =
                        (parameter.shape[0], parameter.shape[2], parameter.shape[3]);
                    assert_eq!(
                        shape,
                        [state as usize, state as usize],
                        "a merged parameter of {heads} heads over {width} numbers reads {:?} where its source holds {shape:?}",
                        [state, state],
                    );
                    merged(&source, state, heads, width)
                }
            };
            runtime.write(program, parameter.value, &laid);
            written += 1;
        }
        written
    }
}

fn floats(view: &TensorView<'_>) -> Vec<f32> {
    assert_eq!(
        view.dtype(),
        safetensors::Dtype::F32,
        "this checkpoint holds another number type than the numbers a graph reads",
    );
    view.data()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| f32::from_le_bytes(*word))
        .collect()
}

fn transposed(source: &[f32], rows: usize) -> Vec<f32> {
    let columns = source.len() / rows;
    let mut out = vec![0.0f32; source.len()];
    for row in 0..rows {
        for column in 0..columns {
            out[column * rows + row] = source[row * columns + column];
        }
    }
    out
}

fn projected(source: &[f32], columns: u32, heads: u32, width: u32) -> Vec<f32> {
    let mut out = vec![0.0f32; source.len()];
    for head in 0..heads {
        for lane in 0..width {
            for column in 0..columns {
                out[((head * columns + column) * width + lane) as usize] =
                    source[((head * width + lane) * columns + column) as usize];
            }
        }
    }
    out
}

fn merged(source: &[f32], state: u32, heads: u32, width: u32) -> Vec<f32> {
    let mut out = vec![0.0f32; source.len()];
    for output in 0..state {
        for head in 0..heads {
            for lane in 0..width {
                out[((head * width + lane) * state + output) as usize] =
                    source[(output * state + head * width + lane) as usize];
            }
        }
    }
    out
}
