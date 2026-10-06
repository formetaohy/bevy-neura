mod apply;
mod model;

use crate::command::Command;
use crate::device::NeuraDevice;
use crate::ids::ModelId;
use crate::report::NeuraReport;
use model::Model;
use neura::{Runtime, RuntimeRequest};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

pub(crate) fn spawn(
    request: RuntimeRequest,
    commands: Receiver<Command>,
    reports: Sender<NeuraReport>,
    opening: Sender<Result<NeuraDevice, String>>,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("bevy-neura".to_owned())
        .spawn(move || serve(request, commands, reports, opening))
        .expect("a Neura device thread starts")
}

fn serve(
    request: RuntimeRequest,
    commands: Receiver<Command>,
    reports: Sender<NeuraReport>,
    opening: Sender<Result<NeuraDevice, String>>,
) {
    let runtime = match pollster::block_on(Runtime::open(request)) {
        Ok(runtime) => runtime,
        Err(unavailable) => {
            let _ = opening.send(Err(unavailable.to_string()));
            return;
        }
    };
    if opening.send(Ok(NeuraDevice::of(&runtime))).is_err() {
        return;
    }
    let mut models: HashMap<ModelId, Model<'_>> = HashMap::new();
    while let Ok(command) = commands.recv() {
        match apply::apply(&runtime, &mut models, command) {
            Some(report) => {
                if reports.send(report).is_err() {
                    return;
                }
            }
            None => return,
        }
    }
}
