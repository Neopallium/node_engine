use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use slotmap::SlotMap;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use crate::*;

#[derive(Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct NodeRegistryInner {
  nodes: HashMap<Uuid, NodeDefinition>,
  name_to_id: HashMap<String, Uuid>,
}

impl NodeRegistryInner {
  pub fn nodes(&self) -> Vec<NodeDefinition> {
    self.nodes.values().cloned().collect()
  }

  fn register(&mut self, def: NodeDefinition) {
    self.name_to_id.insert(def.name.clone(), def.uuid);
    self.nodes.insert(def.uuid, def);
  }

  fn new_by_name(&self, name: &str) -> Option<NodeState> {
    self.name_to_id.get(name).and_then(|&id| self.new_by_id(id))
  }

  fn new_by_id(&self, id: Uuid) -> Option<NodeState> {
    self.nodes.get(&id).map(NodeState::new)
  }
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeRegistry(Arc<RwLock<NodeRegistryInner>>);

impl NodeRegistry {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn nodes(&self) -> Vec<NodeDefinition> {
    let inner = self.0.read().unwrap();
    inner.nodes()
  }

  pub fn add(&self, def: NodeDefinition) {
    let mut inner = self.0.write().unwrap();
    inner.register(def)
  }

  pub fn register<T: GetNodeDefinition>(&self) {
    let def = T::node_definition();
    let mut inner = self.0.write().unwrap();
    inner.register(def)
  }

  pub fn new_by_id(&self, id: Uuid) -> Option<NodeState> {
    let inner = self.0.read().unwrap();
    inner.new_by_id(id)
  }

  pub fn new_by_name(&self, name: &str) -> Option<NodeState> {
    let inner = self.0.read().unwrap();
    inner.new_by_name(name)
  }
}

#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeGraph {
  nodes: SlotMap<NodeId, NodeState>,
  connections: HashMap<InputId, OutputId>,
  output: Option<NodeId>,
}

impl NodeGraph {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add(&mut self, node: NodeState) -> NodeId {
    self.nodes.insert(node)
  }

  pub fn remove(&mut self, id: NodeId) -> Option<NodeState> {
    self.nodes.remove(id)
  }

  pub fn contains(&self, id: NodeId) -> bool {
    self.nodes.contains_key(id)
  }

  pub fn get_input_id<I: Into<InputKey>>(&self, id: NodeId, idx: I) -> Result<InputId> {
    let node = self.get(id)?;
    let idx = node.get_input_idx(&idx.into())?;
    Ok(InputId(id, idx))
  }

  pub fn get_node_input<I: Into<InputKey>>(&self, id: NodeId, idx: I) -> Result<Input> {
    self.get(id).and_then(|n| n.get_node_input(&idx.into()))
  }

  pub fn set_node_input<I: Into<InputKey>>(
    &mut self,
    id: NodeId,
    key: I,
    value: Input,
  ) -> Result<Option<OutputId>> {
    let key = key.into();
    // Get node.
    let node = self
      .nodes
      .get_mut(id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))?;
    // Convert Input key to id.
    let input_id = node.get_input_idx(&key).map(|idx| InputId(id, idx))?;
    match &value {
      Input::Disconnect => {
        self.connections.remove(&input_id);
      }
      Input::Connect(output_id) => {
        self.connections.insert(input_id, *output_id);
      }
      _ => {}
    }
    // Set the node input.
    Ok(node.set_node_input(&key, value)?)
  }

  pub fn set_input(&mut self, input_id: InputId, value: Input) -> Result<Option<OutputId>> {
    self.set_node_input(input_id.node(), input_id, value)
  }

  pub fn disconnect(&mut self, input: InputId) -> Result<()> {
    self.set_input(input, Input::Disconnect)?;
    Ok(())
  }

  pub fn connect(&mut self, input: InputId, output: OutputId) -> Result<()> {
    self.set_input(input, Input::Connect(output))?;
    Ok(())
  }

  pub fn get(&self, id: NodeId) -> Result<&NodeState> {
    self
      .nodes
      .get(id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))
  }

  pub fn get_mut(&mut self, id: NodeId) -> Result<&mut NodeState> {
    self
      .nodes
      .get_mut(id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))
  }

  pub fn set_output(&mut self, output: Option<NodeId>) {
    self.output = output;
  }

  pub fn output(&self) -> Option<NodeId> {
    self.output
  }
}
