use core::fmt;
use std::collections::HashMap;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use serde::{de::Deserializer, ser::SerializeMap, Deserialize, Serialize, Serializer};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

pub type NodeId = Uuid;

pub const NAMESPACE_NODE_IMPL: Uuid = uuid::uuid!("9dee91a8-5af8-11ee-948b-5364d73b1803");

pub fn node_idx(id: NodeId) -> u64 {
  id.as_u64_pair().0
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct InputId {
  pub node: NodeId,
  pub idx: u32,
}

impl InputId {
  pub fn new(node: NodeId, idx: u32) -> Self {
    Self { node, idx }
  }

  pub fn node(&self) -> NodeId {
    self.node
  }

  pub fn key(&self) -> InputKey {
    self.idx.into()
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct OutputId {
  pub node: NodeId,
  pub idx: u32,
}

impl OutputId {
  pub fn new(node: NodeId, idx: u32) -> Self {
    Self { node, idx }
  }

  pub fn node(&self) -> NodeId {
    self.node
  }
}

pub trait NodeImpl: fmt::Debug + erased_serde::Serialize {
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
  fn ui(&mut self, ui: &mut egui::Ui, id: NodeId) {
    let def = self.def();
    let input_count = def.inputs.len();
    let output_count = def.outputs.len();
    let node_style = ui.node_style();
    let zoom = node_style.zoom;
    ui.horizontal(|ui| {
      if input_count > 0 {
        ui.vertical(|ui| {
          self.inputs_ui(ui, id);
        });
      }
      if output_count > 0 {
        ui.vertical(|ui| {
          ui.set_min_width(50.0 * zoom);
          self.outputs_ui(ui, id);
        });
      }
    });
  }

  #[cfg(feature = "egui")]
  fn inputs_ui(&mut self, ui: &mut egui::Ui, id: NodeId) {
    let mut input_changed = None;
    for (idx, name) in self.def().inputs.keys().enumerate() {
      let idx = idx as u32;
      ui.horizontal(|ui| {
        let input_key = InputKey::from(idx);
        let (connected, value) = match self.get_node_input(&input_key) {
          Ok(Input::Value(val)) => (false, Some(val)),
          Ok(Input::Connect(_)) => (true, None),
          Ok(Input::Disconnect) => (false, None),
          Err(err) => {
            ui.label(format!("Invalid input: {err:?}"));
            return;
          }
        };
        let input_id = NodeSocketId::input(0, id, idx);
        ui.add(NodeSocket::new(input_id, connected));
        ui.label(format!("{}", name));
        if let Some(mut value) = value {
          if value.ui(ui).changed() {
            input_changed = Some((input_key, value));
          }
        }
      });
    }
    if let Some((input_key, value)) = input_changed {
      if let Err(err) = self.set_node_input(&input_key, value.into()) {
        log::error!("Failed to update node input: {err:?}");
      }
    }
  }

  #[cfg(feature = "egui")]
  fn outputs_ui(&mut self, ui: &mut egui::Ui, id: NodeId) {
    let def = self.def();
    for (idx, name) in def.outputs.keys().enumerate() {
      ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          let connected = false;
          let output_id = NodeSocketId::output(0, id, idx as _);
          ui.add(NodeSocket::new(output_id, connected));
          ui.label(format!("{}", name));
        });
      });
    }
  }
}

struct NodeData<'a, N: ?Sized>(&'a N);

impl<'a, N> Serialize for NodeData<'a, N>
where
  N: ?Sized + erased_serde::Serialize + 'a,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    erased_serde::serialize(self.0, serializer)
  }
}

impl Serialize for dyn NodeImpl {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut ser = serializer.serialize_map(Some(1))?;
    let node_type = self.def().uuid;
    ser.serialize_entry(&node_type, &NodeData(self))?;
    ser.end()
  }
}

impl<'de> Deserialize<'de> for Box<dyn NodeImpl> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let map = HashMap::<Uuid, serde_json::Value>::deserialize(deserializer)?;
    let (id, node) = map
      .into_iter()
      .next()
      .ok_or_else(|| serde::de::Error::custom("Node missing implementation data"))?;
    NODE_REGISTRY
      .load_node(id, node)
      .map_err(serde::de::Error::custom)
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeState {
  pub(crate) id: NodeId,
  name: String,
  node: Box<dyn NodeImpl>,
  position: emath::Vec2,
  size: emath::Vec2,
  #[serde(skip)]
  updated: bool,
  selected: bool,
}

