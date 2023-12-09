use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use slotmap::SlotMap;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use crate::*;

slotmap::new_key_type! {
  /// Node Id
  pub struct NodeId;
}

pub const NAMESPACE_NODE_IMPL: Uuid = uuid::uuid!("9dee91a8-5af8-11ee-948b-5364d73b1803");

pub fn node_idx(id: NodeId) -> u64 {
  id.0.as_ffi()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InputId(pub NodeId, pub u32);

impl InputId {
  pub fn node(&self) -> NodeId {
    self.0
  }

  pub fn key(&self) -> InputKey {
    self.1.into()
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutputId(pub NodeId, pub u32);

impl OutputId {
  pub fn node(&self) -> NodeId {
    self.0
  }
}

#[cfg_attr(feature = "serde", typetag::serde())]
pub trait NodeImpl: fmt::Debug {
  fn clone_node(&self) -> Box<dyn NodeImpl>;

  fn def(&self) -> &NodeDefinition;

  fn cache_output(&self) -> bool {
    false
  }

  fn get_input_idx(&self, key: &InputKey) -> Result<u32> {
    match key {
      InputKey::Idx(idx) => Ok(*idx),
      key => self.def().get_input_idx(key),
    }
  }

  fn get_node_input(&self, _idx: &InputKey) -> Result<Input> {
    Err(anyhow!("This node doesn't support `get_input`"))
  }

  fn set_node_input(&mut self, _idx: &InputKey, _value: Input) -> Result<Option<OutputId>> {
    Err(anyhow!("This node doesn't support `set_input`"))
  }

  fn get_param(&self, _name: &str) -> Result<ParameterValue> {
    Err(anyhow!("This node doesn't support `get_param`"))
  }

  fn set_param(&mut self, _name: &str, _value: ParameterValue) -> Result<()> {
    Err(anyhow!("This node doesn't support `get_param`"))
  }

  fn eval(
    &self,
    _graph: &NodeGraph,
    _execution: &mut NodeGraphExecution,
    _id: NodeId,
  ) -> Result<Value> {
    Err(anyhow!("This node doesn't support `eval`."))
  }

  fn compile(
    &self,
    _graph: &NodeGraph,
    _compile: &mut NodeGraphCompile,
    _id: NodeId,
  ) -> Result<()> {
    Err(anyhow!("This node doesn't support `compile`."))
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    let def = self.def();
    ui.label(&def.name)
  }
}

/// Define some generic helper methods for Nodes.
///
/// Boxed trait objects can't have generic methods.
pub trait NodeInterface: NodeImpl {
  fn get_input<I: Into<InputKey>>(&self, idx: I) -> Result<Input>;
  fn set_input<I: Into<InputKey>>(&mut self, idx: I, value: Input) -> Result<Option<OutputId>>;
}

impl dyn NodeImpl {
  pub fn get_input<I: Into<InputKey>>(&self, idx: I) -> Result<Input> {
    self.get_node_input(&idx.into())
  }

  pub fn set_input<I: Into<InputKey>>(&mut self, idx: I, value: Input) -> Result<Option<OutputId>> {
    self.set_node_input(&idx.into(), value)
  }
}

impl<T: ?Sized + NodeImpl> NodeInterface for T {
  fn get_input<I: Into<InputKey>>(&self, idx: I) -> Result<Input> {
    self.get_node_input(&idx.into())
  }

  fn set_input<I: Into<InputKey>>(&mut self, idx: I, value: Input) -> Result<Option<OutputId>> {
    self.set_node_input(&idx.into(), value)
  }
}

impl Clone for Box<dyn NodeImpl> {
  fn clone(&self) -> Self {
    self.clone_node()
  }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EditOnlyNode {
  pub def: NodeDefinition,
  pub inputs: HashMap<String, Input>,
  pub params: HashMap<String, ParameterValue>,
}

impl EditOnlyNode {
  pub fn new(def: NodeDefinition) -> Self {
    Self {
      inputs: def
        .inputs
        .iter()
        .map(|(name, input)| (name.clone(), input.default_value().into()))
        .collect(),
      params: def
        .parameters
        .iter()
        .map(|(name, param)| (name.clone(), param.default_value()))
        .collect(),
      def,
    }
  }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl NodeImpl for EditOnlyNode {
  fn clone_node(&self) -> Box<dyn NodeImpl> {
    Box::new(self.clone())
  }

  fn def(&self) -> &NodeDefinition {
    &self.def
  }

  fn get_node_input(&self, idx: &InputKey) -> Result<Input> {
    let input = self
      .def()
      .get_input(idx)
      .and_then(|input| self.inputs.get(&input.field_name));
    match input {
      Some(input) => Ok(input.clone()),
      _ => Err(anyhow::anyhow!("Unknown input: {idx:?}")),
    }
  }

  fn set_node_input(&mut self, idx: &InputKey, value: Input) -> Result<Option<OutputId>> {
    match self.def().get_input(idx) {
      Some(input) => {
        input.validate(&value)?;
        match self.inputs.insert(input.field_name.clone(), value) {
          Some(Input::Connect(id)) => Ok(Some(id)),
          _ => Ok(None),
        }
      }
      _ => Err(anyhow::anyhow!("Unknown input: {idx:?}")),
    }
  }

  fn get_param(&self, name: &str) -> Result<ParameterValue> {
    match self.params.get(name) {
      Some(param) => Ok(param.clone()),
      _ => Err(anyhow::anyhow!("Unknown parameter: {name}")),
    }
  }

  fn set_param(&mut self, name: &str, value: ParameterValue) -> Result<()> {
    match self.def().get_parameter(name) {
      Some(param) => {
        param.validate(&value)?;
        self.params.insert(param.field_name.clone(), value);
        Ok(())
      }
      _ => Err(anyhow::anyhow!("Unknown param: {name}")),
    }
  }
}

pub trait GetNodeDefinition {
  fn node_definition() -> NodeDefinition;
}

#[cfg_attr(feature = "serde", typetag::serde())]
pub trait NodeBuilder: fmt::Debug + Send + Sync + 'static {
  fn new_node(&self, def: &NodeDefinition) -> Box<dyn NodeImpl>;
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefaultNodeBuilder;

#[cfg_attr(feature = "serde", typetag::serde)]
impl NodeBuilder for DefaultNodeBuilder {
  fn new_node(&self, def: &NodeDefinition) -> Box<dyn NodeImpl> {
    Box::new(EditOnlyNode::new(def.clone()))
  }
}

impl Default for Box<dyn NodeBuilder> {
  fn default() -> Self {
    Box::new(DefaultNodeBuilder)
  }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeDefinition {
  pub name: String,
  pub description: String,
  pub categories: Vec<String>,
  pub uuid: Uuid,
  pub parameters: IndexMap<String, ParameterDefinition>,
  pub inputs: IndexMap<String, InputDefinition>,
  pub outputs: IndexMap<String, OutputDefinition>,
  pub custom: IndexMap<String, String>,
  pub builder: Arc<Box<dyn NodeBuilder>>,
}

impl NodeDefinition {
  pub fn new_node(&self) -> Box<dyn NodeImpl> {
    self.builder.new_node(self)
  }

  pub fn parameters(&self) -> impl Iterator<Item = (&String, &ParameterDefinition)> {
    self.parameters.iter()
  }

  pub fn inputs(&self) -> impl Iterator<Item = (&String, &InputDefinition)> {
    self.inputs.iter()
  }

  pub fn outputs(&self) -> impl Iterator<Item = (&String, &OutputDefinition)> {
    self.outputs.iter()
  }

  pub fn get_input_idx(&self, idx: &InputKey) -> Result<u32> {
    match idx {
      InputKey::Idx(idx) => Ok(*idx),
      InputKey::Name(name) => {
        let idx = self
          .inputs
          .get_index_of(name)
          .ok_or_else(|| anyhow::anyhow!("Invalid input: {name}"))?;
        Ok(idx as _)
      }
    }
  }

  pub fn get_input(&self, idx: &InputKey) -> Option<&InputDefinition> {
    match idx {
      InputKey::Idx(idx) => self.inputs.get_index(*idx as _).map(|(_, v)| v),
      InputKey::Name(name) => self.inputs.get(name),
    }
  }

  pub fn get_parameter(&self, name: &str) -> Option<&ParameterDefinition> {
    self.parameters.get(name)
  }

  pub fn get_output(&self, name: &str) -> Option<&OutputDefinition> {
    self.outputs.get(name)
  }
}

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

  fn new_by_name(&self, name: &str) -> Option<Box<dyn NodeImpl>> {
    self.name_to_id.get(name).and_then(|&id| self.new_by_id(id))
  }

  fn new_by_id(&self, id: Uuid) -> Option<Box<dyn NodeImpl>> {
    self.nodes.get(&id).map(|def| def.new_node())
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

  pub fn new_by_id(&self, id: Uuid) -> Option<Box<dyn NodeImpl>> {
    let inner = self.0.read().unwrap();
    inner.new_by_id(id)
  }

  pub fn new_by_name(&self, name: &str) -> Option<Box<dyn NodeImpl>> {
    let inner = self.0.read().unwrap();
    inner.new_by_name(name)
  }
}

#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeGraph {
  nodes: SlotMap<NodeId, Box<dyn NodeImpl>>,
  connections: HashMap<InputId, OutputId>,
  output: Option<NodeId>,
}

impl NodeGraph {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add(&mut self, node: Box<dyn NodeImpl>) -> NodeId {
    self.nodes.insert(node)
  }

  pub fn remove(&mut self, id: NodeId) -> Option<Box<dyn NodeImpl>> {
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

  pub fn get(&self, id: NodeId) -> Result<&dyn NodeImpl> {
    self
      .nodes
      .get(id)
      .map(|n| n.as_ref())
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))
  }

  pub fn get_mut(&mut self, id: NodeId) -> Result<&mut Box<dyn NodeImpl>> {
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
