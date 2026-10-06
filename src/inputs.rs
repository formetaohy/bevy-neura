use crate::role::Role;

#[derive(Clone, Debug, Default)]
pub struct Inputs {
    writes: Vec<(Role, Vec<f32>)>,
}

impl Inputs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(mut self, role: Role, data: impl Into<Vec<f32>>) -> Self {
        assert!(
            !self.writes.iter().any(|(name, _)| *name == role),
            "two writes of one input answer to the role {role}",
        );
        self.writes.push((role, data.into()));
        self
    }

    pub(crate) fn writes(&self) -> &[(Role, Vec<f32>)] {
        &self.writes
    }
}
