use crate::command::Command;
use crate::device::NeuraDevice;
use crate::ids::{ModelId, RequestId};
use crate::plan::{ModelPlan, ModelRecord};
use crate::report::{ModelGeometry, NeuraReport};
use crate::role::Role;
use crate::sample::{Extents, Loss, Sample};
use crate::worker;
use bevy::prelude::Resource;
use neura::RuntimeRequest;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;

#[derive(Resource)]
pub struct NeuraHandle {
    commands: Mutex<Sender<Command>>,
    reports: Mutex<Receiver<NeuraReport>>,
    stash: Mutex<VecDeque<NeuraReport>>,
    opening: Mutex<Option<Receiver<Result<NeuraDevice, String>>>>,
    table: Mutex<HashMap<ModelId, ModelRecord>>,
    worker: Mutex<Option<JoinHandle<()>>>,
    requests: AtomicU64,
    models: AtomicU64,
    stopped: AtomicBool,
}

impl NeuraHandle {
    pub(crate) fn spawn(request: RuntimeRequest) -> Self {
        let (commander, commands) = std::sync::mpsc::channel();
        let (reports, receiver) = std::sync::mpsc::channel();
        let (opened, opening) = std::sync::mpsc::channel();
        let worker = worker::spawn(request, commands, reports, opened);
        Self {
            commands: Mutex::new(commander),
            reports: Mutex::new(receiver),
            stash: Mutex::new(VecDeque::new()),
            opening: Mutex::new(Some(opening)),
            table: Mutex::new(HashMap::new()),
            worker: Mutex::new(Some(worker)),
            requests: AtomicU64::new(1),
            models: AtomicU64::new(1),
            stopped: AtomicBool::new(false),
        }
    }

    pub fn attach(&self, plan: ModelPlan) -> ModelId {
        self.attach_of(plan, None)
    }

    pub fn attach_shared(&self, source: ModelId, plan: ModelPlan) -> ModelId {
        self.record(source);
        self.attach_of(plan, Some(source))
    }

    fn attach_of(&self, plan: ModelPlan, source: Option<ModelId>) -> ModelId {
        let model = ModelId::of(self.models.fetch_add(1, Ordering::Relaxed));
        let replaced = self.table().insert(model, plan.record());
        assert!(
            replaced.is_none(),
            "two models answer to {model:?} before the device reaches the first",
        );
        self.send(match source {
            Some(source) => Command::Share {
                model,
                source,
                plan,
            },
            None => Command::Attach { model, plan },
        });
        model
    }

    pub fn detach(&self, model: ModelId) {
        let detached = self.table().remove(&model);
        assert!(
            detached.is_some(),
            "a detach names the model {model:?}, which is no attached model",
        );
        self.send(Command::Detach { model });
    }

    pub fn train(
        &self,
        model: ModelId,
        extents: Extents,
        samples: Vec<Sample>,
        loss: Loss,
    ) -> RequestId {
        assert!(
            !samples.is_empty(),
            "a training of no sample runs nothing on the device",
        );
        let record = self.record(model);
        assert!(
            loss == Loss::None || record.roles.loss_value().is_some(),
            "a training of the model {model:?} reads a loss no role of it declares",
        );
        let lengths = record.roles.lengths(&extents);
        for sample in &samples {
            record.roles.assert_writes(sample, &lengths);
        }
        let request = self.next_request();
        self.send(Command::Train {
            request,
            model,
            lengths,
            samples,
            loss,
        });
        request
    }

    pub fn infer(&self, model: ModelId, extents: Extents, sample: Sample) -> RequestId {
        let record = self.record(model);
        assert!(
            !record.updates,
            "an inference of the model {model:?} runs the descents of the graph that trains it; attach an inference graph that shares its weights",
        );
        let lengths = record.roles.lengths(&extents);
        record.roles.assert_writes(&sample, &lengths);
        let request = self.next_request();
        self.send(Command::Infer {
            request,
            model,
            lengths,
            sample,
        });
        request
    }

    pub fn read(&self, model: ModelId, extents: Extents, roles: Vec<Role>) -> RequestId {
        assert!(
            !roles.is_empty(),
            "a read names at least one output of the model",
        );
        let declared = self.record(model);
        for role in &roles {
            declared.roles.output_value(role);
        }
        let lengths = declared.roles.lengths(&extents);
        let request = self.next_request();
        self.send(Command::Read {
            request,
            model,
            lengths,
            roles,
        });
        request
    }

