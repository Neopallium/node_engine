#[cfg(feature = "egui")]
use crate::ui::*;
use crate::GetId;

#[derive(Clone, Debug)]
pub struct NodeFrameStyle {
  pub fill: egui::Color32,
  pub selected: egui::Color32,
}

impl Default for NodeFrameStyle {
  fn default() -> Self {
    Self {
      fill: egui::Color32::from_gray(50),
      selected: egui::Color32::WHITE,
    }
  }
}

#[derive(Clone, Debug)]
pub enum NodeAction {
  Dragged(emath::Vec2),
  Resize,
  /// false - Only delete the node, true - Also delete contained nodes (for groups).
  Delete(bool),
  /// Remove a node from a group.
  LeaveGroup(Uuid),
}

#[derive(Clone, Debug)]
pub struct NodeFrameState {
  pub updated: bool,
  pub selected: bool,
  pub edit_title: bool,
  pub drag: Option<NodeFrameDragState>,
}

impl Default for NodeFrameState {
  fn default() -> Self {
    Self {
      updated: true,
      selected: false,
      edit_title: false,
      drag: None,
    }
  }
}

impl NodeFrameState {
  /// Return the updated state and clear it.
  pub fn take_updated(&mut self) -> bool {
    let updated = self.updated;
    self.updated = false;
    updated
  }

  pub fn is_dragging(&self) -> bool {
    self.drag == Some(NodeFrameDragState::Drag)
  }

  pub fn node_selected(&mut self, rect: emath::Rect, selecting: &NodeSelectingState) -> bool {
    match selecting {
      NodeSelectingState::Selecting {
        area, clear_old, ..
      } => {
        if self.selected && *clear_old {
          self.selected = false;
        }
        self.selected | area.intersects(rect)
      }
      NodeSelectingState::Select { area } => {
        if area.intersects(rect) {
          self.selected = true;
        }
        self.selected
      }
      _ => self.selected,
    }
  }

