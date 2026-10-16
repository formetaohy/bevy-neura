use crate::extents::Extents;
use crate::inputs::Inputs;
use crate::roles::Role;
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

    pub fn bound(&self, role: Role) -> u32 {
        self.roles.bound(role)
    }

    pub fn value(&self, role: Role) -> Value {
        self.roles.value(role)
    }

    pub fn weights(&self) -> &Weights {
        self.program.weights()
    }

    pub fn program(&self) -> &Program {
        &self.program
    }
}
