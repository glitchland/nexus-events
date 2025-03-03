pub mod core;

// Re-export the macros so user code can do `use nexus_events::...`
pub use nexus_events_macros::{event_component, event_handler, event_sender};

// A "prelude" for convenience
pub mod prelude {
    pub use crate::core::{
        EventBus, Event, HandlerId, subscribe, dispatch, process_events, unsubscribe,
    };

    pub use nexus_events_macros::{event_component, event_handler, event_sender};
}
