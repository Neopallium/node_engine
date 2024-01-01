#![warn(clippy::all)]

pub mod values;
pub use values::*;
pub mod input;
pub use input::*;
pub mod group;
pub use group::*;
pub mod node;
pub use node::*;
pub mod registry;
pub use registry::*;
pub mod graph;
pub use graph::*;
pub mod macros;

pub mod color;
pub use color::*;

pub mod eval;
pub use eval::*;
pub mod compile;
pub use compile::*;

pub mod nodes;

// pre-export for use in `impl_node` macro.
#[doc(hidden)]
pub extern crate heck;
#[doc(hidden)]
pub extern crate inventory;
#[doc(hidden)]
pub extern crate lazy_static;
#[doc(hidden)]
pub extern crate serde;

#[cfg(feature = "egui")]
pub mod ui;