  /// Render the node.
  pub fn render<N: NodeFrame + GetId>(
    &mut self,
    ui: &mut egui::Ui,
    graph: &NodeGraphMeta,
    node: &mut N,
  ) -> Option<NodeAction> {
    node.render(ui, graph, self)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeFrameDragState {
  Drag,
  Resize(ResizeState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResizeState {
  top: bool,
  right: bool,
  bottom: bool,
  left: bool,
}

impl ResizeState {
  pub fn set_cursor(&self, ui: &egui::Ui) {
    if (self.top && self.left) || (self.bottom && self.right) {
      ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
    } else if (self.bottom && self.left) || (self.top && self.right) {
      ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
    } else if self.top || self.bottom {
      ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    } else if self.left || self.right {
      ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
  }

  pub fn resize_rect(&self, mut rect: emath::Rect, delta: emath::Vec2) -> emath::Rect {
    if self.top {
      rect.set_top(rect.top() + delta.y);
    }
    if self.right {
      rect.set_right(rect.right() + delta.x);
    }
    if self.bottom {
      rect.set_bottom(rect.bottom() + delta.y);
    }
    if self.left {
      rect.set_left(rect.left() + delta.x);
    }
    rect
  }
}

pub trait NodeFrame: GetId {
  /// Get the node's `rect` area.
  fn rect(&self) -> emath::Rect;
  /// Set the node's `rect` area.
  fn set_rect(&mut self, rect: emath::Rect);

  /// Get frame title.
  fn title(&self) -> &str;
  /// Set frame title.
  fn set_title(&mut self, title: String);

  /// Return the node's updated state and clear it.
  fn take_updated(&mut self, state: &mut NodeFrameState) -> bool {
    state.take_updated()
  }

  /// Frame style
  fn frame_style(&self) -> NodeFrameStyle {
    NodeFrameStyle::default()
  }

  /// Automatically fit the frame's contents.
  fn auto_size(&self) -> bool {
    false
  }

  /// Allow resizing.
  fn resizable(&self) -> bool {
    true
  }

  /// Allow moving.
  fn movable(&self) -> bool {
    true
  }

  /// Handle moving - either the frame is being dragged or it's parent group is moving.
  fn handle_move(&mut self, delta: emath::Vec2) {
    if self.movable() {
      self.set_rect(self.rect().translate(delta));
    }
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
    });
    action
  }

  /// Render the node.
  fn render(
    &mut self,
    ui: &mut egui::Ui,
    graph: &NodeGraphMeta,
    state: &mut NodeFrameState,
  ) -> Option<NodeAction> {
    let node_style = NodeStyle::get(ui);
    let zoom = node_style.zoom;
    // Zoom and translate frame to Screen space.
    let mut rect = self.rect();
    let updated = self.take_updated(state);
    if updated && self.auto_size() {
      // Recalculate size.
      rect.set_width(10.);
    }
    rect = graph.node_to_ui(rect);

    // Use child UI for frame.
    let mut child_ui = ui.child_ui_with_id_source(rect, *ui.layout(), self.id());
    let ui = &mut child_ui;

    // Allocate a response for the whole frame area.
    let resp = ui.interact(rect, ui.id(), egui::Sense::click_and_drag());

    // Only render this frame if it is visible or the frame was updated.
    if !updated && !ui.is_rect_visible(rect) {
      // This is needed to stabilize Ui ids when frames become visible.
      ui.skip_ahead_auto_ids(1);
      return None;
    }

    // Is the frame currently selected?
    let selected = graph.selecting(|selecting| state.node_selected(rect, selecting));

    // Render frame UI.
    self.frame_ui(ui, selected, state, node_style);

    // Handle events.
    if resp.clicked() {
      state.selected = !state.selected;
    } else if resp.dragged() {
      let delta = resp.drag_delta() / zoom;
      match state.drag.clone() {
        Some(NodeFrameDragState::Drag) => {
          self.handle_move(delta);
          resp.scroll_to_me(None);
        }
        Some(NodeFrameDragState::Resize(state)) => {
          self.set_rect(state.resize_rect(self.rect(), delta));
          resp.scroll_to_me(None);
          state.set_cursor(ui);
        }
        None => (),
      }
    } else if resp.drag_released() {
      state.drag = None;
    } else {
      // Get pointer.
      if let Some(pointer) = ui.ctx().pointer_interact_pos() {
        let style = ui.style();
        let side_grab_radius = style.interaction.resize_grab_radius_side;
        let corner_grab_radius = style.interaction.resize_grab_radius_corner;
        let inside = if self.resizable() {
          rect.shrink(side_grab_radius)
        } else {
          rect
        };
        // Drag the frame if inside the margin area.
        if inside.contains(pointer) {
          state.drag = Some(NodeFrameDragState::Drag);
          self.handle_move(resp.drag_delta() / zoom);
        } else if self.resizable() && rect.contains(pointer) {
          // Detect sides
          let mut top = (rect.top() - pointer.y).abs() <= side_grab_radius;
          let mut right = (rect.right() - pointer.x).abs() <= side_grab_radius;
          let mut bottom = (rect.bottom() - pointer.y).abs() <= side_grab_radius;
          let mut left = (rect.left() - pointer.x).abs() <= side_grab_radius;
          if top || right || bottom || left {
            // Detect corners.
            if rect.left_top().distance(pointer) < corner_grab_radius {
              left = true;
              top = true;
            }
            if rect.right_top().distance(pointer) < corner_grab_radius {
              right = true;
              top = true;
            }
            if rect.left_bottom().distance(pointer) < corner_grab_radius {
              left = true;
              bottom = true;
            }
            if rect.right_bottom().distance(pointer) < corner_grab_radius {
              right = true;
              bottom = true;
            }
            // Handle resize.
            let resize = ResizeState {
              top,
              right,
              bottom,
              left,
            };
            resize.set_cursor(ui);
            state.drag = Some(NodeFrameDragState::Resize(resize));
          }
        }
      }
    }
    let mut action = self.handle_resp(ui, resp, graph, state);
    if self.auto_size() {
      // Update frame size.
      let size = ui.min_rect().size() / zoom;
      let diff = (self.rect().size() - size).abs().max_elem();
      if diff >= 0.1 {
        eprintln!("update node size: old={:?}, new={:?}, diff={diff:?}", self.rect().size(), size);
        state.updated = true;
        if action.is_none() {
          action = Some(NodeAction::Resize);
        }
        self.set_rect(emath::Rect::from_min_size(self.rect().min, size));
      }
    }
    action
  }

  /// Draw the node's frame.
  fn frame_ui(
    &mut self,
    ui: &mut egui::Ui,
    selected: bool,
    state: &mut NodeFrameState,
    node_style: NodeStyle,
  ) {
    // Window-style frame.
    let style = ui.style();
    let frame_style = self.frame_style();
    let mut frame = egui::Frame::window(style);
    frame.shadow = Default::default();
    if selected {
      frame.stroke.color = frame_style.selected;
    }

    frame.fill(frame_style.fill).show(ui, |ui| {
      ui.vertical(|ui| {
        // Title bar.
        ui.horizontal(|ui| {
          if state.edit_title {
            let mut title = self.title().to_string();
            let resp = ui.add(egui::TextEdit::singleline(&mut title).hint_text("Group name"));
            if resp.changed() {
              self.set_title(title);
            }
            if resp.lost_focus() {
              state.edit_title = false;
            }
            resp.request_focus();
          } else {
            let rect = ui.available_rect_before_wrap();
            ui.label(self.title());
            // Manually detect click.  To fix issue with context menu.
            if ui.rect_contains_pointer(rect) {
              if ui.input(|i| {
                i.pointer
                  .button_double_clicked(egui::PointerButton::Primary)
              }) {
                state.edit_title = true;
              }
            }
          }
        });
        // Contents
        self.contents_ui(ui, node_style);
      });
    });
  }

  fn contents_ui(&mut self, ui: &mut egui::Ui, _node_style: NodeStyle) {
    ui.set_min_size(ui.available_size());
  }
}
