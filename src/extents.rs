use crate::role::Role;

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
