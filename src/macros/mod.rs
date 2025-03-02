//! Runtime macros for the nexus-events event system.
//!
//! This module provides declarative macros that simplify common tasks within the event system.
//! Unlike proc macros which run at compile time, these runtime macros primarily help with
//! reducing boilerplate code through text substitution.
//!
//! The main macro provided is `define_event!`, which makes event definition more concise.

pub(crate) mod define_event;
