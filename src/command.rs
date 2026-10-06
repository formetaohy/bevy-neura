use crate::ids::{ModelId, RequestId};
use crate::plan::ModelPlan;
use crate::role::Role;
use crate::sample::{Loss, Sample};
use std::path::PathBuf;

pub(crate) enum Command {
    Attach {
        model: ModelId,
        plan: ModelPlan,
    },
    Share {
        model: ModelId,
        source: ModelId,
        plan: ModelPlan,
    },
    Detach {
        model: ModelId,
    },
    Train {
        request: RequestId,
        model: ModelId,
        lengths: Vec<u32>,
        samples: Vec<Sample>,
        loss: Loss,
    },
    Infer {
        request: RequestId,
        model: ModelId,
        lengths: Vec<u32>,
        sample: Sample,
    },
    Read {
        request: RequestId,
        model: ModelId,
        lengths: Vec<u32>,
        roles: Vec<Role>,
    },
    Tune {
        request: RequestId,
        model: ModelId,
        lengths: Vec<u32>,
    },
    Save {
        request: RequestId,
        model: ModelId,
        path: PathBuf,
    },
    Restore {
        request: RequestId,
        model: ModelId,
        path: PathBuf,
    },
    Shutdown,
}
