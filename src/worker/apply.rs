use crate::command::Command;
use crate::ids::ModelId;
use crate::report::NeuraReport;
use crate::worker::model::Model;
use neura::Runtime;
use std::collections::HashMap;

pub(crate) fn apply<'r>(
    runtime: &'r Runtime,
    models: &mut HashMap<ModelId, Model<'r>>,
    command: Command,
) -> Option<NeuraReport> {
    match command {
        Command::Attach { model, plan } => {
            let attached = Model::attach(runtime, plan);
            let report = NeuraReport::Attached {
                model,
                geometry: attached.geometry(),
            };
            assert!(
                models.insert(model, attached).is_none(),
                "the model {model:?} attaches twice",
            );
            Some(report)
        }
        Command::Share {
            model,
            source,
            plan,
        } => {
            let shared = Model::share(runtime, held(models, source).weights(), plan);
            let report = NeuraReport::Attached {
                model,
                geometry: shared.geometry(),
            };
            assert!(
                models.insert(model, shared).is_none(),
                "the model {model:?} attaches twice",
            );
            Some(report)
        }
        Command::Detach { model } => {
            assert!(
                models.remove(&model).is_some(),
                "a detach names the model {model:?}, which no attach of this device reached",
            );
            Some(NeuraReport::Detached { model })
        }
        Command::Train {
            request,
            model,
            lengths,
            samples,
            loss,
        } => {
            let steps = samples.len() as u32;
            let (losses, seconds) = held(models, model).train(runtime, &lengths, &samples, loss);
            Some(NeuraReport::Trained {
                request,
                model,
                steps,
                losses,
                seconds,
            })
        }
        Command::Infer {
            request,
            model,
            lengths,
            sample,
        } => {
            let (values, seconds) = held(models, model).infer(runtime, &lengths, &sample);
            Some(NeuraReport::Inferred {
                request,
                model,
                values,
                seconds,
            })
        }
        Command::Read {
            request,
            model,
            lengths,
            roles,
        } => {
            let values = held(models, model).read(runtime, &lengths, &roles);
            Some(NeuraReport::Read {
                request,
                model,
                values,
            })
        }
        Command::Tune {
            request,
            model,
            lengths,
        } => {
            let (profile, seconds) = trained(models, model).tune(runtime, &lengths);
            Some(NeuraReport::Tuned {
                request,
                model,
                profile: Box::new(profile),
                seconds,
            })
        }
        Command::Save {
            request,
            model,
            path,
        } => {
            let bytes = held(models, model).save(runtime, &path);
            Some(NeuraReport::Saved {
                request,
                model,
                bytes,
            })
        }
        Command::Restore {
            request,
            model,
            path,
        } => {
            held(models, model).restore(runtime, &path);
            Some(NeuraReport::Restored { request, model })
        }
        Command::Shutdown => None,
    }
}

fn held<'m, 'r>(models: &'m HashMap<ModelId, Model<'r>>, model: ModelId) -> &'m Model<'r> {
    models
        .get(&model)
        .unwrap_or_else(|| panic!("the model {model:?} is no attached model of this device"))
}

fn trained<'m, 'r>(
    models: &'m mut HashMap<ModelId, Model<'r>>,
    model: ModelId,
) -> &'m mut Model<'r> {
    models
        .get_mut(&model)
        .unwrap_or_else(|| panic!("the model {model:?} is no attached model of this device"))
}
