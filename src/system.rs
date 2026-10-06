use crate::handle::NeuraHandle;
use crate::report::NeuraReport;
use bevy::prelude::{Commands, MessageWriter, Res};

pub(crate) fn open_device(handle: Res<NeuraHandle>, mut commands: Commands) {
    commands.insert_resource(handle.open());
}

pub(crate) fn drain_reports(handle: Res<NeuraHandle>, mut reports: MessageWriter<NeuraReport>) {
    for report in handle.reports() {
        reports.write(report);
    }
}
