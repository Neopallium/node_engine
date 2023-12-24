use crate::registry::*;

pub mod math;

pub mod shader;

pub fn build_registry() -> NodeRegistry {
  NodeRegistry::build()
}
