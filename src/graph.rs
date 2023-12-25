use uuid::Uuid;

use indexmap::IndexMap;

use serde::{de::Deserializer, ser::SerializeSeq, Deserialize, Serialize, Serializer};

use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorState {
  size: emath::Vec2,
  origin: emath::Vec2,
  zoom: f32,
  scroll_offset: emath::Vec2,
  #[serde(skip)]
  current_pos: emath::Vec2,
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
      current_pos: Default::default(),
    }
  }
}

#[cfg(feature = "egui")]
impl EditorState {
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
struct NodeMap(pub(crate) IndexMap<NodeId, Node>);

impl Serialize for NodeMap {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
    for n in self.0.values() {
      seq.serialize_element(n)?;
    }
    seq.end()
  }
}

impl<'de> Deserialize<'de> for NodeMap {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let nodes = Vec::<Node>::deserialize(deserializer)?;
    Ok(Self(nodes.into_iter().map(|n| (n.id, n)).collect()))
  }
}

#[derive(Clone, Default, Debug)]
struct ConnectionMap(pub(crate) IndexMap<InputId, OutputId>);

impl Serialize for ConnectionMap {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    #[derive(Serialize)]
    struct Connection<'a> {
      input: &'a InputId,
      output: &'a OutputId,
    }
    let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
    for (input, output) in &self.0 {
      seq.serialize_element(&Connection { input, output })?;
    }
    seq.end()
  }
}

impl<'de> Deserialize<'de> for ConnectionMap {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct Connection {
      input: InputId,
      output: OutputId,
    }
    let connections = Vec::<Connection>::deserialize(deserializer)?;
    Ok(Self(
      connections
        .into_iter()
        .map(|c| (c.input, c.output))
        .collect(),
    ))
  }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeGraph {
  editor: EditorState,
  nodes: NodeMap,
  connections: ConnectionMap,
  output: Option<NodeId>,
}

impl NodeGraph {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add(&mut self, mut node: Node) -> NodeId {
    // Check for duplicate node ids.
    if self.contains(node.id) {
      node.id = Uuid::new_v4();
    }
    let id = node.id;
    self.nodes.0.insert(id, node);
    id
  }

  pub fn remove(&mut self, id: NodeId) -> Option<Node> {
    // Remove all connections to the node.
    self.connections.0.retain(|input, output| {
      if output.node() == id {
        // Need to disconnect inputs from the nodes outputs.
        let node = self.nodes.0.get_mut(&input.node());
        if let Some(node) = node {
          if let Err(err) = node.set_input(*input, Input::Disconnect) {
            log::warn!("Failed to disconnect from input node: {err:?}");
          }
        }
        false
      } else if input.node() == id {
        // We can just remove the nodes own inputs.
        false
      } else {
        // Keep
        true
      }
    });
    // Remove node.
    self.nodes.0.remove(&id)
  }

  pub fn contains(&self, id: NodeId) -> bool {
    self.nodes.0.contains_key(&id)
  }

  pub fn get_input_id<I: Into<InputKey>>(&self, id: NodeId, idx: I) -> Result<InputId> {
    let node = self.get(id)?;
    let idx = node.get_input_idx(&idx.into())?;
    Ok(InputId::new(id, idx))
  }

  pub fn get_node_input<I: Into<InputKey>>(&self, id: NodeId, idx: I) -> Result<Input> {
    self.get(id).and_then(|n| n.get_input(idx.into()))
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
      .0
      .get_mut(&id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))?;
    // Convert Input key to id.
    let input_id = node.get_input_idx(&key).map(|idx| InputId::new(id, idx))?;
    match &value {
      Input::Disconnect => {
        self.connections.0.remove(&input_id);
      }
      Input::Connect(output_id) => {
        self.connections.0.insert(input_id, *output_id);
      }
      _ => {}
    }
    // Set the node input.
    Ok(node.set_input(key, value)?)
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

