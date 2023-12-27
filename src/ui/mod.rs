use egui::{self, NumExt};

use crate::node::{InputId, NodeId, OutputId};
use crate::values::DataType;

mod frame;
mod zoom;
pub use frame::*;
pub use zoom::*;

const NODE_STYLE: &'static str = "NodeStyle";
const NODE_GRAPH_META: &'static str = "NodeGraphMeta";
const NODE_SOCKET_DRAG_SRC: &'static str = "NodeSocketDragSrc";
const NODE_SOCKET_DRAG_DST: &'static str = "NodeSocketDragDst";
const NODE_INPUT_ID: &'static str = "NodeInputId";
const NODE_OUTPUT_ID: &'static str = "NodeOutputId";

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
      line_stroke: (5.0, egui::Color32::WHITE).into(),
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

pub trait NodeGraphAccess {
  fn set_src_node_socket(&mut self, id: NodeSocketId);

  fn get_src_node_socket(&self) -> Option<NodeSocketId>;

  fn set_dst_node_socket(&mut self, id: NodeSocketId);

  fn clear_dst_node_socket(&mut self);

  fn get_dst_node_socket(&self) -> Option<NodeSocketId>;

  fn get_dropped_node_sockets(&mut self) -> Option<(NodeSocketId, Option<NodeSocketId>)>;

  fn update_node_socket(&mut self, id: NodeSocketId, pos: egui::Pos2);

  fn ui_to_graph(&self, pos: egui::Pos2) -> egui::Vec2;

  fn node_graph_meta(&self) -> NodeGraphMeta;

  fn set_node_graph_meta(&mut self, node_graph: NodeGraphMeta);

  fn node_style(&self) -> NodeStyle;

  fn set_node_style(&mut self, node_style: NodeStyle);

  fn zoom_style(&mut self, zoom: f32) -> NodeStyle;
}

impl NodeGraphAccess for egui::Ui {
  fn set_src_node_socket(&mut self, id: NodeSocketId) {
    self.data_mut(|d| d.insert_temp(egui::Id::new(NODE_SOCKET_DRAG_SRC), id));
  }

  fn get_src_node_socket(&self) -> Option<NodeSocketId> {
    self.data(|d| d.get_temp::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_SRC)))
  }

  fn set_dst_node_socket(&mut self, id: NodeSocketId) {
    self.data_mut(|d| d.insert_temp(egui::Id::new(NODE_SOCKET_DRAG_DST), id));
  }

  fn clear_dst_node_socket(&mut self) {
    self.data_mut(|d| d.remove::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_DST)));
  }

  fn get_dst_node_socket(&self) -> Option<NodeSocketId> {
    self.data(|d| d.get_temp::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_DST)))
  }

  fn get_dropped_node_sockets(&mut self) -> Option<(NodeSocketId, Option<NodeSocketId>)> {
    self.data_mut(|d| {
      let src = d.get_temp::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_SRC));
      let dst = d.get_temp::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_DST));
      d.remove::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_SRC));
      d.remove::<NodeSocketId>(egui::Id::new(NODE_SOCKET_DRAG_DST));
      src.map(|src| (src, dst))
    })
  }

  fn update_node_socket(&mut self, id: NodeSocketId, pos: egui::Pos2) {
    self.data_mut(|d| {
      let graph = d
        .get_temp::<NodeGraphMeta>(egui::Id::new(NODE_GRAPH_META))
        .unwrap_or_default();
      let meta = d.get_temp_mut_or_default::<NodeSocketMeta>(id.ui_id());
      meta.center = graph.ui_to_graph(pos);
    });
  }

  fn ui_to_graph(&self, pos: egui::Pos2) -> egui::Vec2 {
    let graph = self.node_graph_meta();
    graph.ui_to_graph(pos)
  }

  fn node_graph_meta(&self) -> NodeGraphMeta {
    self
      .data(|d| d.get_temp::<NodeGraphMeta>(egui::Id::new(NODE_GRAPH_META)))
      .unwrap_or_default()
  }

  fn set_node_graph_meta(&mut self, node_graph: NodeGraphMeta) {
    self.data_mut(|d| d.insert_temp(egui::Id::new(NODE_GRAPH_META), node_graph));
  }

  fn node_style(&self) -> NodeStyle {
    self
      .data(|d| d.get_temp::<NodeStyle>(egui::Id::new(NODE_STYLE)))
      .unwrap_or_default()
  }

  fn set_node_style(&mut self, node_style: NodeStyle) {
    self.data_mut(|d| d.insert_temp(egui::Id::new(NODE_STYLE), node_style));
  }

  fn zoom_style(&mut self, zoom: f32) -> NodeStyle {
    self.style_mut().zoom(zoom);
    self.data_mut(|d| {
      let node_style = d.get_temp_mut_or_default::<NodeStyle>(egui::Id::new(NODE_STYLE));
      node_style.zoom(zoom);
      node_style.clone()
    })
  }
}

#[derive(Clone, Debug, Default)]
pub struct NodeGraphMeta {
  pub ui_min: egui::Vec2,
  pub zoom: f32,
}

