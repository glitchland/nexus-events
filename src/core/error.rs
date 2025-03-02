//! Error types for the nexus-events crate

use std::fmt;
use std::error::Error as StdError;

/// Error types for the event system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventError {
    /// Handler with the specified ID was not found
    HandlerNotFound { id: usize },
    /// The specified event type isn't registered
    EventTypeNotFound,
    /// An error occurred during event publishing
    PublishError { details: String },
    /// An error occurred during handler execution
    HandlerError { details: String },
    /// Error when trying to register a duplicate handler
    DuplicateHandler { id: usize },
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::HandlerNotFound { id } => 
                write!(f, "Handler with ID {} not found", id),
            EventError::EventTypeNotFound => 
                write!(f, "No handlers registered for this event type"),
            EventError::PublishError { details } => 
                write!(f, "Error publishing event: {}", details),
            EventError::HandlerError { details } => 
                write!(f, "Error in event handler: {}", details),
            EventError::DuplicateHandler { id } => 
                write!(f, "Handler with ID {} already registered", id),
        }
    }
}

impl StdError for EventError {}

/// Type alias for Results from the event system
pub type EventResult<T> = Result<T, EventError>;