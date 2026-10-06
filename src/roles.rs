use crate::extents::Extents;
use crate::role::Role;
use neura::{Free, Graph, Value};
use neura_abi::NO_VALUE;

#[derive(Default)]
pub struct Roles {
    axes: Vec<(Role, Free)>,
    inputs: Vec<(Role, Value<'static>)>,
    outputs: Vec<(Role, Value<'static>)>,
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
        self.assert_names_one(role, value);
        self.inputs.push((role, value));
        self
    }

    pub fn output(mut self, role: Role, value: Value<'static>) -> Self {
        assert!(
            !self.outputs.iter().any(|(name, _)| *name == role),
            "two outputs answer to the role {role}",
        );
        self.assert_names_one(role, value);
        self.outputs.push((role, value));
        self
    }

    fn assert_names_one(&self, role: Role, value: Value<'static>) {
        if let Some((_, held)) = self
            .inputs
            .iter()
            .chain(&self.outputs)
            .find(|(name, _)| *name == role)
        {
            assert_eq!(
                *held,
                value,
                "the role {role} names one tensor of a model, and it reaches tensor {} where the model holds {}",
                value.id(),
                held.id(),
            );
        }
    }

    pub(crate) fn assert_binds(&self, graph: &Graph<'static>) {
        for (_, value) in self.inputs.iter().chain(&self.outputs) {
            graph.shape(*value);
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
        let mut axes = self.axes.clone();
        let mut ordered = Vec::with_capacity(axes.len());
        for (role, length) in extents.axes() {
            let index = axes
                .iter()
                .position(|(name, _)| name == role)
                .unwrap_or_else(|| {
                    panic!("the live length of {role} binds no host extent of this model")
                });
            let free = axes.remove(index).1;
            ordered.push((free.slot(), *length));
        }
        ordered.sort_unstable_by_key(|(slot, _)| *slot);
        ordered.into_iter().map(|(_, length)| length).collect()
    }

    pub(crate) fn value(&self, role: Role) -> Value<'static> {
        self.inputs
            .iter()
            .chain(&self.outputs)
            .find(|(name, _)| *name == role)
            .map(|(_, value)| *value)
            .unwrap_or_else(|| panic!("the model holds no value under the role {role}"))
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

    pub(crate) fn retains(&self) -> Vec<Value<'static>> {
        self.outputs.iter().map(|(_, value)| *value).collect()
    }

    pub(crate) fn bounds(&self) -> Vec<(Role, u32)> {
        self.axes
            .iter()
            .map(|(role, free)| (*role, free.bound()))
            .collect()
    }
}
