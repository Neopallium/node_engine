use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use egui::{self, NumExt};

use crate::node::{InputId, NodeId, OutputId};
use crate::values::DataType;
use crate::{InputDefinition, OutputDefinition};

mod frame;
mod zoom;
pub use frame::*;
pub use zoom::*;

const NODE_STYLE: &'static str = "NodeStyle";
const NODE_GRAPH_META: &'static str = "NodeGraphMeta";

#[derive(Clone, Debug)]
pub struct NodeStyle {
  pub node_min_size: egui::Vec2,
  pub line_stroke: egui::Stroke,
  pub zoom: f32,
}

impl Default for NodeStyle {
  fn default() -> Self {
    Self {
      node_min_size: (200.0, 10.0).into(),
      line_stroke: (3.0, egui::Color32::WHITE).into(),
      zoom: 1.0,
    }
  }
}

impl Zoom for NodeStyle {
  #[inline(always)]
  fn zoom(&mut self, zoom: f32) {
    self.zoom *= zoom;
    self.node_min_size.zoom(zoom);
    self.line_stroke.zoom(zoom);
  }
}

impl NodeStyle {
  pub fn get(ui: &mut egui::Ui) -> Self {
    ui.data(|d| d.get_temp::<Self>(egui::Id::new(NODE_STYLE)))
      .unwrap_or_default()
  }

  pub fn set(&self, ui: &mut egui::Ui) {
    ui.data_mut(|d| d.insert_temp(egui::Id::new(NODE_STYLE), self.clone()));
  }

  pub fn zoom_style(ui: &mut egui::Ui, zoom: f32) -> Self {
    ui.style_mut().zoom(zoom);
    ui.data_mut(|d| {
      let node_style = d.get_temp_mut_or_default::<Self>(egui::Id::new(NODE_STYLE));
      node_style.zoom(zoom);
      node_style.clone()
    })
  }
}

#[derive(Clone, Debug, Default)]
pub struct NodeSocketDragState {
  pub src: Option<NodeSocket>,
  pub dst: Option<NodeSocket>,
}

impl NodeSocketDragState {
  pub fn get_dropped_node_sockets(&self) -> Option<(NodeSocket, Option<NodeSocket>)> {
    self.src.clone().map(|src| (src, self.dst.clone()))
  }

  pub fn clear_dropped_node_sockets(&mut self) {
    self.src = None;
    self.dst = None;
  }
}

#[derive(Clone, Debug, Default)]
struct NodeGraphMetaInner {
  ui_min: egui::Vec2,
  zoom: f32,
  sockets: HashMap<NodeSocketId, NodeSocket>,
  drag_state: NodeSocketDragState,
}

impl NodeGraphMetaInner {
  pub fn update(&mut self, ui_min: egui::Vec2, zoom: f32) {
    self.ui_min = ui_min;
    self.zoom = zoom;
  }

  pub fn remove_node(&mut self, node_id: NodeId) {
    self.sockets.retain(|id, _| id.node() != node_id);
  }

  /// Convert from UI screen-space to graph-space and unzoom.
  pub fn ui_to_graph(&self, pos: egui::Pos2) -> egui::Vec2 {
    (pos.to_vec2() - self.ui_min) / self.zoom
  }

  pub fn update_node_socket(&mut self, socket: &mut NodeSocket, pos: egui::Pos2) {
    socket.center = self.ui_to_graph(pos);
    self.sockets.insert(socket.id, socket.clone());
  }

  pub fn get_connection_meta(
    &self,
    input: &InputId,
    output: &OutputId,
  ) -> Option<(NodeSocket, NodeSocket)> {
    self.sockets.get(&input.into()).and_then(|in_meta| {
      self
        .sockets
        .get(&output.into())
        .map(|out_meta| (in_meta.clone(), out_meta.clone()))
    })
  }
}

#[derive(Clone, Debug, Default)]
pub struct NodeGraphMeta(Arc<RwLock<NodeGraphMetaInner>>);

