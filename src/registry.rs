use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use serde::{Deserialize, Serialize};

use crate::*;

lazy_static::lazy_static! {
  pub static ref NODE_REGISTRY: NodeRegistry = {
    let registry = NodeRegistry::new();
    for reg in inventory::iter::<RegisterNode> {
      let def = (reg.get_def)();
      if let Some(prev) = registry.register(&def) {
        log::error!(
          "Node {:?} re-defined at {}, prev definition at: {}",
          def.name,
          def.source_file,
          prev.source_file
        );
      }
    }
    registry
  };
}

#[derive(Clone, Default, Debug)]
pub struct NodeFilter {
  pub name: String,
}

impl NodeFilter {
  #[cfg(feature = "egui")]
  pub fn ui(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("🔍 Search"))
        .request_focus();
    });
  }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct NodeRegistryInner {
  nodes: HashMap<Uuid, NodeDefinition>,
  name_to_id: HashMap<String, Uuid>,
}

impl NodeRegistryInner {
  pub fn nodes(&self) -> Vec<NodeDefinition> {
    self.nodes.values().cloned().collect()
  }

  fn register(&mut self, def: &NodeDefinition) -> Option<NodeDefinition> {
    self.name_to_id.insert(def.name.clone(), def.uuid);
    self.nodes.insert(def.uuid, def.clone())
  }

  pub fn load_node(&self, data: LoadNodeState) -> Result<Node> {
    let id = data.node_type;
    let def = self
      .nodes
      .get(&id)
      .ok_or_else(|| anyhow!("Missing Node definition: {id:?}"))?;
    Node::load(def, data)
  }

  fn new_by_name(&self, name: &str) -> Result<Node> {
    let id = self
      .name_to_id
      .get(name)
      .ok_or_else(|| anyhow!("Missing Node definition: {name:?}"))?;
    self.new_by_id(*id)
  }

  fn new_by_id(&self, id: Uuid) -> Result<Node> {
    let def = self
      .nodes
      .get(&id)
      .ok_or_else(|| anyhow!("Missing Node definition: {id:?}"))?;
    Node::new(def)
  }

  #[cfg(feature = "egui")]
  pub fn ui(&self, ui: &mut egui::Ui, filter: &NodeFilter) -> Option<Node> {
    let mut selected_node = None;
    ui.group(|ui| {
      for def in self.nodes.values() {
        if def.matches(filter) {
          if ui.button(&def.name).clicked() {
            selected_node = Some(Node::new(def).expect("Node building shouldn't fail"));
          }
        }
      }
    });
    selected_node
  }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeRegistry(Arc<RwLock<NodeRegistryInner>>);

impl NodeRegistry {
  /// An empty node registry.
  pub fn new() -> Self {
    Default::default()
  }

  /// Build node registry from all node definitions.
  pub fn build() -> Self {
    NODE_REGISTRY.clone()
  }

  pub fn nodes(&self) -> Vec<NodeDefinition> {
    let inner = self.0.read().unwrap();
    inner.nodes()
  }

  pub fn register(&self, def: &NodeDefinition) -> Option<NodeDefinition> {
    let mut inner = self.0.write().unwrap();
    inner.register(def)
  }

  pub fn load_node(&self, data: LoadNodeState) -> Result<Node> {
    let inner = self.0.read().unwrap();
    inner.load_node(data)
  }

  pub fn new_by_id(&self, id: Uuid) -> Result<Node> {
    let inner = self.0.read().unwrap();
    inner.new_by_id(id)
  }

  pub fn new_by_name(&self, name: &str) -> Result<Node> {
    let inner = self.0.read().unwrap();
    inner.new_by_name(name)
  }

  #[cfg(feature = "egui")]
  pub fn ui(&self, ui: &mut egui::Ui, filter: &NodeFilter) -> Option<Node> {
    let inner = self.0.write().unwrap();
    inner.ui(ui, filter)
  }
}

#[derive(Clone, Debug)]
pub struct RegisterNode {
  pub get_def: fn() -> NodeDefinition,
}
inventory::collect!(RegisterNode);

impl RegisterNode {
  pub const fn new(get_def: fn() -> NodeDefinition) -> Self {
    Self { get_def }
  }
}

#[macro_export]
macro_rules! register_node {
  ($($definition:tt)*) => {
    inventory::submit! {
      $crate::RegisterNode::new(|| {
        $($definition)*
      })
    }
  };
}

pub trait NodeBuilder: Send + Sync {
  fn new_node(
    &self,
    def: &NodeDefinition,
    data: Option<serde_json::Value>,
  ) -> Result<Box<dyn NodeImpl>>;
}

impl fmt::Debug for Box<dyn NodeBuilder> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    f.debug_tuple("NodeBuilder").finish()
  }
}

impl Default for Box<dyn NodeBuilder> {
  fn default() -> Self {
    Box::new(NodeBuilderFn(|def, _| {
      Err(anyhow!("Missing code for Node definition: {def:?}"))
    }))
  }
}

pub struct NodeBuilderFn(
  fn(&NodeDefinition, Option<serde_json::Value>) -> Result<Box<dyn NodeImpl>>,
);

impl NodeBuilder for NodeBuilderFn {
  fn new_node(
    &self,
    def: &NodeDefinition,
    data: Option<serde_json::Value>,
  ) -> Result<Box<dyn NodeImpl>> {
    (self.0)(def, data)
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct NodeDefinition {
  pub name: String,
  pub description: String,
  pub categories: Vec<String>,
  pub uuid: Uuid,
  pub parameters: IndexMap<String, ParameterDefinition>,
  pub inputs: IndexMap<String, InputDefinition>,
  pub outputs: IndexMap<String, OutputDefinition>,
  pub custom: IndexMap<String, String>,
  #[serde(skip)]
  pub source_file: String,
  #[serde(skip)]
  pub builder: Arc<Box<dyn NodeBuilder>>,
}

impl NodeDefinition {
  pub fn new(
    name: &str,
    create: fn(&NodeDefinition, Option<serde_json::Value>) -> Result<Box<dyn NodeImpl>>,
  ) -> Self {
    Self {
      name: name.to_string(),
      uuid: uuid::Uuid::new_v5(&NAMESPACE_NODE_IMPL, name.as_bytes()),
      builder: Arc::new(Box::new(NodeBuilderFn(create))),
      ..Default::default()
    }
  }

  pub fn matches(&self, filter: &NodeFilter) -> bool {
    self
      .name
      .to_lowercase()
      .contains(&filter.name.to_lowercase())
  }

  pub fn new_node(&self) -> Result<Box<dyn NodeImpl>> {
    self.builder.new_node(self, None)
  }

  pub fn load_node(&self, data: serde_json::Value) -> Result<Box<dyn NodeImpl>> {
    self.builder.new_node(self, Some(data))
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
          .ok_or_else(|| anyhow!("Invalid input: {name}"))?;
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
