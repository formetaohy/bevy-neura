use crate::extents::Extents;
use crate::geometry::ModelGeometry;
use crate::inputs::Inputs;
use crate::role::Role;
use crate::roles::Roles;
use bevy::prelude::Component;
use neura::{Checkpoint, Program, Readout, Run, Runtime, Value, Weights};

#[derive(Component)]
pub struct Model {
    roles: Roles,
    program: Program,
}

impl Model {
    pub(crate) fn of(roles: Roles, program: Program) -> Self {
        Self { roles, program }
    }

    pub fn bind(&self, runtime: &Runtime, extents: &Extents) {
        runtime.bind(&self.program, &self.roles.lengths(extents));
    }

    pub fn run(&self, runtime: &Runtime, inputs: &Inputs) -> Run {
        for (role, data) in inputs.writes() {
            runtime.write(&self.program, self.roles.input_value(role), data);
        }
        runtime.run(&self.program)
    }

    pub fn read(&self, runtime: &Runtime, role: Role) -> Vec<f32> {
        runtime.read(&self.program, self.roles.output_value(role))
    }

    pub fn read_all(&self, runtime: &Runtime) -> Vec<(Role, Vec<f32>)> {
        let roles = self.roles.output_roles();
        let values = roles
            .iter()
            .map(|role| self.roles.output_value(role))
            .collect::<Vec<_>>();
        runtime
            .read_many(&self.program, &values)
            .into_iter()
            .zip(roles)
            .map(|(data, role)| (role, data))
            .collect()
    }

    pub fn pull(&self, runtime: &Runtime, roles: &[Role]) -> Readout {
        let values = roles
            .iter()
            .map(|role| self.roles.output_value(role))
            .collect::<Vec<_>>();
        runtime.pull(&self.program, &values)
    }

    pub fn checkpoint(&self, runtime: &Runtime) -> Checkpoint {
        runtime.checkpoint(self.program.weights())
    }

    pub fn restore(&self, runtime: &Runtime, checkpoint: &Checkpoint) {
        runtime.restore(self.program.weights(), checkpoint);
    }

    pub fn geometry(&self) -> ModelGeometry {
        ModelGeometry {
            tasks: self.program.task_count(),
            waves: self.program.wave_count(),
            workgroups: self.program.workgroups(),
            tensors: self.program.tensor_bytes(),
            weights: self.program.weights().bytes(),
            arena: self.program.arena_bytes(),
            device: self.program.device_bytes(),
            updates: self.program.updates_weights(),
            dynamic: self.program.dynamic(),
            bounds: self.roles.bounds(),
        }
    }

    pub fn value(&self, role: Role) -> Value<'static> {
        self.roles.value(role)
    }

    pub fn weights(&self) -> &Weights {
        self.program.weights()
    }

    pub fn program(&self) -> &Program {
        &self.program
    }
}
