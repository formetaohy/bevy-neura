use crate::ids::{ModelId, RequestId};
use crate::role::Role;
use bevy::prelude::Message;
use neura::Profile;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, Message)]
pub enum NeuraReport {
    Attached {
        model: ModelId,
        geometry: ModelGeometry,
    },
    Detached {
        model: ModelId,
    },
    Trained {
        request: RequestId,
        model: ModelId,
        steps: u32,
        losses: Vec<f32>,
        seconds: f64,
    },
    Inferred {
        request: RequestId,
        model: ModelId,
        values: Vec<(Role, Vec<f32>)>,
        seconds: f64,
    },
    Read {
        request: RequestId,
        model: ModelId,
        values: Vec<(Role, Vec<f32>)>,
    },
    Tuned {
        request: RequestId,
        model: ModelId,
        profile: Box<Profile>,
        seconds: f64,
    },
    Saved {
        request: RequestId,
        model: ModelId,
        bytes: u64,
    },
    Restored {
        request: RequestId,
        model: ModelId,
    },
}

impl NeuraReport {
    pub(crate) fn request(&self) -> Option<RequestId> {
        match self {
            Self::Attached { .. } | Self::Detached { .. } => None,
            Self::Trained { request, .. }
            | Self::Inferred { request, .. }
            | Self::Read { request, .. }
            | Self::Tuned { request, .. }
            | Self::Saved { request, .. }
            | Self::Restored { request, .. } => Some(*request),
        }
    }
}
