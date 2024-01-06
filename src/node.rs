use core::fmt;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use serde::{de::Deserializer, Deserialize, Serialize, Serializer};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

pub type NodeId = Uuid;

pub const NAMESPACE_NODE_IMPL: Uuid = uuid::uuid!("9dee91a8-5af8-11ee-948b-5364d73b1803");

/// This is used to resolve Dynamic Vector/Matrix inputs/outputs.
#[derive(Copy, Clone, Debug, Default)]
pub struct NodeConcreteType {
  /// The minimum size of connected Vector/Matrix.  Scalars are not counted.
  pub min: Option<DynamicSize>,
  pub scalars: usize,
  pub vectors: usize,
  pub matrixes: usize,
}

impl NodeConcreteType {
  pub fn has_dynamic(&self) -> bool {
    let count = self.scalars + self.vectors + self.matrixes;
    count > 0
  }

  pub fn data_type(&self) -> Option<DataType> {
    let count = self.scalars + self.vectors + self.matrixes;
    if count > 0 {
      match (self.min, self.scalars, self.vectors, self.matrixes) {
        (None | Some(DynamicSize::D1), s, 0, 0) if s > 0 => Some(DataType::F32),
        (Some(DynamicSize::D2), _, 0, m) if m > 0 => Some(DataType::Mat2),
        (Some(DynamicSize::D3), _, 0, m) if m > 0 => Some(DataType::Mat3),
        (Some(DynamicSize::D4), _, 0, m) if m > 0 => Some(DataType::Mat4),
        (Some(DynamicSize::D2), _, v, _) if v > 0 => Some(DataType::Vec2),
        (Some(DynamicSize::D3), _, v, _) if v > 0 => Some(DataType::Vec3),
        (Some(DynamicSize::D4), _, v, _) if v > 0 => Some(DataType::Vec4),
        _ => None,
      }
    } else {
      None
    }
  }

  pub fn convert(&self, value: &mut CompiledValue) -> Result<()> {
    if let Some(min) = self.min {
      let (vec_dt, mat_dt) = match min {
        DynamicSize::D3 => (DataType::Vec3, DataType::Mat3),
        DynamicSize::D4 => (DataType::Vec4, DataType::Mat4),
        _ => (DataType::Vec2, DataType::Mat2),
      };
      match value.dt.class() {
        DataTypeClass::Scalar => value.convert(vec_dt),
        DataTypeClass::Vector => value.convert(vec_dt),
        DataTypeClass::Matrix => value.convert(mat_dt),
        class => Err(anyhow!(
          "Unsupported data type conversion: class={class:?}, dt={:?}",
          value.dt
        )),
      }
    } else {
      Ok(())
    }
  }

  pub fn add_input_type(&mut self, dt: DataType) {
    let min = self.min.unwrap_or(DynamicSize::D4).len();
    match dt {
      DataType::I32 | DataType::U32 | DataType::F32 => {
        self.scalars += 1;
        // Don't update the `min` for Scalars.
      }
      DataType::Vec2 => {
        self.vectors += 1;
        // D2 is the smallest.
        self.min = Some(DynamicSize::D2);
      }
      DataType::Vec3 => {
        self.vectors += 1;
        if min > 3 {
          self.min = Some(DynamicSize::D3);
        }
      }
      DataType::Vec4 => {
        self.vectors += 1;
        if min > 4 {
          self.min = Some(DynamicSize::D4);
        }
      }
      DataType::Mat2 => {
        self.matrixes += 1;
        // D2 is the smallest.
        self.min = Some(DynamicSize::D2);
      }
      DataType::Mat3 => {
        self.matrixes += 1;
        if min > 3 {
          self.min = Some(DynamicSize::D3);
        }
      }
      DataType::Mat4 => {
        self.matrixes += 1;
        if min > 4 {
          self.min = Some(DynamicSize::D4);
        }
      }
      _ => (),
    }
  }
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

impl From<NodeId> for OutputId {
  fn from(node: NodeId) -> Self {
    Self { node, idx: 0 }
  }
}

pub trait NodeImpl: fmt::Debug + erased_serde::Serialize + Send + Sync {
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
  fn details_ui(&mut self, ui: &mut egui::Ui, id: NodeId) -> bool {
    self.ui(ui, id, true)
  }

  #[cfg(feature = "egui")]
  fn node_ui(&mut self, ui: &mut egui::Ui, id: NodeId) -> bool {
    self.ui(ui, id, false)
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui, id: NodeId, details: bool) -> bool {
    let def = self.def();
    let input_count = def.inputs.len();
    let param_count = def.parameters.len();
    let node_style = NodeStyle::get(ui);
    let zoom = node_style.zoom;
    let mut concrete_type = NodeConcreteType::default();
    let mut updated = false;
    if details {
      if self.inputs_ui(&mut concrete_type, ui, id, details) {
        updated = true;
      }
      if param_count > 0 {
        if input_count > 0 {
          ui.separator();
        }
        if self.parameters_ui(&mut concrete_type, ui, id, details) {
          updated = true;
        }
      }
    } else {
      let output_count = def.outputs.len();
      ui.vertical(|ui| {
        ui.horizontal(|ui| {
          if input_count > 0 {
            ui.vertical(|ui| {
              if self.inputs_ui(&mut concrete_type, ui, id, details) {
                updated = true;
              }
            });
          }
          if output_count > 0 {
            if input_count > 0 {
              ui.separator();
            }
            ui.vertical(|ui| {
              ui.set_min_width(50.0 * zoom);
              if self.outputs_ui(&mut concrete_type, ui, id, details) {
                updated = true;
              }
            });
          }
        });
        if param_count > 0 {
          ui.separator();
          if self.parameters_ui(&mut concrete_type, ui, id, details) {
            updated = true;
          }
        }
      });
    }
    updated
  }

  #[cfg(feature = "egui")]
  fn inputs_ui(
    &mut self,
    concrete_type: &mut NodeConcreteType,
    ui: &mut egui::Ui,
    id: NodeId,
    details: bool,
  ) -> bool;

  #[cfg(feature = "egui")]
  fn parameters_ui(
    &mut self,
    concrete_type: &mut NodeConcreteType,
    ui: &mut egui::Ui,
    _id: NodeId,
    details: bool,
  ) -> bool;

  #[cfg(feature = "egui")]
  fn outputs_ui(
    &mut self,
    concrete_type: &mut NodeConcreteType,
    ui: &mut egui::Ui,
    id: NodeId,
    details: bool,
  ) -> bool;
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
  pub updated: bool,
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

  #[cfg(feature = "egui")]
  pub fn details_ui(&mut self, ui: &mut egui::Ui, id: NodeId) -> bool {
    self.node.details_ui(ui, id)
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
        if self.node.node_ui(ui, self.id) {
          self.updated = true;
        }
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
    if resp.clicked() {
      action = Some(NodeAction::Clicked);
    } else if resp.dragged() {
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