  pub fn get(&self, id: NodeId) -> Result<&Node> {
    self
      .nodes
      .0
      .get(&id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))
  }

  pub fn get_mut(&mut self, id: NodeId) -> Result<&mut Node> {
    self
      .nodes
      .0
      .get_mut(&id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))
  }

  pub fn set_output(&mut self, output: Option<NodeId>) {
    self.output = output;
  }

  pub fn output(&self) -> Option<NodeId> {
    self.output
  }

  #[cfg(feature = "egui")]
  pub fn ui(&mut self, ui: &mut egui::Ui) {
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
      ui.set_node_graph_meta(NodeGraphMeta { ui_min, zoom });

      // Convert pointer position to graph-space.  (Used for adding new nodes).
      if ui.ui_contains_pointer() {
        if let Some(pos) = ui.ctx().pointer_latest_pos() {
          self.editor.current_pos = (pos - origin).to_vec2() / zoom;
        }
      }

      // Render nodes.
      let mut remove_node = None;
      for (node_id, node) in &mut self.nodes.0 {
        node.ui_at(ui, origin).context_menu(|ui| {
          if ui.button("Delete").clicked() {
            remove_node = Some(*node_id);
            ui.close_menu();
          }
        });
      }
      // Handle removing a node.
      if let Some(node_id) = remove_node {
        self.remove(node_id);
      }

      // Handle connecting/disconnecting.
      if ui.input(|i| i.pointer.any_released()) {
        // Check if a connection was being dragged.
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
        // Still dragging a connection.
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
          let src_meta = ui
            .data(|d| d.get_temp::<NodeSocketMeta>(src.ui_id()))
            .unwrap();
          let center = (src_meta.center * zoom).to_pos2() + ui_min;
          ui.painter()
            .line_segment([center, end], node_style.line_stroke);
        }
      }

      // Draw connections.
      let painter = ui.painter();
      for (input, output) in &self.connections.0 {
        let in_id = input.ui_id();
        let out_id = output.ui_id();
        let meta = ui.data(|d| {
          d.get_temp::<NodeSocketMeta>(in_id).and_then(|in_meta| {
            d.get_temp::<NodeSocketMeta>(out_id)
              .map(|out_meta| (in_meta, out_meta))
          })
        });
        if let Some((in_meta, out_meta)) = meta {
          // Convert the sockets back to screen-space
          // and apply zoom.
          let in_pos = (in_meta.center * zoom).to_pos2() + ui_min;
          let out_pos = (out_meta.center * zoom).to_pos2() + ui_min;
          let rect = egui::Rect::from_points(&[in_pos, out_pos]);
          // Check if part of the connection is visible.
          if ui.is_rect_visible(rect) {
            painter.line_segment([in_pos, out_pos], node_style.line_stroke);
          }
        }
      }

      // Restore old NodeStyle.
      ui.set_node_style(old_node_style);
    });
    // Save scroll offset and de-zoom it.
    self.editor.scroll_offset = resp.state.offset / zoom;
  }
}

#[derive(Clone)]
#[cfg(feature = "egui")]
pub struct NodeGraphEditor {
  pub size: emath::Vec2,
  pub graph: NodeGraph,
  pub registry: NodeRegistry,
  pub node_filter: NodeFilter,
  next_position: Option<emath::Vec2>,
}

#[cfg(feature = "egui")]
impl Default for NodeGraphEditor {
  fn default() -> Self {
    Self {
      size: (900., 500.).into(),
      graph: Default::default(),
      registry: build_registry(),
      node_filter: Default::default(),
      next_position: None,
    }
  }
}

#[cfg(feature = "egui")]
impl NodeGraphEditor {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn show(&mut self, ctx: &egui::Context) {
    egui::Window::new("Graph editor")
      .default_size(self.size)
      .show(ctx, |ui| {
        egui::SidePanel::right("graph_right_panel").show_inside(ui, |ui| {
          ui.label("TODO: Node finder here");
        });
        let resp = egui::CentralPanel::default()
          .show_inside(ui, |ui| {
            self.graph.ui(ui);
          })
          .response;
        // Graph menu.
        resp.context_menu(|ui| {
          if self.next_position.is_none() {
            self.next_position = Some(self.graph.editor.current_pos);
          }
          // Node filter UI first.
          self.node_filter.ui(ui);
          if let Some(mut node) = self.registry.ui(ui, &self.node_filter) {
            if let Some(position) = self.next_position.take() {
              node.set_position(position);
            }
            self.graph.add(node);
            ui.close_menu();
          }
        });
      });
  }
}
