use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use slotmap::SlotMap;

use uuid::Uuid;

use anyhow::{anyhow, Result};

use crate::*;
#[cfg(feature = "egui")]
use crate::ui::{
  NodeGraphAccess,
  NodeGraphMeta,
  NodeSocketMeta,
  Zoom,
};

#[derive(Default, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EditorState {
  size: emath::Vec2,
  origin: emath::Vec2,
  zoom: f32,
  scroll_offset: emath::Vec2,
  // stats
  #[serde(skip)]
  render_nodes: Duration,
  #[serde(skip)]
  render_connections: Duration,
  #[serde(skip)]
  last_update: Option<Instant>,
}

impl Default for EditorState {
  fn default() -> Self {
    let size = emath::vec2(10000.0, 10000.0);
    let origin = size / 2.0;
    Self {
      size,
      origin,
      zoom: 0.5,
      scroll_offset: origin - emath::vec2(450., 250.),
      render_nodes: Default::default(),
      render_connections: Default::default(),
      last_update: None,
    }
  }
}

#[cfg(feature = "egui")]
impl EditorState {
  fn update_stats(&mut self, render_nodes: Duration, render_connections: Duration) {
    if let Some(last_update) = self.last_update {
      if last_update.elapsed() < Duration::from_secs(1) {
        return;
      }
    }
    self.last_update = Some(Instant::now());
    self.render_nodes = (self.render_nodes + render_nodes) / 2;
    self.render_connections = (self.render_connections + render_connections) / 2;
  }

  fn get_zoomed(&self) -> (emath::Vec2, emath::Vec2, emath::Vec2, f32) {
    let mut size = self.size;
    let mut origin = self.origin;
    let mut scroll_offset = self.scroll_offset;
    size.zoom(self.zoom);
    origin.zoom(self.zoom);
    scroll_offset.zoom(self.zoom);
    (size, origin, scroll_offset, self.zoom)
  }
}


#[derive(Clone, Default, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeGraph {
  editor: EditorState,
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
    Ok(InputId::new(id, idx))
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
    let input_id = node.get_input_idx(&key).map(|idx| InputId::new(id, idx))?;
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

#[cfg(feature = "egui")]
impl NodeGraph {
  pub fn show(&mut self, ctx: &egui::Context) {
    egui::Window::new("Graph editor")
      .default_size((900., 500.))
      .show(ctx, |ui| {
        egui::SidePanel::right("graph_right_panel").show_inside(ui, |ui| {
          ui.label("zoom:");
          ui.add(egui::Slider::new(&mut self.editor.zoom, 0.1..=1.0).text("Zoom"));
          ui.label(format!("Render nodes: {:?}", self.editor.render_nodes));
          ui.label(format!("Render connections: {:?}", self.editor.render_connections));
          ui.label("TODO: Node finder here");
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
          self.node_graph_ui(ui);
        });
      });
  }

  fn node_graph_ui(&mut self, ui: &mut egui::Ui) {
    // Use mouse wheel for zoom instead of scrolling.
    // Mouse wheel + ctrl scrolling left/right.
    // Multitouch (pinch gesture) zoom.
    let mut scrolling = true;
    if ui.ui_contains_pointer() {
      let z_delta = ui.input(|i| {
        // Use up/down mouse wheel for zoom.
        let scroll_delta = i.scroll_delta.y;
        if scroll_delta > 0.1 {
          0.01
        } else if scroll_delta < -0.1 {
          -0.01
        } else {
          // For Multitouch devices (pinch gesture).
          i.zoom_delta() - 1.0
        }
      });
      if z_delta != 0.0 {
        let zoom = (self.editor.zoom + z_delta).clamp(0.1, 1.0);
        self.editor.zoom = zoom;
        scrolling = false;
      }
    }
    let (size, origin, scroll_offset, zoom) = self.editor.get_zoomed();
    // Create scroll area and restore zoomed scroll offset.
    let scroll_area = egui::ScrollArea::both()
      .enable_scrolling(scrolling)
      .scroll_offset(scroll_offset);

    // Show scroll area.
    let resp = scroll_area.show(ui, |ui| {
      // Save old node style.
      let old_node_style = ui.node_style();

      // Apply zoom to Ui style.
      let node_style = ui.zoom_style(zoom);

      // Set node graph area.
      ui.set_width(size.x);
      ui.set_height(size.y);
      // Need UI screen-space `min` to covert Node Graph positions to screen-space.
      let ui_min = ui.min_rect().min.to_vec2();
      let origin = origin + ui_min;
      ui.set_node_graph_meta(NodeGraphMeta {
        ui_min,
        zoom,
      });

      let now = Instant::now();
      // Render nodes.
      for (id, node) in &mut self.nodes {
        node.ui_at(ui, origin, id);
      }
      let render_nodes = now.elapsed();

      if ui.input(|i| i.pointer.any_released()) {
        if let Some((src, dst)) = ui.get_dropped_node_sockets() {
          // Make sure the input is first and that the sockets are compatible.
          if let Some((src, dst)) = src.input_id_first(dst) {
            if let Some(dst) = dst {
              // Connect.
              if let Err(err) = self.connect(src, dst) {
                log::warn!("Failed to connect input[{src:?}] to output[{dst:?}]: {err:?}");
              }
            } else {
              // Disconnect
              if let Err(err) = self.disconnect(src) {
                log::warn!("Failed to disconnect input[{src:?}]: {err:?}");
              }
            }
          }
        }
      } else if let Some(src) = ui.get_src_node_socket() {
        if ui.get_dst_node_socket().is_some() {
          ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        } else {
          ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        // If the dragged socket is an input, then remove it's current connection.
        if let Some(src) = src.as_input_id() {
          if let Err(err) = self.disconnect(src) {
            log::warn!("Failed to disconnect input[{src:?}]: {err:?}");
          }
        }
        if let Some(end) = ui.ctx().pointer_latest_pos() {
          let src_meta = ui.data(|d| d.get_temp::<NodeSocketMeta>(src.ui_id())).unwrap();
          let center = (src_meta.center * zoom).to_pos2() + ui_min;
          let layer_id = egui::LayerId::new(egui::Order::Foreground, ui.id());
          ui.with_layer_id(layer_id, |ui| {
            ui.painter().line_segment([center, end], node_style.line_stroke);
          });
        }
      }

      // Draw connections.
      let now = std::time::Instant::now();
      let layer_id = egui::LayerId::new(egui::Order::Foreground, ui.id());
      ui.with_layer_id(layer_id, |ui| {
        let painter = ui.painter();
        for (input, output) in &self.connections {
          let in_id = input.ui_id();
          let out_id = output.ui_id();
          let meta = ui.data(|d| {
            d.get_temp::<NodeSocketMeta>(in_id).and_then(|in_meta| {
              d.get_temp::<NodeSocketMeta>(out_id).map(|out_meta| (in_meta, out_meta))
            })
          });
          if let Some((in_meta, out_meta)) = meta {
            let rect = egui::Rect::from_min_max(
              (in_meta.center * zoom).to_pos2(),
              (out_meta.center * zoom).to_pos2(),
            ).translate(ui_min);
            if ui.is_rect_visible(rect) {
              painter.line_segment([rect.min, rect.max], node_style.line_stroke);
            }
          }
        }
      });
      let render_connections = now.elapsed();
      self.editor.update_stats(render_nodes, render_connections);

      // Restore old NodeStyle.
      ui.set_node_style(old_node_style);
    });
    // Save scroll offset and de-zoom it.
    self.editor.scroll_offset = resp.state.offset / zoom;
  }
}