impl NodeState {
  pub fn new(def: &NodeDefinition) -> Result<Self> {
    Ok(Self {
      id: Uuid::new_v4(),
      name: def.name.clone(),
      node: def.new_node()?,
      position: [0., 0.].into(),
      size: [10., 10.].into(),
      updated: true,
      selected: false,
    })
  }

  pub fn set_position(&mut self, position: emath::Vec2) {
    self.position = position;
  }

  /// Clone node with a new uuid.
  pub fn duplicate(&self) -> Self {
    let mut node = self.clone();
    node.id = Uuid::new_v4();
    node
  }

  pub fn eval(
    &self,
    graph: &NodeGraph,
    execution: &mut NodeGraphExecution,
    id: NodeId,
  ) -> Result<Value> {
    self.node.eval(graph, execution, id)
  }

  pub fn compile(
    &self,
    graph: &NodeGraph,
    compile: &mut NodeGraphCompile,
    id: NodeId,
  ) -> Result<()> {
    self.node.compile(graph, compile, id)
  }

  pub fn cache_output(&self) -> bool {
    self.node.cache_output()
  }

  pub fn get_input_idx(&self, idx: &InputKey) -> Result<u32> {
    self.node.get_input_idx(idx)
  }

  pub fn get_input<I: Into<InputKey>>(&self, idx: I) -> Result<Input> {
    self.node.get_node_input(&idx.into())
  }

  pub fn set_input<I: Into<InputKey>>(&mut self, idx: I, value: Input) -> Result<Option<OutputId>> {
    self.updated = true;
    self.node.set_node_input(&idx.into(), value)
  }
}

#[cfg(feature = "egui")]
impl NodeState {
  fn get_zoomed(&self, zoom: f32) -> (emath::Vec2, emath::Vec2) {
    let mut position = self.position;
    let mut size = self.size;
    position.zoom(zoom);
    size.zoom(zoom);
    (position, size)
  }

  pub fn ui_at(&mut self, ui: &mut egui::Ui, offset: egui::Vec2) -> egui::Response {
    let node_style = ui.node_style();
    let zoom = node_style.zoom;
    // Apply zoom to node position and size.
    let (position, size) = self.get_zoomed(zoom);
    // Need to translate node to Screen space.
    let pos = position + offset;
    let rect = egui::Rect::from_min_size(pos.to_pos2(), size);

    // Dragged or clicked.
    let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
    if resp.dragged() {
      self.position = (position + resp.drag_delta()) / zoom;
      resp.scroll_to_me(None);
    }
    if resp.clicked() {
      self.selected = !self.selected;
    }

    // Only render this node if it is visible or the node was updated.
    if !self.updated && !ui.is_rect_visible(rect) {
      // This is needed to stabilize Ui ids when nodes become visible.
      ui.skip_ahead_auto_ids(1);
      return resp;
    }
    self.updated = false;

    let mut child_ui = ui.child_ui_with_id_source(rect, *ui.layout(), self.id);
    self.frame_ui(&mut child_ui, node_style);

    // Update node size.
    let rect = child_ui.min_rect();
    self.size = rect.size() / zoom;
    resp
  }

  fn frame_ui(&mut self, ui: &mut egui::Ui, node_style: NodeStyle) {
    // Window-style frame.
    let style = ui.style();
    let mut frame = egui::Frame::window(style);
    frame.shadow = Default::default();
    if self.selected {
      frame.stroke.color = egui::Color32::WHITE;
    }

    frame.fill(egui::Color32::from_gray(50)).show(ui, |ui| {
      ui.vertical(|ui| {
        // Title bar.
        ui.horizontal(|ui| {
          ui.label(&self.name);
        });
        // Contents
        egui::Frame::none()
          .fill(egui::Color32::from_gray(63))
          .show(ui, |ui| {
            ui.set_min_width(node_style.node_min_size.x);
            self.node.ui(ui, self.id);
          });
      });
    });
  }
}
