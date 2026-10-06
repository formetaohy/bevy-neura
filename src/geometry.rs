use crate::role::Role;

#[derive(Debug)]
pub struct ModelGeometry {
    pub tasks: u32,
    pub waves: u32,
    pub workgroups: u32,
    pub tensors: u64,
    pub weights: u64,
    pub arena: u64,
    pub device: u64,
    pub updates: bool,
    pub dynamic: bool,
    pub bounds: Vec<(Role, u32)>,
}
