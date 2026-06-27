use std::{collections::HashMap, sync::Arc};

use bevy_app::{App, Update};
use bevy_ecs::{message::Messages, prelude::*};

use crate::core::interaction::types::BuiActionTriggered;

type BuiActionHandler = Arc<dyn Fn(&mut World, &BuiActionTriggered) + Send + Sync + 'static>;

/// Runtime registry that routes declarative BUI actions to game-side ECS handlers.
///
/// This is the Bevy-side equivalent of a UGUI listener or a backend API route:
/// UI markup emits a stable action name, while game code owns the business logic.
#[derive(Resource, Clone, Default)]
pub struct BuiActionRegistry {
    handlers: HashMap<String, Vec<BuiActionHandler>>,
}

impl BuiActionRegistry {
    /// Registers a handler for an action name emitted by BUI markup.
    pub fn register(
        &mut self,
        action: impl Into<String>,
        handler: impl Fn(&mut World, &BuiActionTriggered) + Send + Sync + 'static,
    ) -> &mut Self {
        self.handlers
            .entry(action.into())
            .or_default()
            .push(Arc::new(handler));
        self
    }

    /// Returns true when at least one handler is registered for the action name.
    pub fn contains(&self, action: &str) -> bool {
        self.handlers.contains_key(action)
    }

    fn handlers_for(&self, action: &str) -> Option<&[BuiActionHandler]> {
        self.handlers.get(action).map(Vec::as_slice)
    }
}

#[derive(Resource, Default)]
pub(crate) struct BuiActionRegistryCursor {
    cursor: bevy_ecs::message::MessageCursor<BuiActionTriggered>,
}

/// Extension trait for registering game-side BUI action handlers on a Bevy app.
pub trait BuiActionAppExt {
    /// Registers an ECS-side handler for a declarative BUI action name.
    ///
    /// ```
    /// # use bevy_app::App;
    /// # use bevy_ai_ui_parser::BuiActionAppExt;
    /// let mut app = App::new();
    /// app.add_bui_action_handler("start-race", |_world, event| {
    ///     assert_eq!(event.action, "start-race");
    /// });
    /// ```
    fn add_bui_action_handler(
        &mut self,
        action: impl Into<String>,
        handler: impl Fn(&mut World, &BuiActionTriggered) + Send + Sync + 'static,
    ) -> &mut Self;
}

impl BuiActionAppExt for App {
    fn add_bui_action_handler(
        &mut self,
        action: impl Into<String>,
        handler: impl Fn(&mut World, &BuiActionTriggered) + Send + Sync + 'static,
    ) -> &mut Self {
        if !self.world().contains_resource::<BuiActionRegistry>() {
            self.init_resource::<BuiActionRegistry>();
        }
        if !self.world().contains_resource::<BuiActionRegistryCursor>() {
            self.init_resource::<BuiActionRegistryCursor>();
            self.add_systems(Update, dispatch_registered_bui_action_handlers_system);
        }

        self.world_mut()
            .resource_mut::<BuiActionRegistry>()
            .register(action, handler);
        self
    }
}

pub(crate) fn dispatch_registered_bui_action_handlers_system(world: &mut World) {
    let Some(registry) = world.get_resource::<BuiActionRegistry>().cloned() else {
        return;
    };
    if registry.handlers.is_empty() {
        return;
    }

    let actions = world.resource_scope(|world, mut cursor: Mut<BuiActionRegistryCursor>| {
        let Some(messages) = world.get_resource::<Messages<BuiActionTriggered>>() else {
            return Vec::new();
        };
        cursor
            .cursor
            .read(messages)
            .filter(|event| registry.contains(&event.action))
            .cloned()
            .collect::<Vec<_>>()
    });

    for action in actions {
        if let Some(handlers) = registry.handlers_for(&action.action) {
            for handler in handlers {
                handler(world, &action);
            }
        }
    }
}
