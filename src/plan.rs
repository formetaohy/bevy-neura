use crate::role::Role;
use crate::sample::{Extents, Sample};
use neura::{Graph, Shape, Value};
use neura_abi::{MAX_RANK, NO_VALUE};
use neura_graph::Free;

#[derive(Clone, Default)]
pub struct Roles {
    axes: Vec<(Role, Free)>,
    inputs: Vec<(Role, Value<'static>)>,
    outputs: Vec<(Role, Value<'static>)>,
    loss: Option<Value<'static>>,
}

impl Roles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn axis(mut self, role: Role, free: Free) -> Self {
        assert!(
            !self.axes.iter().any(|(name, _)| *name == role),
            "two host extents answer to the role {role}",
        );
        self.axes.push((role, free));
        self
    }

    pub fn input(mut self, role: Role, value: Value<'static>) -> Self {
        assert!(
            !self.inputs.iter().any(|(name, _)| *name == role),
            "two inputs answer to the role {role}",
        );
        self.inputs.push((role, value));
        self
    }

    pub fn output(mut self, role: Role, value: Value<'static>) -> Self {
        assert!(
            !self.outputs.iter().any(|(name, _)| *name == role),
            "two outputs answer to the role {role}",
        );
        self.outputs.push((role, value));
        self
    }

    pub fn loss(mut self, value: Value<'static>) -> Self {
        assert!(self.loss.is_none(), "a model trains against one loss");
        self.loss = Some(value);
        self
    }

    pub(crate) fn assert_binds(&self, graph: &Graph<'static>) {
        for (_, value) in self.inputs.iter().chain(&self.outputs) {
            graph.shape(*value);
        }
        if let Some(loss) = self.loss {
            graph.shape(loss);
        }
        let snapshot = graph.snapshot();
        let hosts = snapshot
            .authored()
            .iter()
            .enumerate()
            .filter(|(_, count)| **count == NO_VALUE)
            .map(|(slot, _)| slot as u32)
            .collect::<Vec<u32>>();
        let mut named = self
            .axes
            .iter()
            .map(|(_, free)| free.slot())
            .collect::<Vec<u32>>();
        named.sort_unstable();
        assert_eq!(
            named, hosts,
            "a model names the axes {named:?} its roles bind, where the graph carries the host extents {hosts:?}",
        );
    }

    pub(crate) fn lengths(&self, extents: &Extents) -> Vec<u32> {
        assert_eq!(
            extents.axes().len(),
            self.axes.len(),
            "a model of {} host extents binds {} live lengths",
            self.axes.len(),
            extents.axes().len(),
        );
        let mut axes = self
            .axes
            .iter()
            .map(|(role, free)| (free.slot(), *role, free.bound()))
            .collect::<Vec<(u32, Role, u32)>>();
        let mut ordered = Vec::with_capacity(axes.len());
        for (role, length) in extents.axes() {
            let index = axes
                .iter()
                .position(|(_, name, _)| name == role)
                .unwrap_or_else(|| {
                    panic!("the live length of {role} binds no host extent of this model")
                });
            let (slot, _, bound) = axes.remove(index);
            assert!(
                *length <= bound,
                "a live {role} of {length} outruns the bound of {bound}",
            );
            ordered.push((slot, *length));
        }
        ordered.sort_unstable_by_key(|(slot, _)| *slot);
        ordered.into_iter().map(|(_, length)| length).collect()
    }

    pub(crate) fn assert_writes(&self, sample: &Sample, lengths: &[u32]) {
        let live = self.live(lengths);
        for (role, data) in sample.writes() {
            let (_, value) = self
                .inputs
                .iter()
                .find(|(name, _)| name == role)
                .unwrap_or_else(|| {
                    panic!("a write of the role {role} reaches no input of this model")
                });
            if let Some(elements) = live_elements(value.shape(), &live) {
                assert_eq!(
                    data.len(),
                    elements as usize,
                    "a write of {} numbers reaches the input {role} of {elements} numbers",
                    data.len(),
                );
            }
        }
    }

    pub(crate) fn input_value(&self, role: Role) -> Value<'static> {
        self.inputs
            .iter()
            .find(|(name, _)| *name == role)
            .map(|(_, value)| *value)
            .unwrap_or_else(|| panic!("the model holds no input under the role {role}"))
    }

    pub(crate) fn output_value(&self, role: Role) -> Value<'static> {
        self.outputs
            .iter()
            .find(|(name, _)| *name == role)
            .map(|(_, value)| *value)
            .unwrap_or_else(|| panic!("the model holds no output under the role {role}"))
    }

    pub(crate) fn output_roles(&self) -> Vec<Role> {
        self.outputs.iter().map(|(role, _)| *role).collect()
    }

    pub(crate) fn loss_value(&self) -> Option<Value<'static>> {
        self.loss
    }

    pub(crate) fn retains(&self) -> Vec<Value<'static>> {
        self.outputs
            .iter()
            .map(|(_, value)| *value)
            .chain(self.loss)
            .collect()
    }

    pub(crate) fn bounds(&self) -> Vec<(Role, u32)> {
        self.axes
            .iter()
            .map(|(role, free)| (*role, free.bound()))
            .collect()
    }

    fn live(&self, lengths: &[u32]) -> Vec<(u32, u32)> {
        let mut slots = self
            .axes
            .iter()
            .map(|(_, free)| free.slot())
            .collect::<Vec<u32>>();
        slots.sort_unstable();
        slots.into_iter().zip(lengths.iter().copied()).collect()
    }
}

fn live_elements(shape: Shape, live: &[(u32, u32)]) -> Option<u32> {
    let mut elements = 1u64;
    for axis in 0..MAX_RANK {
        let dim = match shape.free(axis) {
            Some(slot) => live
                .iter()
                .find(|(name, _)| *name == slot)
                .map(|(_, length)| *length)?,
            None => shape.dims()[axis as usize],
        };
        elements *= u64::from(dim);
    }
    assert!(
        elements <= u64::from(u32::MAX),
        "a tensor of {elements} numbers outruns the numbers a device addresses",
    );
    Some(elements as u32)
}

pub struct ModelPlan {
    pub(crate) graph: Graph<'static>,
    pub(crate) roles: Roles,
    pub(crate) updates: bool,
}

impl ModelPlan {
    pub fn build(learn: impl FnOnce(&Graph<'static>) -> Roles) -> Self {
        let graph: Graph<'static> = Graph::new();
        let roles = learn(&graph);
        roles.assert_binds(&graph);
        let updates = graph.updates_weights();
        Self {
            graph,
            roles,
            updates,
        }
    }

    pub(crate) fn record(&self) -> ModelRecord {
        ModelRecord {
            roles: self.roles.clone(),
            updates: self.updates,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ModelRecord {
    pub(crate) roles: Roles,
    pub(crate) updates: bool,
}
