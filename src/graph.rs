use std::fs::File;
use std::path::Path;

use std::collections::BTreeSet;
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
  graph_pointer_pos: Option<emath::Vec2>,
  #[serde(skip)]
  add_node_at: Option<emath::Vec2>,
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
      graph_pointer_pos: None,
      add_node_at: None,
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

pub trait GetId {
  fn id(&self) -> Uuid;
}

#[derive(Clone, Debug)]
struct IdMap<V>(pub(crate) IndexMap<Uuid, V>);

impl<V> Default for IdMap<V> {
  fn default() -> Self {
    Self(Default::default())
  }
}

impl<V> Serialize for IdMap<V>
where
  V: Serialize,
{
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

impl<'de, V> Deserialize<'de> for IdMap<V>
where
  V: GetId + serde::de::DeserializeOwned,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let nodes = Vec::<V>::deserialize(deserializer)?;
    Ok(Self(nodes.into_iter().map(|n| (n.id(), n)).collect()))
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
pub struct NodeGraphProperty {
  id: Uuid,
  name: String,
  description: String,
  value: Value,
}

impl GetId for NodeGraphProperty {
  fn id(&self) -> Uuid {
    self.id
  }
}

#[derive(Clone, Debug)]
pub struct NodeFinder {
  pub registry: NodeRegistry,
  pub node_filter: NodeFilter,
  open: bool,
  open_at: Option<emath::Pos2>,
}

impl Default for NodeFinder {
  fn default() -> Self {
    Self {
      registry: NodeRegistry::build(),
      node_filter: Default::default(),
      open: false,
      open_at: None,
    }
  }
}

impl NodeFinder {
  pub fn open_at(&mut self, pos: emath::Pos2) {
    self.open_at = Some(pos);
    self.open = true;
  }

  pub fn close(&mut self) {
    self.open = false;
    self.open_at = None;
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<Node> {
    if !self.open {
      return None;
    }

    let mut area = egui::Area::new("NodeFinder");
    if let Some(pos) = self.open_at.take() {
      area = area.current_pos(pos);
    }
    let node = area
      .show(ui.ctx(), |ui| {
        self.frame_ui(ui)
      }).inner;
    if node.is_some() {
      // A node was selected close the finder.
      self.close();
    }
    node
  }

  // Draw a frame around the node finder UI.
  fn frame_ui(
    &mut self,
    ui: &mut egui::Ui,
  ) -> Option<Node> {
    // Window-style frame.
    let style = ui.style();
    let mut frame = egui::Frame::window(style);
    frame.shadow = Default::default();

    let mut node = None;
    frame.show(ui, |ui| {
      ui.vertical(|ui| {
        // Title bar.
        ui.label("Create node");
        // Node filter UI.
        self.node_filter.ui(ui);
        // Show available nodes from registry.
        node = self.registry.ui(ui, &self.node_filter);
      });
    });
    node
  }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct MenuState {
  hover_connection: Option<InputId>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct NodeGraph {
  id: Uuid,
  editor: EditorState,
  properties: IdMap<NodeGraphProperty>,
  nodes: IdMap<Node>,
  groups: IdMap<NodeGroup>,
  connections: ConnectionMap,
  output: Option<NodeId>,
  #[serde(skip)]
  changed: usize,
  #[serde(skip)]
  hover_connection: Option<InputId>,
  #[serde(skip)]
  menu_state: Option<MenuState>,
  #[serde(skip)]
  #[cfg(feature = "egui")]
  ui_state: NodeGraphMeta,
  #[serde(skip)]
  node_finder: NodeFinder,
}

impl NodeGraph {
  pub fn new() -> Self {
    Self {
      id: Uuid::new_v4(),
      ..Self::default()
    }
  }

  pub fn add_group(&mut self, mut group: NodeGroup) -> NodeGroupId {
    // Check for duplicate node group ids.
    if self.groups.0.contains_key(&group.id) {
      group.id = Uuid::new_v4();
    }
    let id = group.id;
    self.groups.0.insert(id, group);
    id
  }

  pub fn remove_group(&mut self, group_id: NodeGroupId, delete_nodes: bool) {
    self.groups.0.remove(&group_id);
    if delete_nodes {
      let mut nodes = Vec::new();
      for (node_id, node) in &mut self.nodes.0 {
        if node.group_id == group_id {
          nodes.push(*node_id);
        }
      }
      for node_id in nodes {
        self.remove(node_id);
      }
    } else {
      for (_, node) in &mut self.nodes.0 {
        if node.group_id == group_id {
          node.group_id = Uuid::nil();
        }
      }
    }
  }

  pub fn resize_group(&mut self, group_id: NodeGroupId) {
    if let Some(group) = self.groups.0.get_mut(&group_id) {
      let mut area = emath::Rect::NOTHING;
      for (_, node) in &mut self.nodes.0 {
        if node.group_id == group_id {
          area = area.union(node.rect());
        }
      }
      group.set_area(area);
    }
  }

  /// Returns the `changed` counter to detect when the graph needs to be recompiled.
  pub fn changed_counter(&self) -> usize {
    self.changed
  }

  // Inc. the `changed` counter to detect when the graph needs to be recompiled.
  fn updated(&mut self) {
    self.changed += 1;
  }

  pub fn add(&mut self, mut node: Node) -> NodeId {
    self.updated();
    if let Some(position) = &self.editor.add_node_at {
      node.set_position(*position);
    }
    // Check for duplicate node ids.
    if self.contains(node.id()) {
      node.new_id();
    }
    let id = node.id();
    self.nodes.0.insert(id, node);
    id
  }

  pub fn remove(&mut self, id: NodeId) -> Option<Node> {
    self.updated();
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
    #[cfg(feature = "egui")]
    {
      // Remove all UI state for the node
      self.ui_state.remove_node(id);
    }
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
    // Update connections.
    match &value {
      Input::Disconnect => {
        if self.connections.0.remove(&input_id).is_none() {
          // The input was already disconnected.  No change.
          return Ok(None);
        }
      }
      Input::Connect(output_id, _) => {
        self.connections.0.insert(input_id, *output_id);
      }
      _ => {}
    }
    // Set the node input.
    let old = node.set_input(key, value.clone())?;
    // Mark graph as updated.
    self.updated();
    Ok(old)
  }

  pub fn set_input(&mut self, input_id: InputId, value: Input) -> Result<Option<OutputId>> {
    self.set_node_input(input_id.node(), input_id, value)
  }

  pub fn disconnect(&mut self, input: InputId) -> Result<()> {
    self.set_input(input, Input::Disconnect)?;
    Ok(())
  }

  pub fn connect(&mut self, input: InputId, output: OutputId, dt: DataType) -> Result<()> {
    self.set_input(input, Input::Connect(output, Some(dt)))?;
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
    self.updated();
    self
      .nodes
      .0
      .get_mut(&id)
      .ok_or_else(|| anyhow!("Missing node: {id:?}"))
  }

  pub fn set_output(&mut self, output: Option<NodeId>) {
    self.updated();
    self.output = output;
  }

  pub fn output(&self) -> Option<NodeId> {
    self.output
  }
}

#[cfg(feature = "egui")]
impl NodeGraph {
  /// Returns true if there are selected nodes.
  pub fn has_selected(&self) -> bool {
    self.ui_state.has_selected()
  }

  /// Is the pointer hover a connection.
  pub fn hover_connection(&self) -> Option<InputId> {
    self.hover_connection
  }

  pub fn open_node_finder(&mut self, ui: &egui::Ui) {
    if let Some(pos) = ui.ctx().pointer_latest_pos() {
      self.editor.add_node_at = self.editor.graph_pointer_pos;
      self.node_finder.open_at(pos);
    }
  }

  pub fn group_selected_nodes(&mut self) -> Option<NodeGroupId> {
    let mut group = NodeGroup::new();

    let mut empty = true;
    for node_id in self.ui_state.take_selected() {
      if let Some(node) = self.nodes.0.get_mut(&node_id) {
        group.add_node(node);
        empty = false;
      }
    }

    if empty {
      None
    } else {
      let id = group.id;
      self.groups.0.insert(id, group);
      Some(id)
    }
  }

  pub fn select_node(&mut self, id: NodeId, select: bool) {
    self.ui_state.frame_state_mut(id, |frame| {
      frame.selected = select;
    });
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    // Show the node finder if it is open.
    if let Some(node) = self.node_finder.ui(ui) {
      self.add(node);
    }

    let mut scrolling = true;
    let mut selecting = true;
    let mut clear_selected = true;
    // Detect drag mode.
    // * Select nodes only in dragged area - Primary mouse button and no modifiers.
    // * Add nodes in dragged area to selected set - Primary mouse button + SHIFT.
    // * Scroll - Primary mouse button + CTRL.
    ui.input(|i| {
      // Enable scrolling when CTRL is down.
      if i.modifiers.ctrl {
        selecting = false;
      }
      // Don't scroll from secondary mouse button.
      if i.pointer.secondary_down() {
        // Don't clear selected when opening the Context menu.
        clear_selected = false;
        scrolling = false;
      }
      // When SHIFT is down keep current selected nodes.
      if i.modifiers.shift {
        clear_selected = false;
      }
    });

    if ui.ui_contains_pointer() {
      // Use mouse wheel for zoom instead of scrolling.
      // Mouse wheel + ctrl scrolling left/right.
      // Multitouch (pinch gesture) zoom.
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
    let out = scroll_area.show(ui, |ui| {
      // Id for selecting nodes or dragging connections.
      let id = ui.next_auto_id();

      // Save old node style.
      let old_node_style = NodeStyle::get(ui);

      // Apply zoom to Ui style.
      let node_style = NodeStyle::zoom_style(ui, zoom);

      // Set node graph area.
      ui.set_width(size.x);
      ui.set_height(size.y);
      // Need UI screen-space `min` to covert from graph-space to screen-space.
      let ui_min = ui.min_rect().min.to_vec2();
      let origin = origin + ui_min;
      let state = self.ui_state.clone();
      state.load(ui, origin, ui_min, zoom);

      // Convert pointer position to graph-space.  (Used for adding new nodes).
      let mut pointer_pos = emath::Pos2::default();
      if let Some(pos) = ui.ctx().pointer_latest_pos() {
        pointer_pos = pos;
        if ui.ui_contains_pointer() {
          self.editor.graph_pointer_pos = Some((pos - origin).to_vec2() / zoom);
        }
      }
      // When not scrolling, detect click and drag to select nodes.
      let mut area_resp = None;
      let mut select_state = None;
      if selecting {
        let rect = ui.available_rect_before_wrap();
        let resp = ui.interact(rect, id, egui::Sense::click_and_drag());
        state.selecting_mut(|selecting| {
          if resp.drag_started() {
            selecting.drag_started(pointer_pos, clear_selected);
            // Close the NodeFinder on clicks.
            self.node_finder.close();
          } else if resp.drag_released() {
            selecting.drag_released();
          } else {
            selecting.update(pointer_pos);
          }
          select_state = Some(selecting.clone());
        });
        area_resp = Some(resp);
      }

      // Render groups.
      let mut remove_group = None;
      let mut resize_groups = BTreeSet::new();
      for (group_id, group) in &mut self.groups.0 {
        match state.render(ui, group) {
          Some(NodeAction::Dragged(delta)) => {
            let delta = delta / zoom;
            for (_, node) in &mut self.nodes.0 {
              if node.group_id == *group_id {
                node.handle_move(delta);
              }
            }
          }
          Some(NodeAction::Delete(nodes)) => {
            remove_group = Some((*group_id, nodes));
          }
          Some(NodeAction::JoinGroup(group_id)) => {
            for node_id in self.ui_state.take_selected() {
              if let Some(node) = self.nodes.0.get_mut(&node_id) {
                node.group_id = group_id;
              }
            }
            resize_groups.insert(group_id);
          }
          _ => (),
        }
      }
      if let Some((group_id, remove_nodes)) = remove_group {
        self.remove_group(group_id, remove_nodes);
      }

      // Draw connections.
      let connection_style = NodeConnection::new(&node_style, ui_min);
      self.render_connections(ui, id, &state, connection_style);

      // Render nodes.
      let mut remove_node = None;
      let mut updated = false;
      for (node_id, node) in &mut self.nodes.0 {
        match state.render(ui, node) {
          Some(NodeAction::Dragged(_) | NodeAction::Resize) => {
            if !node.group_id.is_nil() {
              resize_groups.insert(node.group_id);
            }
          }
          Some(NodeAction::Delete(_)) => {
            remove_node = Some(*node_id);
          }
          Some(NodeAction::LeaveGroup(group_id)) => {
            resize_groups.insert(group_id);
          }
          _ => (),
        }
        updated |= node.updated;
      }

      // Check for outputs that have changed their data types.
      let outputs = state.take_updated_outputs();
      if outputs.len() > 0 {
        // Update any node that is connected to the changed outputs.
        for (input, output) in self.connections.0.iter() {
          if outputs.contains(output) {
            if let Some(node) = self.nodes.0.get_mut(&input.node()) {
              node.updated = true;
            }
          }
        }
      }
      // Check if any of the nodes have been updated.
      if updated {
        self.updated();
      }
      // Handle node actions.
      if let Some(node_id) = remove_node {
        self.remove(node_id);
      }
      for group_id in resize_groups {
        self.resize_group(group_id);
      }

      // Unload the graph state from egui.
      state.unload(ui);

      if let Some(selecting) = select_state {
        selecting.ui(ui);
      }

      // Restore old NodeStyle and unzoom
      old_node_style.unzoom_style(ui, zoom);

      area_resp
    });
    // Save scroll offset and de-zoom it.
    self.editor.scroll_offset = out.state.offset / zoom;

    if let Some(resp) = out.inner {
      resp.context_menu(|ui| self.context_menu(ui));
      if !ui.ctx().is_context_menu_open() {
        self.menu_state = None;
      }
    }
  }

  fn context_menu(&mut self, ui: &mut egui::Ui) {
    let state = self.menu_state.get_or_insert_with(|| {
      MenuState {
        hover_connection: self.hover_connection,
      }
    }).clone();
    if ui.button("Create node").clicked() {
      self.open_node_finder(ui);
      ui.close_menu();
    }
    if self.has_selected() && ui.button("Group Nodes").clicked() {
      self.group_selected_nodes();
      ui.close_menu();
    }
    if let Some(input) = state.hover_connection {
      if ui.button("Delete connection").clicked() {
        if let Err(err) = self.disconnect(input) {
          log::error!("Failed to delete connection: {err:?}");
        }
        ui.close_menu();
      }
    }
  }

  fn render_connections(&mut self, ui: &mut egui::Ui, id: egui::Id, state: &NodeGraphMeta, conn: NodeConnection) {
    //let zoom = style.zoom;
    // Check if a connection is being dragged.
    state.drag_state_mut(|drag| {
      // Handle connecting/disconnecting.
      if ui.memory(|m| m.is_being_dragged(id)) && ui.input(|i| i.pointer.any_released()) {
        ui.memory_mut(|m| m.stop_dragging());
        // The connection was dropped, take the sockets and check that they are compatible.
        if let Some((src, dst)) = drag.take_sockets() {
          if let Some((dst, dt)) = dst {
            // Connect.
            if let Err(err) = self.connect(src, dst, dt) {
              log::warn!("Failed to connect input[{src:?}] to output[{dst:?}]: {err:?}");
            }
          } else {
            // Disconnect
            if let Err(err) = self.disconnect(src) {
              log::warn!("Failed to disconnect input[{src:?}]: {err:?}");
            }
          }
        }
      } else if let Some(src) = &drag.src {
        ui.memory_mut(|m| m.set_dragged_id(id));
        // Still dragging a connection.
        let dst = if let Some(dst) = &drag.dst {
          // Hovering over the destination socket.
          ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
          Some((conn.to_ui_pos(dst.center), dst.color))
        } else if let Some(end) = ui.ctx().pointer_latest_pos() {
          ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
          // Try to scroll with the mouse pointer during the drag.
          if let Some(last) = drag.pointer_last_pos {
            let delta = (last - end) * 2.0;
            ui.scroll_with_delta(delta);
          }
          drag.pointer_last_pos = Some(end);
          Some((end, src.color))
        } else {
          None
        };
        if let Some((dst, color)) = dst {
          let (start, end, color) = if let Some(input_id) = src.id.as_input_id() {
            // If the dragged socket is an input, then remove it's current connection.
            if let Err(err) = self.disconnect(input_id) {
              log::warn!("Failed to disconnect input[{input_id:?}]: {err:?}");
            }
            (conn.to_ui_pos(src.center), dst, color)
          } else {
            // The dragged socket is an output.
            (dst, conn.to_ui_pos(src.center), color)
          };
          conn.draw(ui, start, end, Some(color), false);
        }
      }
    });

    // Draw connections.
    self.hover_connection = None;
    for (input, output) in &self.connections.0 {
      let meta = state.get_connection_meta(input, output);
      if let Some((in_meta, out_meta)) = meta {
        let start = conn.to_ui_pos(in_meta.center);
        let end = conn.to_ui_pos(out_meta.center);
        if conn.draw(ui, start, end, Some(out_meta.color), true).is_some() {
          self.hover_connection = Some(*input);
        }
      }
    }
  }
}

#[derive(Clone)]
#[cfg(feature = "egui")]
pub struct NodeGraphEditor {
  pub title: String,
  pub size: emath::Vec2,
  pub graph: NodeGraph,
}

#[cfg(feature = "egui")]
impl Default for NodeGraphEditor {
  fn default() -> Self {
    Self {
      title: "Graph editor".to_string(),
      size: (900., 500.).into(),
      graph: Default::default(),
    }
  }
}

#[cfg(feature = "egui")]
impl NodeGraphEditor {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
    let path = path.as_ref();
    let file = File::open(path)?;
    self.graph = serde_json::from_reader(file)?;
    Ok(())
  }

  pub fn show(&mut self, ctx: &egui::Context) {
    egui::Window::new(&self.title)
      .default_size(self.size)
      .show(ctx, |ui| {
        // HACK: Without this side panel, the central panel will not work with a ScrollArea.
        egui::SidePanel::right("graph_right_panel")
          .min_width(0.)
          .frame(egui::Frame::none())
          .show_separator_line(false)
          .show_inside(ui, |_ui| {});

        egui::CentralPanel::default().show_inside(ui, |ui| self.graph.ui(ui));
      });
  }
}