    pub fn tune(&self, model: ModelId, extents: Extents) -> RequestId {
        let record = self.record(model);
        let lengths = record.roles.lengths(&extents);
        let request = self.next_request();
        self.send(Command::Tune {
            request,
            model,
            lengths,
        });
        request
    }

    pub fn save(&self, model: ModelId, path: impl Into<PathBuf>) -> RequestId {
        self.record(model);
        let request = self.next_request();
        self.send(Command::Save {
            request,
            model,
            path: path.into(),
        });
        request
    }

    pub fn restore(&self, model: ModelId, path: impl Into<PathBuf>) -> RequestId {
        self.record(model);
        let request = self.next_request();
        self.send(Command::Restore {
            request,
            model,
            path: path.into(),
        });
        request
    }

    pub fn wait(&self, request: RequestId) -> NeuraReport {
        self.claim(|report| report.request() == Some(request))
    }

    pub fn wait_attached(&self, model: ModelId) -> ModelGeometry {
        let report = self.claim(|candidate| {
            matches!(candidate, NeuraReport::Attached { model: attached, .. } if *attached == model)
        });
        match report {
            NeuraReport::Attached { geometry, .. } => geometry,
            _ => unreachable!("the claim of an attachment answers an attachment"),
        }
    }

    pub fn reports(&self) -> Vec<NeuraReport> {
        let mut batch = Vec::new();
        {
            let mut stash = self
                .stash
                .lock()
                .expect("the report stash of the device thread is never poisoned");
            batch.extend(stash.drain(..));
        }
        let channel = self
            .reports
            .lock()
            .expect("the report channel of the device thread is never poisoned");
        loop {
            match channel.try_recv() {
                Ok(report) => batch.push(report),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    assert!(
                        self.stopped.load(Ordering::Relaxed),
                        "the Neura device thread stopped, and the reports it owes this app never arrive",
                    );
                    break;
                }
            }
        }
        batch
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        let _ = self
            .commands
            .lock()
            .expect("the command channel of the device thread is never poisoned")
            .send(Command::Shutdown);
        if let Some(worker) = self
            .worker
            .lock()
            .expect("the handle of the device thread is never poisoned")
            .take()
        {
            let _ = worker.join();
        }
    }

    pub(crate) fn open(&self) -> NeuraDevice {
        let opening = self
            .opening
            .lock()
            .expect("the opening channel of the device thread is never poisoned")
            .take()
            .unwrap_or_else(|| panic!("the Neura device opens once per app"));
        match opening.recv() {
            Ok(Ok(device)) => device,
            Ok(Err(reason)) => panic!("the Neura device does not open: {reason}"),
            Err(_) => panic!(
                "the Neura device thread stopped before a device opened; its panic prints above this line"
            ),
        }
    }

    fn claim(&self, mut matches: impl FnMut(&NeuraReport) -> bool) -> NeuraReport {
        {
            let mut stash = self
                .stash
                .lock()
                .expect("the report stash of the device thread is never poisoned");
            if let Some(index) = stash.iter().position(&mut matches) {
                return stash
                    .remove(index)
                    .expect("a claimed report sits in the stash of the device thread");
            }
        }
        loop {
            let report = self
                .reports
                .lock()
                .expect("the report channel of the device thread is never poisoned")
                .recv()
                .unwrap_or_else(|_| {
                    panic!(
                        "the Neura device thread stopped, and the report of a caller never arrives"
                    )
                });
            if matches(&report) {
                return report;
            }
            self.stash
                .lock()
                .expect("the report stash of the device thread is never poisoned")
                .push_back(report);
        }
    }

    fn record(&self, model: ModelId) -> ModelRecord {
        self.table()
            .get(&model)
            .cloned()
            .unwrap_or_else(|| panic!("the model {model:?} is no attached model"))
    }

    fn table(&self) -> MutexGuard<'_, HashMap<ModelId, ModelRecord>> {
        self.table
            .lock()
            .expect("the model table of the device thread is never poisoned")
    }

    fn next_request(&self) -> RequestId {
        RequestId::of(self.requests.fetch_add(1, Ordering::Relaxed))
    }

    fn send(&self, command: Command) {
        self.commands
            .lock()
            .expect("the command channel of the device thread is never poisoned")
            .send(command)
            .unwrap_or_else(|_| {
                panic!("the Neura device thread stopped, and no command reaches it")
            });
    }
}

impl Drop for NeuraHandle {
    fn drop(&mut self) {
        self.stop();
    }
}