impl NodeGraphMeta {
  pub fn get(ui: &egui::Ui) -> Option<Self> {
    ui.data(|d| d.get_temp::<NodeGraphMeta>(egui::Id::new(NODE_GRAPH_META)))
  }

  pub fn load(&self, ui: &mut egui::Ui, ui_min: egui::Vec2, zoom: f32) {
    let mut inner = self.0.write().unwrap();
    inner.update(ui_min, zoom);
    ui.data_mut(|d| {
      d.insert_temp(egui::Id::new(NODE_GRAPH_META), self.clone());
    });
  }

  pub fn unload(&self, ui: &mut egui::Ui) {
    ui.data_mut(|d| {
      d.remove::<NodeGraphMeta>(egui::Id::new(NODE_GRAPH_META));
    });
  }

  pub fn drag_state(&self) -> NodeSocketDragState {
    let inner = self.0.read().unwrap();
    inner.drag_state.clone()
  }

  pub fn set_drag_state(&self, state: NodeSocketDragState) {
    let mut inner = self.0.write().unwrap();
    inner.drag_state = state;
  }

  pub fn get_dropped_node_sockets(&self) -> Option<(NodeSocket, Option<NodeSocket>)> {
    let inner = self.0.read().unwrap();
    inner.drag_state.get_dropped_node_sockets()
  }

  pub fn clear_dropped_node_sockets(&self) {
    let mut inner = self.0.write().unwrap();
    inner.drag_state.clear_dropped_node_sockets();
  }

  pub fn remove_node(&self, node_id: NodeId) {
    let mut inner = self.0.write().unwrap();
    inner.remove_node(node_id);
  }

  /// Convert from UI screen-space to graph-space and unzoom.
  pub fn ui_to_graph(&self, pos: egui::Pos2) -> egui::Vec2 {
    let inner = self.0.read().unwrap();
    inner.ui_to_graph(pos)
  }

  pub fn update_node_socket(&self, socket: &mut NodeSocket, pos: egui::Pos2) {
    let mut inner = self.0.write().unwrap();
    inner.update_node_socket(socket, pos)
  }

  pub fn get_connection_meta(
    &self,
    input: &InputId,
    output: &OutputId,
  ) -> Option<(NodeSocket, NodeSocket)> {
    let inner = self.0.read().unwrap();
    inner.get_connection_meta(input, output)
  }
}

#[derive(Clone, Debug)]
pub struct NodeSocket {
  pub id: NodeSocketId,
  connected: bool,
  pub center: egui::Vec2,
  pub color: egui::Color32,
  pub dt: DataType,
}

impl NodeSocket {
  pub fn input(node: NodeId, idx: usize, connected: bool, def: &InputDefinition) -> Self {
    let id = NodeSocketId::input(node, idx as _);
    Self::new(id, connected, def.value_type, def.color)
  }

  pub fn output(node: NodeId, idx: usize, def: &OutputDefinition) -> Self {
    let id = NodeSocketId::output(node, idx as _);
    Self::new(id, false, def.value_type, def.color)
  }

  pub fn new(
    id: NodeSocketId,
    connected: bool,
    dt: DataType,
    color: Option<egui::Color32>,
  ) -> Self {
    Self {
      id,
      connected,
      center: Default::default(),
      color: color.unwrap_or_else(|| dt.color()),
      dt,
    }
  }

  pub fn is_compatible(&self, dst: &NodeSocket) -> bool {
    self.id.is_compatible(dst.id) && self.dt.is_compatible(&dst.dt)
  }

  pub fn input_id_first(
    &self,
    dst: Option<NodeSocket>,
  ) -> Option<(InputId, Option<(OutputId, DataType)>)> {
    match (self.id, dst.map(|d| (d.id, d.dt))) {
      // Disconect input.
      (NodeSocketId::Input(id), None) => Some((id, None)),
      // Connect input to output.
      (NodeSocketId::Input(input), Some((NodeSocketId::Output(output), dt)))
        if input.node != output.node =>
      {
        Some((input, Some((output, dt))))
      }
      // Connect output to input.
      (NodeSocketId::Output(output), Some((NodeSocketId::Input(input), dt)))
        if input.node != output.node =>
      {
        Some((input, Some((output, dt))))
      }
      // Other non-compatible connections.
      _ => None,
    }
  }
}

