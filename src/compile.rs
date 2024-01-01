use std::collections::HashMap;

use indexmap::IndexMap;

use anyhow::{anyhow, Result};

use crate::graph::*;
use crate::node::*;
use crate::values::*;

#[derive(Clone, Debug)]
pub struct CompiledValue {
  pub value: String,
  pub dt: DataType,
}

impl core::fmt::Display for CompiledValue {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.value)
  }
}

impl CompiledValue {
  pub fn convert(&mut self, to_dt: DataType) -> Result<()> {
    self.value = match (self.dt, to_dt) {
      // Same data type, no conversion.
      (from, to) if from == to => {
        return Ok(());
      },
      (DataType::Dynamic, _) => {
        return Err(anyhow!("Dynamic compiled values are not supported"));
      }
      (DataType::DynamicVector, _) => {
        return Err(anyhow!("Dynamic Vector compiled values are not supported"));
      }
      (DataType::DynamicMatrix, _) => {
        return Err(anyhow!("Dynamic Matrix compiled values are not supported"));
      }
      // From I32
      (DataType::I32, DataType::U32) => format!("u32({})", self.value),
      (DataType::I32, DataType::F32) => format!("f32({})", self.value),
      (DataType::I32, DataType::Vec2) => format!("vec2<f32>({}, 0.)", self.value),
      (DataType::I32, DataType::Vec3) => format!("vec3<f32>({}, 0., 0.)", self.value),
      (DataType::I32, DataType::Vec4) => format!("vec4<f32>({}, 0., 0., 1.)", self.value),
      (DataType::I32, DataType::Dynamic | DataType::DynamicVector) => {
        return Ok(());
      },
      // From U32
      (DataType::U32, DataType::I32) => format!("i32({})", self.value),
      (DataType::U32, DataType::F32) => format!("f32({})", self.value),
      (DataType::U32, DataType::Vec2) => format!("vec2<f32>({}, 0.)", self.value),
      (DataType::U32, DataType::Vec3) => format!("vec3<f32>({}, 0., 0.)", self.value),
      (DataType::U32, DataType::Vec4) => format!("vec4<f32>({}, 0., 0., 1.)", self.value),
      (DataType::U32, DataType::Dynamic | DataType::DynamicVector) => {
        return Ok(());
      },
      // From F32
      (DataType::F32, DataType::I32) => format!("i32({})", self.value),
      (DataType::F32, DataType::U32) => format!("u32({})", self.value),
      (DataType::F32, DataType::Vec2) => format!("vec2<f32>({}, 0.)", self.value),
      (DataType::F32, DataType::Vec3) => format!("vec3<f32>({}, 0., 0.)", self.value),
      (DataType::F32, DataType::Vec4) => format!("vec4<f32>({}, 0., 0., 1.)", self.value),
      (DataType::F32, DataType::Dynamic | DataType::DynamicVector) => {
        return Ok(());
      },
      // From Vec2
      (DataType::Vec2, DataType::I32) =>  format!("i32({}.x)", self.value),
      (DataType::Vec2, DataType::U32) =>  format!("u32({}.x)", self.value),
      (DataType::Vec2, DataType::F32) =>  format!("f32({}.x)", self.value),
      (DataType::Vec2, DataType::Vec3) => format!("vec3<f32>({}.xy, 0.)", self.value),
      (DataType::Vec2, DataType::Vec4) => format!("vec4<f32>({}.xy, 0., 1.)", self.value),
      (DataType::Vec2, DataType::Dynamic | DataType::DynamicVector) => {
        return Ok(());
      },
      // From Vec3
      (DataType::Vec3, DataType::I32) =>  format!("i32({}.x)", self.value),
      (DataType::Vec3, DataType::U32) =>  format!("u32({}.x)", self.value),
      (DataType::Vec3, DataType::F32) =>  format!("f32({}.x)", self.value),
      (DataType::Vec3, DataType::Vec2) => format!("vec2<f32>({}.xy)", self.value),
      (DataType::Vec3, DataType::Vec4) => format!("vec4<f32>({}.xyz, 1.)", self.value),
      (DataType::Vec3, DataType::Dynamic | DataType::DynamicVector) => {
        return Ok(());
      },
      // From Vec4
      (DataType::Vec4, DataType::I32) =>  format!("i32({}.x)", self.value),
      (DataType::Vec4, DataType::U32) =>  format!("u32({}.x)", self.value),
      (DataType::Vec4, DataType::F32) =>  format!("f32({}.x)", self.value),
      (DataType::Vec4, DataType::Vec2) => format!("vec2<f32>({}.xy)", self.value),
      (DataType::Vec4, DataType::Vec3) => format!("vec4<f32>({}.xyz)", self.value),
      (DataType::Vec4, DataType::Dynamic | DataType::DynamicVector) => {
        return Ok(());
      },
      // From Mat2
      (DataType::Mat2, DataType::Mat3) => {
        return Err(anyhow!("Promoting Mat2 to Mat3 not supported."));
      },
      (DataType::Mat2, DataType::Mat4) => {
        return Err(anyhow!("Promoting Mat2 to Mat4 not supported."));
      },
      (DataType::Mat2, DataType::Dynamic | DataType::DynamicMatrix) => {
        return Ok(());
      },
      // From Mat3
      (DataType::Mat3, DataType::Mat2) => format!("mat2x2<f32>({}[0].xy, {}[1].xy)", self.value, self.value),
      (DataType::Mat3, DataType::Mat4) => {
        return Err(anyhow!("Promoting Mat3 to Mat4 not supported."));
      },
      (DataType::Mat3, DataType::Dynamic | DataType::DynamicMatrix) => {
        return Ok(());
      },
      // From Mat4
      (DataType::Mat4, DataType::Mat2) => format!("mat2x2<f32>({}[0].xy, {}[1].xy)", self.value, self.value),
      (DataType::Mat4, DataType::Mat3) => format!("mat3x3<f32>({}[0].xyz, {}[1].xyz, {}[2].xyz)", self.value, self.value, self.value),
      (DataType::Mat4, DataType::Dynamic | DataType::DynamicMatrix) => {
        return Ok(());
      },
      // Unsupport conversions
      (from_dt, to_dt) => {
        return Err(anyhow!("Conversion from {from_dt:?} to {to_dt:?} not supported."));
      }
    };
    self.dt = to_dt;
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub enum NodeOutput {
  /// Only generated if it is used.
  LazyCode(String, String, DataType),
  /// Already generated, so reference the variable.
  Compiled(CompiledValue),
}

/// Code block Id
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct CodeBlockId(u32);

#[derive(Clone, Default, Debug)]
pub struct CodeBlock {
  code: Vec<String>,
  variables: IndexMap<String, DataType>,
  outputs: IndexMap<OutputId, NodeOutput>,
  counter: usize,
}

impl CodeBlock {
  pub fn add_local(&mut self, prefix: &str, code: String, dt: DataType) -> String {
    self.counter += 1;
    let idx = self.counter;
    let name = format!("{prefix}_{idx}");
    self.append(format!(
      r#"
  let {name} = {code};"#));
    self.variables.insert(name.clone(), dt);
    name
  }

  pub fn add_output(&mut self, id: OutputId, prefix: &str, code: String, dt: DataType) {
    self.outputs.insert(id, NodeOutput::LazyCode(
      prefix.to_string(), code, dt
    ));
  }

  pub fn resolve_output(&mut self, id: OutputId) -> Result<CompiledValue> {
    let output = self.outputs.get(&id)
      .ok_or_else(|| anyhow!("Tried to resolve an unknown output: {id:?}"))?;
    // Generate the output if it hasn't already been generated.
    let value = match output.clone() {
      NodeOutput::LazyCode(prefix, code, dt) => {
        let name = self.add_local(&prefix, code, dt);
        let value = CompiledValue {
          value: name, dt
        };
        // Mark the output as generated and reference the variable.
        self.outputs.insert(id, NodeOutput::Compiled(value.clone()));
        Ok(value)
      }
      NodeOutput::Compiled(value) => Ok(value),
    };
    value
  }

  pub fn append_output(&mut self, node: NodeId, code: String) {
    let id = OutputId {
      node,
      idx: 0,
    };
    let dt = DataType::Vec4;
    let name = self.add_local("out", code, dt);
    let value = CompiledValue {
      value: name, dt
    };
    self.outputs.insert(id, NodeOutput::Compiled(value));
  }

  pub fn append(&mut self, code: String) {
    self.code.push(code);
  }

  pub fn clear(&mut self) {
    self.code.clear();
    self.variables.clear();
    self.outputs.clear();
    self.counter = 0;
  }

  pub fn dump(&self) -> String {
    self.code.join("")
  }
}

#[derive(Default, Debug)]
pub struct NodeGraphCompile {
  next_id: CodeBlockId,
  blocks: IndexMap<CodeBlockId, CodeBlock>,
  names: HashMap<String, CodeBlockId>,
  block_order: Vec<CodeBlockId>,
  stack: Vec<CodeBlockId>,
  compiled: HashMap<NodeId, bool>,
}

impl NodeGraphCompile {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn define_block(&mut self, name: &str) -> CodeBlockId {
    if let Some(&id) = self.names.get(name) {
      return id;
    }
    let id = CodeBlockId(self.next_id.0 + 1);
    self.next_id = id;
    self.blocks.insert(id, CodeBlock::default());
    self.block_order.push(id);
    self.names.insert(name.to_string(), id);
    id
  }

  pub fn current_block(&mut self) -> Result<&mut CodeBlock> {
    self
      .stack
      .last()
      .and_then(|&id| self.blocks.get_mut(&id))
      .ok_or_else(|| anyhow!("No current block on stack"))
  }

  pub fn push(&mut self, id: CodeBlockId) {
    self.stack.push(id);
  }

  pub fn push_new_block(&mut self, name: &str) -> CodeBlockId {
    let id = self.define_block(name);
    self.push(id);
    id
  }

  pub fn pop(&mut self, expect: Option<CodeBlockId>) -> Result<Option<CodeBlockId>> {
    let id = self.stack.pop();
    if id != expect {
      Err(anyhow!("Stack corruption detected."))
    } else {
      Ok(id)
    }
  }

  pub fn get_block(&self, name: &str) -> Option<&CodeBlock> {
    self
      .names
      .get(name)
      .and_then(|&id| self.get_block_by_id(id))
  }

  pub fn get_block_mut(&mut self, name: &str) -> Option<&mut CodeBlock> {
    self
      .names
      .get(name)
      .copied()
      .and_then(|id| self.get_block_by_id_mut(id))
  }

  pub fn get_block_by_id(&self, id: CodeBlockId) -> Option<&CodeBlock> {
    self.blocks.get(&id)
  }

  pub fn get_block_by_id_mut(&mut self, id: CodeBlockId) -> Option<&mut CodeBlock> {
    self.blocks.get_mut(&id)
  }

  pub fn add_output(&mut self, id: OutputId, prefix: &str, code: String, dt: DataType) -> Result<()> {
    let block = self.current_block()?;
    block.add_output(id, prefix, code, dt);
    Ok(())
  }

  pub fn append_code(&mut self, name: &str, code: String) -> Result<()> {
    match self.get_block_mut(name) {
      Some(block) => {
        block.append(code);
        Ok(())
      }
      None => Err(anyhow!("Undefined block: {name:?}")),
    }
  }

  pub fn dump(&self) -> String {
    let mut output = Vec::new();
    for block in self.blocks.values() {
      output.push(block.dump());
    }
    output.join("")
  }

  pub fn clear(&mut self) {
    for block in self.blocks.values_mut() {
      block.clear();
    }
  }

  pub fn resolve_output(&mut self, graph: &NodeGraph, id: OutputId) -> Result<CompiledValue> {
    // Make sure the output node has been compiled.
    self.compile_node(graph, id.node)?;
    let block = self.current_block()?;
    block.resolve_output(id)
  }

  pub fn compile_graph(&mut self, graph: &NodeGraph) -> Result<()> {
    let id = graph
      .output()
      .ok_or_else(|| anyhow!("Graph missing output node"))?;
    self.compile_node(graph, id)
  }

  pub fn compile_node(&mut self, graph: &NodeGraph, id: NodeId) -> Result<()> {
    let compiled = self.compiled.insert(id, true).unwrap_or_default();
    if compiled {
      return Ok(());
    }
    let node = graph.get(id)?;
    // compile node.
    node.compile(graph, self, id)
  }
}
