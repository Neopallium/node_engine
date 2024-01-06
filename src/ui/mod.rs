use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use egui::{self, NumExt};

use indexmap::IndexMap;
use uuid::Uuid;

use crate::node::{InputId, NodeId, OutputId};
use crate::values::DataType;
use crate::{GetId, InputDefinition, OutputDefinition};

mod frame;
mod zoom;
pub use frame::*;
pub use zoom::*;

const NODE_STYLE: &'static str = "NodeStyle";
const NODE_GRAPH_META: &'static str = "NodeGraphMeta";

#[derive(Clone, Debug)]
pub struct NodeStyle {
  pub node_min_size: emath::Vec2,
  pub line_stroke: egui::Stroke,
  pub input_to_edge: f32,
  pub output_to_edge: f32,
  pub curve_offset: f32,
  pub zoom: f32,
}

impl Default for NodeStyle {
  fn default() -> Self {
    Self {
      node_min_size: (200.0, 10.0).into(),
      line_stroke: (2.0, egui::Color32::WHITE).into(),
      input_to_edge: -13.0,
      output_to_edge: 17.0,
      curve_offset: 10.0,
      zoom: 1.0,
    }
  }
}

impl Zoom for NodeStyle {
  #[inline(always)]
  fn zoom(&mut self, zoom: f32) {
    self.zoom *= zoom;
    self.curve_offset *= zoom;
    self.input_to_edge *= zoom;
    self.output_to_edge *= zoom;
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

  pub fn unzoom_style(&self, ui: &mut egui::Ui, zoom: f32) {
    ui.style_mut().zoom(1.0 / zoom);
    self.set(ui);
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

#[derive(Clone, Debug)]
pub struct NodeConnection {
  pub ui_min: emath::Vec2,
  pub zoom: f32,
  pub line_stroke: egui::Stroke,
  pub curve_offset: f32,
}

impl NodeConnection {
  pub fn new(style: &NodeStyle, ui_min: emath::Vec2) -> Self {
    Self {
      ui_min,
      zoom: style.zoom,
      line_stroke: style.line_stroke,
      curve_offset: style.curve_offset,
    }
  }

  /// Convert a point from Graph-space to UI-space.
  pub fn to_ui_pos(&self, pos: emath::Vec2) -> emath::Pos2 {
    (pos * self.zoom).to_pos2() + self.ui_min
  }

  pub fn draw(&self, ui: &mut egui::Ui, start: emath::Pos2, end: emath::Pos2, color: Option<ecolor::Color32>, highlight: bool) -> Option<emath::Rect> {
    let mut offset = (start - end) * 0.2;
    offset.x = self.curve_offset + offset.y.abs() + offset.x.abs();
    let start2 = start - offset;
    let end2 = end + offset;
    let mut shape = egui::epaint::CubicBezierShape {
      points: [start, start2, end2, end],
      closed: false,
      fill: ecolor::Color32::TRANSPARENT,
      stroke: self.line_stroke,
    };
    let rect = shape.visual_bounding_rect();
    if let Some(color) = color {
      shape.stroke.color = color;
    }
    // Check if the mouse pointer is close to the connection.
    let mut hover = false;
    let resp_rect = if highlight {
      let margin = 10.0 * self.zoom;
      let rect = rect.expand(margin);
      match ui.ctx().pointer_latest_pos() {
        Some(pointer) if rect.contains(pointer) => {

          let tolerance = (start.x - end.x).abs() * 0.001;
          let mut last = start;
          shape.for_each_flattened_with_t(tolerance, &mut |pos, _t| {
            // Make bounding box for each line segment.
            let rect = emath::Rect::from_two_pos(last, pos).expand(margin);
            if rect.contains(pointer) {
              hover = true;
            }
            last = pos;
          });

          if hover {
            shape.stroke.width *= 1.8;
            Some(rect)
          } else {
            None
          }
        }
        _ => None,
      }
    } else {
      None
    };
    // Check if part of the connection is visible.
    let id = ui.next_auto_id();
    if ui.is_rect_visible(rect) {
      let mut painter = ui.painter().clone();
      if hover {
        painter = painter.with_layer_id(egui::layers::LayerId::new(egui::layers::Order::Foreground, id));
      }
      painter.add(shape);
    }
    resp_rect
  }
}

#[derive(Clone, Debug, Default)]
pub enum NodeSelectingState {
  #[default]
  None,
  Selecting {
    start: emath::Pos2,
    clear_old: bool,
    area: emath::Rect,
  },
  Select {
    area: emath::Rect,
  },
}

impl NodeSelectingState {
  pub fn update(&mut self, pos: emath::Pos2) {
    match self {
      Self::Selecting { area, start, .. } => {
        // Update selected area.
        *area = emath::Rect::from_points(&[*start, pos]);
      }
      Self::Select { .. } => {
        // Reset back to not selecting state.
        *self = Self::None;
      }
      _ => (),
    }
  }

  pub fn drag_started(&mut self, start: emath::Pos2, clear_old: bool) {
    *self = Self::Selecting {
      start,
      clear_old,
      area: emath::Rect::from_points(&[start]),
    }
  }

  pub fn drag_released(&mut self) {
    if let Self::Selecting { area, .. } = *self {
      *self = Self::Select { area };
    }
  }

  pub fn ui(&self, ui: &egui::Ui) {
    if let Self::Selecting { area, .. } = self {
      ui.painter()
        .rect_stroke(*area, 0.0, (0.5, egui::Color32::LIGHT_GRAY));
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct NodeSocketDragState {
  pub src: Option<NodeSocket>,
  pub dst: Option<NodeSocket>,
  pub pointer_last_pos: Option<emath::Pos2>,
}

impl NodeSocketDragState {
  pub fn is_dragging(&self) -> bool {
    self.src.is_some()
  }

  pub fn take_sockets(
    &mut self,
  ) -> Option<(InputId, Option<(OutputId, DataType)>)> {
    let src = self.src.take()?;
    let dst = self.dst.take();
    self.pointer_last_pos = None;
    src.input_id_first(dst)
  }
}

#[derive(Clone, Debug, Default)]
struct NodeGraphMetaInner {
  ui_min: emath::Vec2,
  zoom: f32,
  origin: emath::Vec2,
  sockets: HashMap<NodeSocketId, NodeSocket>,
  outputs_changed: HashSet<OutputId>,
  frames: IndexMap<Uuid, NodeFrameState>,
  drag_state: NodeSocketDragState,
  selecting_state: NodeSelectingState,
}

impl NodeGraphMetaInner {
  pub fn update(&mut self, origin: emath::Vec2, ui_min: emath::Vec2, zoom: f32) {
    self.origin = origin;
    self.ui_min = ui_min;
    self.zoom = zoom;
  }

  pub fn remove_node(&mut self, node_id: NodeId) {
    self.sockets.retain(|id, _| id.node() != node_id);
    self.frames.retain(|id, _| id != &node_id);
  }

  pub fn take_selected(&mut self) -> Vec<Uuid> {
    self
      .frames
      .iter_mut()
      .filter_map(|(id, frame)| {
        if frame.selected {
          frame.selected = false;
          Some(*id)
        } else {
          None
        }
      })
      .collect()
  }

  pub fn has_selected(&self) -> bool {
    for frame in self.frames.values() {
      if frame.selected {
        return true;
      }
    }
    false
  }

  pub fn frame_state(&self, id: Uuid) -> NodeFrameState {
    self.frames.get(&id).cloned().unwrap_or_default()
  }

  pub fn frame_state_mut<R>(
    &mut self,
    id: Uuid,
    writer: impl FnOnce(&mut NodeFrameState) -> R,
  ) -> R {
    let frame = self.frames.entry(id).or_default();
    writer(frame)
  }

  pub fn set_frame_state(&mut self, id: Uuid, state: NodeFrameState) {
    self.frames.insert(id, state);
  }

  /// Convert from UI screen-space to graph-space and unzoom.
  pub fn ui_to_graph(&self, pos: egui::Pos2) -> emath::Vec2 {
    (pos.to_vec2() - self.ui_min) / self.zoom
  }

  pub fn update_node_socket(&mut self, socket: &mut NodeSocket, pos: egui::Pos2) {
    socket.center = self.ui_to_graph(pos);
    self.sockets.insert(socket.id, socket.clone());
  }

  pub fn update_output(&mut self, output_id: OutputId) {
    self.outputs_changed.insert(output_id);
  }

  pub fn take_updated_outputs(&mut self) -> HashSet<OutputId> {
    self.outputs_changed.drain().collect()
  }

  pub fn resolve_output(
    &self,
    output: &OutputId,
  ) -> Option<DataType> {
    self.sockets.get(&output.into()).map(|meta| meta.dt)
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

  pub fn load(&self, ui: &mut egui::Ui, origin: emath::Vec2, ui_min: emath::Vec2, zoom: f32) {
    let mut inner = self.0.write().unwrap();
    inner.update(origin, ui_min, zoom);
    ui.data_mut(|d| {
      d.insert_temp(egui::Id::new(NODE_GRAPH_META), self.clone());
    });
  }

  pub fn unload(&self, ui: &mut egui::Ui) {
    ui.data_mut(|d| {
      d.remove::<NodeGraphMeta>(egui::Id::new(NODE_GRAPH_META));
    });
  }

  pub fn selecting<R>(&self, reader: impl FnOnce(&NodeSelectingState) -> R) -> R {
    let inner = self.0.read().unwrap();
    reader(&inner.selecting_state)
  }

  pub fn selecting_mut<R>(&self, writer: impl FnOnce(&mut NodeSelectingState) -> R) -> R {
    let mut inner = self.0.write().unwrap();
    writer(&mut inner.selecting_state)
  }

  pub fn take_selected(&self) -> Vec<Uuid> {
    let mut inner = self.0.write().unwrap();
    inner.take_selected()
  }

  pub fn clear_selected(&self) {
    self.take_selected();
  }

  pub fn has_selected(&self) -> bool {
    let inner = self.0.read().unwrap();
    inner.has_selected()
  }

  pub fn frame_state(&self, id: Uuid) -> NodeFrameState {
    let inner = self.0.read().unwrap();
    inner.frame_state(id)
  }

  pub fn set_frame_state(&self, id: Uuid, state: NodeFrameState) {
    let mut inner = self.0.write().unwrap();
    inner.set_frame_state(id, state);
  }

  pub fn frame_state_mut<R>(&self, id: Uuid, writer: impl FnOnce(&mut NodeFrameState) -> R) -> R {
    let mut inner = self.0.write().unwrap();
    inner.frame_state_mut(id, writer)
  }

  pub fn drag_state(&self) -> NodeSocketDragState {
    let inner = self.0.read().unwrap();
    inner.drag_state.clone()
  }

  pub fn set_drag_state(&self, state: NodeSocketDragState) {
    let mut inner = self.0.write().unwrap();
    inner.drag_state = state;
  }

  pub fn drag_state_mut<R>(&self, writer: impl FnOnce(&mut NodeSocketDragState) -> R) -> R {
    let mut inner = self.0.write().unwrap();
    writer(&mut inner.drag_state)
  }

  pub fn remove_node(&self, node_id: NodeId) {
    let mut inner = self.0.write().unwrap();
    inner.remove_node(node_id);
  }

  /// Convert from UI screen-space to graph-space and unzoom.
  pub fn ui_to_graph(&self, pos: egui::Pos2) -> emath::Vec2 {
    let inner = self.0.read().unwrap();
    inner.ui_to_graph(pos)
  }

  /// Convert node position/size from graph-space to screen-space.
  pub fn node_to_ui(&self, mut rect: emath::Rect) -> emath::Rect {
    let inner = self.0.read().unwrap();
    rect.zoom(inner.zoom);
    rect.translate(inner.origin)
  }

  pub fn update_node_socket(&self, socket: &mut NodeSocket, pos: egui::Pos2) {
    let mut inner = self.0.write().unwrap();
    inner.update_node_socket(socket, pos)
  }

  pub fn update_output(&self, output_id: OutputId) {
    let mut inner = self.0.write().unwrap();
    inner.update_output(output_id)
  }

  pub fn take_updated_outputs(&self) -> HashSet<OutputId> {
    let mut inner = self.0.write().unwrap();
    inner.take_updated_outputs()
  }

  pub fn resolve_output(
    &self,
    output: &OutputId,
  ) -> Option<DataType> {
    let inner = self.0.read().unwrap();
    inner.resolve_output(output)
  }

  pub fn get_connection_meta(
    &self,
    input: &InputId,
    output: &OutputId,
  ) -> Option<(NodeSocket, NodeSocket)> {
    let inner = self.0.read().unwrap();
    inner.get_connection_meta(input, output)
  }

  pub fn render<N: NodeFrame + GetId>(
    &self,
    ui: &mut egui::Ui,
    node: &mut N,
  ) -> Option<NodeAction> {
    let id = node.id();
    let mut frame = self.frame_state(id);

    let action = frame.render(ui, self, node);
    self.set_frame_state(id, frame);
    action
  }
}

#[derive(Clone, Debug)]
pub struct NodeSocket {
  pub id: NodeSocketId,
  connected: bool,
  pub center: emath::Vec2,
  pub color: egui::Color32,
  pub dt: DataType,
}

impl NodeSocket {
  pub fn input(node: NodeId, idx: u32, connected: bool, def: &InputDefinition) -> Self {
    let id = NodeSocketId::input(node, idx);
    Self::new(id, connected, def.value_type, def.color)
  }

  pub fn output(node: NodeId, idx: u32, def: &OutputDefinition, concrete_type: Option<DataType>) -> Self {
    let dt = concrete_type.unwrap_or_else(|| def.value_type);
    let id = NodeSocketId::output(node, idx);
    Self::new(id, false, dt, def.color)
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

  pub fn set_data_type(&mut self, dt: DataType) {
    self.dt = dt;
    self.color = dt.color();
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
    let node_style = NodeStyle::get(ui);
    // 1. Deciding widget size:
    let spacing = &ui.spacing();
    let icon_width = spacing.icon_width;
    let desired_size =
      egui::vec2(icon_width, icon_width).at_least(emath::Vec2::splat(spacing.interact_size.y));

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

    // Calculate connecting point (at the edge of the node's frame).
    let (to_edge, dir) = if self.id.is_input() {
      (node_style.input_to_edge, -1.0)
    } else {
      (node_style.output_to_edge, 1.0)
    };
    let end = center + emath::Vec2::from((to_edge, 0.));

    // Update socket metadata.
    graph.update_node_socket(&mut self, end);

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

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, selected, ""));

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
      // We will follow the current style by asking
      // "how should something that is being interacted with be painted?".
      // This will, for instance, give us different colors when the widget is hovered or clicked.
      let style = ui.style();
      let visuals = style.interact_selectable(&response, selected);
      let mut bg_stroke = visuals.bg_stroke;
      let mut line_stroke = node_style.line_stroke;
      bg_stroke.color = self.color;
      line_stroke.color = self.color;
      if hovered {
        bg_stroke.width *= 1.5;
        line_stroke.width *= 1.5;
      }

      let scale = 0.7;
      let big_radius = (big_icon_rect.width() / 2.0 + visuals.expansion) * scale;
      let small_radius = (small_icon_rect.width() / 2.0) * scale;

      let painter = ui.painter();

      // Main socket circle.
      painter.circle(center, big_radius, visuals.bg_fill, bg_stroke);
      // Draw line from socket to edge of the frame.
      let start_offset = emath::Vec2::from((big_radius, 0.)) * dir;
      let start = center + start_offset;
      painter.line_segment([start, end], line_stroke);

      // Fill the socket circle if selected or connected.
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

  pub fn is_input(&self) -> bool {
    self.as_input_id().is_some()
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
