use bevy::prelude::Resource;
use neura::MemoryRequest;

#[derive(Resource, Default)]
pub struct NeuraMemory(pub MemoryRequest);
