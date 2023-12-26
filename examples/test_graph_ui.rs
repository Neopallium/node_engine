use glam::Vec4;

use node_engine::ui::*;
use node_engine::*;

fn main() {
  env_logger::init();
  let native_options = eframe::NativeOptions {
    initial_window_size: Some(egui::vec2(1000., 600.)),
    ..Default::default()
  };
  eframe::run_native(
    "My egui App",
    native_options,
    Box::new(|cc| Box::new(MyEguiApp::new(cc))),
  )
  .expect("ok");
}

const X_OFFSET: f32 = 400.0;
const Y_OFFSET: f32 = 50.0;

fn build_sub_graph(
  node: &Node,
  graph: &mut NodeGraph,
  position: emath::Vec2,
  depth: usize,
) -> anyhow::Result<(usize, NodeId)> {
  let position = position - emath::vec2(X_OFFSET, 0.);
  let depth = depth - 1;
  let (sub_size, a, b) = if depth > 0 {
    let a_pos = position - emath::vec2(0., Y_OFFSET);
    let b_pos = position + emath::vec2(0., Y_OFFSET);
    let (a_size, a) = build_sub_graph(node, graph, a_pos, depth)?;
    let (b_size, b) = build_sub_graph(node, graph, b_pos, depth)?;
    (a_size + b_size, Input::from(a), Input::from(b))
  } else {
    (0, Input::from(Vec4::ONE), Input::from(Vec4::ONE))
  };
  let mut node = node.duplicate();
  node.frame_state_mut().selected = true;
  node.set_position(position);
  let id = graph.add(node);
  graph.set_node_input(id, "A", a)?;
  graph.set_node_input(id, "B", b)?;

  Ok((sub_size + 1, id))
}

fn build_graph(reg: &NodeRegistry, max_depth: usize) -> anyhow::Result<(usize, NodeGraph)> {
  let scalar = reg.new_by_name("Vec4 Math").expect("Scalar math node");
  let mut graph = NodeGraph::new();

  let position = [max_depth as f32 * X_OFFSET, 0.].into();
  let (size, id) = build_sub_graph(&scalar, &mut graph, position, max_depth)?;

  let mut frag = reg
    .new_by_name("Fragment output")
    .expect("Fragment output node");
  frag.set_position(position);
  let output_id = graph.add(frag);
  graph.set_node_input(output_id, "Color", Input::from(id))?;
  graph.set_output(Some(output_id));

  graph.group_selected_nodes();
  Ok((size, graph))
}

#[derive(Default)]
struct MyEguiApp {
  editor: NodeGraphEditor,
}

impl MyEguiApp {
  fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    let reg = NodeRegistry::build();
    eprintln!("Build shader graph");
    let (_size, graph) = build_graph(&reg, 2).expect("built graph");
    let mut editor = NodeGraphEditor::new();
    editor.graph = graph;
    Self { editor }
  }
}

impl eframe::App for MyEguiApp {
  fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    let json = serde_json::to_string_pretty(&self.editor.graph);
    match json {
      Ok(json) => {
        eprintln!("graph.json = {json}");
        /*
        // Try round-trip.
        let graph: NodeGraph = serde_json::from_str(&json).expect("should decode");
        eprintln!("graph = {graph:#?}");
        // */
      }
      Err(err) => eprintln!("Failed to encode: {err:?}"),
    }
  }

  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
      frame.close();
    }
    self.editor.show(ctx);
  }
}
