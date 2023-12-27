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

  /// Frame style
  fn frame_style(&self) -> NodeFrameStyle {
    NodeFrameStyle::default()
  }

  /// Frame UI state.
  fn frame_state(&self) -> &NodeFrameState;
  fn frame_state_mut(&mut self) -> &mut NodeFrameState;

  /// Automatically fit the frame's contents.
  fn auto_size(&self) -> bool {
    false
  }

  /// Allow resizing.
  fn resizable(&self) -> bool {
    true
  }

  /// Allow dragging.
  fn draggable(&self) -> bool {
    true
  }

  /// Force draw, even if not visible.
  fn updated(&self) -> bool {
    self.frame_state().updated
  }

  /// Frame is selected.
  fn selected(&self) -> bool {
    self.frame_state().selected
  }

  /// Set frame selected state.
  fn set_selected(&mut self, selected: bool) -> bool {
    let state = self.frame_state_mut();
    let old = state.selected;
    state.selected = selected;
    old
  }

  /// Handle drag events from other UI responses.
  /// This is mainly to handle drag events from the title bar.
  fn handle_dragged(&mut self, resp: &egui::Response, zoom: f32) {
    if self.draggable() && resp.dragged() {
      self.set_rect(self.rect().translate(resp.drag_delta() / zoom));
      resp.scroll_to_me(None);
    }
  }

  /// Handle other events.
  fn handle_resp(&mut self, _ui: &egui::Ui, _resp: &egui::Response) {}

  /// Render the node.
  fn render(&mut self, ui: &mut egui::Ui, offset: egui::Vec2) -> egui::Response {
    let node_style = ui.node_style();
    let zoom = node_style.zoom;
    // Zoom and translate frame to Screen space.
    let mut rect = self.rect();
    if self.updated() && self.auto_size() {
      // Recalculate size.
      rect.set_width(10.);
    }
    rect.zoom(zoom);
    rect = rect.translate(offset);

    // Use child UI for frame.
    let mut child_ui = ui.child_ui_with_id_source(rect, *ui.layout(), self.id());
    let ui = &mut child_ui;

    // Allocate a response for the whole frame area.
    let resp = ui.interact(rect, ui.id(), egui::Sense::click_and_drag());

    // Only render this frame if it is visible or the frame was updated.
    if !self.updated() && !ui.is_rect_visible(rect) {
      // This is needed to stabilize Ui ids when frames become visible.
      ui.skip_ahead_auto_ids(1);
      return resp;
    }
    self.frame_state_mut().updated = false;

    // Render frame UI.
    self.frame_ui(ui, node_style);

    // Handle events.
    if resp.clicked() {
      let state = self.frame_state_mut();
      state.selected = !state.selected;
    } else if resp.dragged() {
      match self.frame_state().drag.clone() {
        Some(NodeFrameDragState::Drag) => {
          self.handle_dragged(&resp, zoom);
        }
        Some(NodeFrameDragState::Resize(state)) => {
          self.set_rect(state.resize_rect(self.rect(), resp.drag_delta() / zoom));
          resp.scroll_to_me(None);
          state.set_cursor(ui);
        }
        None => (),
      }
    } else if resp.drag_released() {
      self.frame_state_mut().drag = None;
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
          self.frame_state_mut().drag = Some(NodeFrameDragState::Drag);
          self.handle_dragged(&resp, zoom);
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
            let state = ResizeState {
              top,
              right,
              bottom,
              left,
            };
            state.set_cursor(ui);
            self.frame_state_mut().drag = Some(NodeFrameDragState::Resize(state));
          }
        }
      }
    }
    self.handle_resp(ui, &resp);
    if self.auto_size() {
      // Update frame size.
      let size = ui.min_rect().size() / zoom;
      if self.rect().size() != size {
        self.frame_state_mut().updated = true;
        self.set_rect(emath::Rect::from_min_size(self.rect().min, size));
      }
    }
    resp
  }

  /// Draw the node's frame.
  fn frame_ui(&mut self, ui: &mut egui::Ui, node_style: NodeStyle) {
    // Window-style frame.
    let style = ui.style();
    let frame_style = self.frame_style();
    let mut frame = egui::Frame::window(style);
    frame.shadow = Default::default();
    if self.selected() {
      frame.stroke.color = frame_style.selected;
    }

    frame.fill(frame_style.fill).show(ui, |ui| {
      ui.vertical(|ui| {
        // Title bar.
        ui.horizontal(|ui| {
          let state = self.frame_state();
          if state.edit_title {
            let mut title = self.title().to_string();
            let resp = ui.add(egui::TextEdit::singleline(&mut title).hint_text("Group name"));
            if resp.changed() {
              self.set_title(title);
            }
            if resp.lost_focus() {
              self.frame_state_mut().edit_title = false;
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
                self.frame_state_mut().edit_title = true;
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