impl egui::Widget for NodeSocket {
  fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
    // 1. Deciding widget size:
    let spacing = &ui.spacing();
    let icon_width = spacing.icon_width;
    let desired_size =
      egui::vec2(icon_width, icon_width).at_least(egui::Vec2::splat(spacing.interact_size.y));

    // 2. Allocating space:
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());
    // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
    let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
    let center = small_icon_rect.center();

    let graph = match NodeGraphMeta::get(ui) {
      Some(graph) => graph,
      None => {
        return ui.label("NodeSocket not inside a NodeGraph");
      }
    };
    // Update socket metadata.
    graph.update_node_socket(&mut self, center);

    // Get current socket drag state.
    let mut drag_state = graph.drag_state();

    // 3. Interact: Time to check for clicks!
    if response.drag_started() {
      drag_state.src = Some(self.clone());
    }
    // `hovered()` doesn't work during drag.
    let mut hovered = ui.rect_contains_pointer(rect) || response.hovered();
    if hovered {
      if let Some(src) = &drag_state.src {
        // Check if src socket is compatible.
        if self.is_compatible(src) {
          drag_state.dst = Some(self.clone());
        } else {
          hovered = false;
          drag_state.dst = None;
        }
      }
    } else if let Some(dst) = &drag_state.dst {
      if dst.id == self.id {
        // No longer hovering our socket.  Cleanup.
        drag_state.dst = None;
      }
    }
    graph.set_drag_state(drag_state);
    let selected = hovered || self.connected;

    // We will follow the current style by asking
    // "how should something that is being interacted with be painted?".
    // This will, for instance, give us different colors when the widget is hovered or clicked.
    let style = ui.style();
    let visuals = style.interact_selectable(&response, selected);
    let mut bg_stroke = visuals.bg_stroke;
    bg_stroke.color = self.color;
    if hovered {
      bg_stroke = bg_stroke;
    }

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, selected, ""));

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
      let scale = 0.7;
      let big_radius = (big_icon_rect.width() / 2.0 + visuals.expansion) * scale;
      let small_radius = (small_icon_rect.width() / 2.0) * scale;

      let painter = ui.painter();

      painter.circle(center, big_radius, visuals.bg_fill, bg_stroke);

      if selected {
        painter.circle_filled(center, small_radius, self.color);
      }
    }
    response
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeSocketId {
  Input(InputId),
  Output(OutputId),
}

impl From<&InputId> for NodeSocketId {
  fn from(id: &InputId) -> Self {
    NodeSocketId::Input(*id)
  }
}

impl From<&OutputId> for NodeSocketId {
  fn from(id: &OutputId) -> Self {
    NodeSocketId::Output(*id)
  }
}

impl NodeSocketId {
  pub fn input(node: NodeId, idx: u32) -> Self {
    Self::Input(InputId::new(node, idx))
  }

  pub fn output(node: NodeId, idx: u32) -> Self {
    Self::Output(OutputId::new(node, idx))
  }

  pub fn is_compatible(&self, dst: NodeSocketId) -> bool {
    match (self, &dst) {
      (Self::Input(input), Self::Output(output)) | (Self::Output(output), Self::Input(input)) => {
        input.node != output.node
      }
      _ => false,
    }
  }

  pub fn as_input_id(&self) -> Option<InputId> {
    match self {
      Self::Input(id) => Some(*id),
      _ => None,
    }
  }

  pub fn node(&self) -> NodeId {
    match self {
      Self::Input(id) => id.node(),
      Self::Output(id) => id.node(),
    }
  }
}
