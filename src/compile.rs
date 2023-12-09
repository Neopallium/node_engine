use std::collections::HashMap;

use slotmap::{SecondaryMap, SlotMap};

use anyhow::{anyhow, Result};

use crate::graph::*;
use crate::values::*;

slotmap::new_key_type! {
  /// Code block Id
  pub struct CodeBlockId;
}

#[derive(Clone, Default, Debug)]
pub struct CodeBlock {
  code: Vec<String>,
  counter: usize,
}

impl CodeBlock {
  pub fn new_local(&mut self, prefix: &str) -> String {
    self.counter += 1;
    let idx = self.counter;
    let name = format!("{prefix}_{idx}");
    name
  }

  pub fn append_output(&mut self, node: NodeId, out: String) {
    self.append(format!(
      r#"
  let out_{} = {out};"#,
      node_idx(node)
    ));
  }

  pub fn append(&mut self, code: String) {
    self.code.push(code);
  }

  pub fn clear(&mut self) {
    self.code.clear();
  }

  pub fn dump(&self) -> String {
    self.code.join("")
  }
}

#[derive(Default, Debug)]
pub struct NodeGraphCompile {
  blocks: SlotMap<CodeBlockId, CodeBlock>,
  names: HashMap<String, CodeBlockId>,
  block_order: Vec<CodeBlockId>,
  stack: Vec<CodeBlockId>,
  compiled: SecondaryMap<NodeId, bool>,
}

impl NodeGraphCompile {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn define_block(&mut self, name: &str) -> CodeBlockId {
    if let Some(&id) = self.names.get(name) {
      return id;
    }
    let id = self.blocks.insert(CodeBlock::default());
    self.block_order.push(id);
    self.names.insert(name.to_string(), id);
    id
  }

  pub fn current_block(&mut self) -> Result<&mut CodeBlock> {
    self
      .stack
      .last()
      .and_then(|&id| self.blocks.get_mut(id))
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
    self.blocks.get(id)
  }

  pub fn get_block_by_id_mut(&mut self, id: CodeBlockId) -> Option<&mut CodeBlock> {
    self.blocks.get_mut(id)
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

  pub fn compile_value(&mut self, value: &Value) -> Result<String> {
    Ok(match value {
      Value::Scalar(val) => {
        format!("{val:?}")
      }
      Value::Vec2(v) => {
        format!("vec2<f32>({:?}, {:?})", v.x, v.y)
      }
      Value::Vec3(v) => {
        format!("vec3<f32>({:?}, {:?}, {:?})", v.x, v.y, v.z)
      }
      Value::Vec4(v) => {
        format!("vec4<f32>({:?}, {:?}, {:?}, {:?})", v.x, v.y, v.z, v.w)
      }
    })
  }

  pub fn resolve_node(&mut self, graph: &NodeGraph, id: NodeId) -> Result<String> {
    // Make sure the input node has been compiled.
    self.compile_node(graph, id)?;
    Ok(format!("out_{}", node_idx(id)))
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
