use crate::role::Role;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Loss {
    #[default]
    None,
    Final,
    Every,
}

#[derive(Clone, Debug, Default)]
pub struct Sample {
    writes: Vec<(Role, Vec<f32>)>,
}

impl Sample {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(mut self, role: Role, data: impl Into<Vec<f32>>) -> Self {
        assert!(
            !self.writes.iter().any(|(name, _)| *name == role),
            "two writes of one sample answer to the role {role}",
        );
        self.writes.push((role, data.into()));
        self
    }

    pub(crate) fn writes(&self) -> &[(Role, Vec<f32>)] {
        &self.writes
    }
}

#[derive(Clone, Debug, Default)]
pub struct Extents {
    axes: Vec<(Role, u32)>,
}

impl Extents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn axis(mut self, role: Role, length: u32) -> Self {
        assert!(
            !self.axes.iter().any(|(name, _)| *name == role),
            "two lengths of one binding answer to the role {role}",
        );
        self.axes.push((role, length));
        self
    }

    pub(crate) fn axes(&self) -> &[(Role, u32)] {
        &self.axes
    }
}
