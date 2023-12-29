use core::fmt;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use serde::{de::Deserializer, Deserialize, Serialize, Serializer};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

pub type NodeId = Uuid;

pub const NAMESPACE_NODE_IMPL: Uuid = uuid::uuid!("9dee91a8-5af8-11ee-948b-5364d73b1803");

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
    let param_count = def.parameters.len();
    let node_style = NodeStyle::get(ui);
    let zoom = node_style.zoom;
    ui.vertical(|ui| {
      ui.horizontal(|ui| {
        if input_count > 0 {
          ui.vertical(|ui| {
            self.inputs_ui(ui, id);
          });
        }
        if output_count > 0 {
          if input_count > 0 {
            ui.separator();
          }
          ui.vertical(|ui| {
            ui.set_min_width(50.0 * zoom);
            self.outputs_ui(ui, id);
          });
        }
      });
      if param_count > 0 {
        ui.separator();
        self.parameters_ui(ui, id);
      }
    });
  }

  #[cfg(feature = "egui")]
  fn inputs_ui(&mut self, ui: &mut egui::Ui, id: NodeId) {
    let mut input_changed = None;
    for (idx, (name, def)) in self.def().inputs.iter().enumerate() {
      ui.horizontal(|ui| {
        let input_key = InputKey::from(idx as u32);
        let (connected, value) = match self.get_node_input(&input_key) {
          Ok(Input::Value(val)) => (false, Some(val)),
          Ok(Input::Connect(_, _)) => (true, None),
          Ok(Input::Disconnect) => (false, None),
          Err(err) => {
            ui.label(format!("Invalid input: {err:?}"));
            return;
          }
        };
        ui.add(NodeSocket::input(id, idx, connected, def));
        if connected {
          ui.label(name);
        } else {
          ui.collapsing(name, |ui| {
            if let Some(mut value) = value {
              if value.ui(ui) {
                input_changed = Some((input_key, value));
              }
            }
          });
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
  fn parameters_ui(&mut self, ui: &mut egui::Ui, _id: NodeId) {
    let mut parameter_changed = None;
    for (name, def) in &self.def().parameters {
      ui.horizontal(|ui| {
        let mut value = match self.get_param(name) {
          Ok(val) => val,
          Err(err) => {
            ui.label(format!("Invalid parameter: {err:?}"));
            return;
          }
        };
        ui.label(name);
        if def.ui(ui, &mut value) {
          parameter_changed = Some((name.to_string(), value));
        }
      });
    }
    if let Some((name, value)) = parameter_changed {
      if let Err(err) = self.set_param(&name, value.into()) {
        log::error!("Failed to update node parameter: {err:?}");
      }
    }
  }

  #[cfg(feature = "egui")]
  fn outputs_ui(&mut self, ui: &mut egui::Ui, id: NodeId) {
    for (idx, (name, def)) in self.def().outputs.iter().enumerate() {
      ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          ui.add(NodeSocket::output(id, idx, def));
          ui.label(name);
        });
      });
    }
  }
}

impl Serialize for dyn NodeImpl {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    erased_serde::serialize(self, serializer)
  }
}

/// Define some generic helper methods for Nodes.
///
/// Boxed trait objects can't have generic methods.
pub trait NodeInterface: NodeImpl {
  fn get_input<I: Into<InputKey>>(&self, idx: I) -> Result<Input>;
  fn set_input<I: Into<InputKey>>(&mut self, idx: I, value: Input) -> Result<Option<OutputId>>;
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

#[derive(serde::Deserialize)]
pub struct LoadNodeState {
  pub id: NodeId,
  pub group_id: NodeGroupId,
  pub name: String,
  pub node_type: Uuid,
  pub node: serde_json::Value,
  pub area: emath::Rect,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Node {
  pub id: NodeId,
  pub group_id: NodeGroupId,
  pub name: String,
  node_type: Uuid,
  node: Box<dyn NodeImpl>,
  pub area: emath::Rect,
  #[serde(skip)]
  updated: bool,
}

impl GetId for Node {
  fn id(&self) -> Uuid {
    self.id
  }
}

impl<'de> Deserialize<'de> for Node {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let node = LoadNodeState::deserialize(deserializer)?;
    NODE_REGISTRY
      .load_node(node)
      .map_err(serde::de::Error::custom)
  }
}

impl Node {
  pub fn new(def: &NodeDefinition) -> Result<Self> {
    Ok(Self {
      id: Uuid::new_v4(),
      group_id: Uuid::nil(),
      name: def.name.clone(),
      node_type: def.id,
      node: def.new_node()?,
      area: emath::Rect::from_min_size([0., 0.].into(), [10., 10.].into()),
      updated: true,
    })
  }

  pub fn load(def: &NodeDefinition, data: LoadNodeState) -> Result<Self> {
    Ok(Self {
      id: data.id,
      group_id: data.group_id,
      name: data.name,
      node_type: data.node_type,
      node: def.load_node(data.node)?,
      area: data.area,
      updated: true,
    })
  }

  pub fn set_position(&mut self, position: emath::Vec2) {
    self.area = emath::Rect::from_min_size(position.to_pos2(), self.area.size());
  }

  pub(crate) fn new_id(&mut self) {
    self.id = Uuid::new_v4();
  }

  /// Clone node with a new uuid.
  pub fn duplicate(&self) -> Self {
    let mut node = self.clone();
    node.new_id();
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
impl NodeFrame for Node {
  fn title(&self) -> &str {
    &self.name
  }

  fn set_title(&mut self, title: String) {
    self.name = title;
  }

  fn take_updated(&mut self, state: &mut NodeFrameState) -> bool {
    let updated = self.updated | state.take_updated();
    self.updated = false;
    updated
  }

  fn rect(&self) -> emath::Rect {
    self.area
  }

  fn set_rect(&mut self, rect: emath::Rect) {
    self.area = rect;
  }

  fn auto_size(&self) -> bool {
    true
  }

  fn resizable(&self) -> bool {
    false
  }

  fn contents_ui(&mut self, ui: &mut egui::Ui, node_style: NodeStyle) {
    egui::Frame::none()
      .fill(egui::Color32::from_gray(63))
      .show(ui, |ui| {
        ui.set_min_width(node_style.node_min_size.x);
        self.node.ui(ui, self.id);
      });
  }

  /// Handle events and context menu.
  fn handle_resp(
    &mut self,
    _ui: &mut egui::Ui,
    resp: egui::Response,
    _graph: &NodeGraphMeta,
    frame: &mut NodeFrameState,
  ) -> Option<NodeAction> {
    let mut action = None;
    if resp.dragged() {
      if frame.is_dragging() {
        action = Some(NodeAction::Dragged(resp.drag_delta()));
      } else {
        action = Some(NodeAction::Resize);
      }
    }
    resp.context_menu(|ui| {
      if ui.button("Delete").clicked() {
        action = Some(NodeAction::Delete(false));
        ui.close_menu();
      }
      if !self.group_id.is_nil() {
        if ui.button("Remove from group").clicked() {
          action = Some(NodeAction::LeaveGroup(self.group_id));
          self.group_id = Uuid::nil();
          ui.close_menu();
        }
      }
    });
    action
  }
}