impl NodeGraphMeta {
  /// Convert from UI screen-space to graph-space and unzoom.
  pub fn ui_to_graph(&self, pos: egui::Pos2) -> egui::Vec2 {
    (pos.to_vec2() - self.ui_min) / self.zoom
  }
}

#[derive(Clone, Debug, Default)]
pub struct NodeSocketMeta {
  pub center: egui::Vec2,
}

pub struct NodeSocket {
  id: NodeSocketId,
  connected: bool,
}

impl NodeSocket {
  pub fn new(id: NodeSocketId, connected: bool) -> Self {
    Self { id, connected }
  }
}

impl egui::Widget for NodeSocket {
  fn ui(self, ui: &mut egui::Ui) -> egui::Response {
    let Self { id, connected } = self;

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

    // Update socket metadata.
    ui.update_node_socket(id, center);

    // 3. Interact: Time to check for clicks!
    if response.drag_started() {
      ui.set_src_node_socket(id);
    }
    // HACK: Fix hover during drag.
    let mut hovered = ui.rect_contains_pointer(rect) || response.hovered();
    if hovered {
      if let Some(src) = ui.get_src_node_socket() {
        // Check if src socket is compatible.
        if src.is_compatible(id) {
          ui.set_dst_node_socket(id);
        } else {
          hovered = false;
          ui.clear_dst_node_socket();
        }
      }
    } else if ui.get_dst_node_socket() == Some(id) {
      // No longer hovering our socket.  Cleanup.
      ui.clear_dst_node_socket();
    }
    let selected = hovered || connected;

    // We will follow the current style by asking
    // "how should something that is being interacted with be painted?".
    // This will, for instance, give us different colors when the widget is hovered or clicked.
    let style = ui.style();
    let mut visuals = style.interact_selectable(&response, selected);
    // HACK: response.hovered() doesn't work during drag.
    if hovered {
      visuals.bg_stroke = style.visuals.widgets.hovered.bg_stroke;
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

      painter.circle(center, big_radius, visuals.bg_fill, visuals.bg_stroke);

      if selected {
        painter.circle_filled(
          center,
          small_radius,
          visuals.fg_stroke.color, // Intentional to use stroke and not fill
        );
      }
    }
    response
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeSocketId {
  Input(u32, InputId, DataType),
  Output(u32, OutputId, DataType),
}

impl NodeSocketId {
  pub fn input(graph: u32, node: NodeId, idx: u32, dt: DataType) -> Self {
    Self::Input(graph, InputId::new(node, idx), dt)
  }

  pub fn output(graph: u32, node: NodeId, idx: u32, dt: DataType) -> Self {
    Self::Output(graph, OutputId::new(node, idx), dt)
  }

  pub fn is_compatible(&self, dst: NodeSocketId) -> bool {
    match (self, &dst) {
      (Self::Input(g_in, input, dt_in), Self::Output(g_out, output, dt_out))
      | (Self::Output(g_out, output, dt_out), Self::Input(g_in, input, dt_in))
        if g_in == g_out =>
      {
        input.node != output.node && dt_in.is_compatible(dt_out)
      }
      _ => false,
    }
  }

  pub fn input_id_first(
    &self,
    dst: Option<NodeSocketId>,
  ) -> Option<(InputId, Option<(OutputId, DataType)>)> {
    match (*self, dst) {
      // Disconect input.
      (Self::Input(_, id, _), None) => Some((id, None)),
      // Connect input to output.
      (Self::Input(g_in, input, _), Some(Self::Output(g_out, output, dt)))
        if g_in == g_out && input.node != output.node =>
      {
        Some((input, Some((output, dt))))
      }
      // Connect output to input.
      (Self::Output(g_out, output, dt), Some(Self::Input(g_in, input, _)))
        if g_in == g_out && input.node != output.node =>
      {
        Some((input, Some((output, dt))))
      }
      // Other non-compatible connections.
      _ => None,
    }
  }

  pub fn is_input(&self) -> bool {
    self.as_input_id().is_some()
  }

  pub fn as_input_id(&self) -> Option<InputId> {
    match self {
      Self::Input(_, id, _) => Some(*id),
      _ => None,
    }
  }

  pub fn is_output(&self) -> bool {
    self.as_output_id().is_some()
  }

  pub fn as_output_id(&self) -> Option<OutputId> {
    match self {
      Self::Output(_, id, _) => Some(*id),
      _ => None,
    }
  }

  pub fn ui_id(&self) -> egui::Id {
    match self {
      Self::Input(_graph, id, _) => id.ui_id(),
      Self::Output(_graph, id, _) => id.ui_id(),
    }
  }
}

impl InputId {
  pub fn ui_id(&self) -> egui::Id {
    egui::Id::new(NODE_INPUT_ID).with(self.node).with(self.idx)
  }
}

impl OutputId {
  pub fn ui_id(&self) -> egui::Id {
    egui::Id::new(NODE_OUTPUT_ID).with(self.node).with(self.idx)
  }
}
