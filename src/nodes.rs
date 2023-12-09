use crate::graph::*;

pub mod math;

pub mod shader;

pub fn build_registry() -> NodeRegistry {
  let reg = NodeRegistry::new();
  math::register(&reg);
  shader::register(&reg);
  reg
}
