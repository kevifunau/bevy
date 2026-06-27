use bevy_app::Update;
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemSet};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) enum BuiSystems {
    DataUpdate,
    BindingSync,
    VisualResolve,
}

pub(crate) fn configure_bui_system_sets(app: &mut bevy_app::App) {
    app.configure_sets(
        Update,
        (
            BuiSystems::DataUpdate,
            BuiSystems::BindingSync,
            BuiSystems::VisualResolve,
        )
            .chain(),
    );
}
