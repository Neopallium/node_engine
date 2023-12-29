use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

pub type NodeGroupId = Uuid;

pub const NODE_GROUP_MARGIN: f32 = 50.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeGroup {
  pub id: NodeGroupId,
  title: String,
  area: emath::Rect,
}

impl GetId for NodeGroup {
  fn id(&self) -> Uuid {
    self.id
  }
}

impl NodeGroup {
  pub fn new() -> Self {
    Self {
      id: Uuid::new_v4(),
      title: "".to_string(),
      area: emath::Rect::NOTHING,
    }
  }

  pub fn set_area(&mut self, area: emath::Rect) {
    self.area = area.expand(NODE_GROUP_MARGIN);
  }

  pub fn add_node(&mut self, node: &mut Node) {
    node.group_id = self.id;
    self.area = self.area.union(node.rect().expand(NODE_GROUP_MARGIN));
  }
}

#[cfg(feature = "egui")]
impl NodeFrame for NodeGroup {
  fn title(&self) -> &str {
    &self.title
  }

  fn set_title(&mut self, title: String) {
    self.title = title;
  }

  fn rect(&self) -> emath::Rect {
    self.area
  }

  fn set_rect(&mut self, rect: emath::Rect) {
    self.area = rect;
  }

  /// Frame style
  fn frame_style(&self) -> NodeFrameStyle {
    NodeFrameStyle {
      fill: egui::Color32::from_gray(10),
      ..Default::default()
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
      if ui.button("Delete group").clicked() {
        action = Some(NodeAction::Delete(false));
        ui.close_menu();
      }
      if ui.button("Delete group and nodes").clicked() {
        action = Some(NodeAction::Delete(true));
        ui.close_menu();
      }
    });
    action
  }
}
