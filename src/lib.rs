#![warn(clippy::all)]

pub mod values;
pub use values::*;
pub mod node;
pub use node::*;
pub mod graph;
pub use graph::*;
pub mod macros;

pub mod eval;
pub use eval::*;
pub mod compile;
pub use compile::*;

pub mod nodes;
pub use nodes::*;

#[cfg(feature = "egui")]
pub mod ui;