use glam::Vec4;

use node_engine::*;

const X_OFFSET: f32 = 250.0;
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
  node.set_position(position);
  let id = graph.add(node);
  graph.set_node_input(id, "A", a)?;
  graph.set_node_input(id, "B", b)?;

  Ok((sub_size + 1, id))
}

fn build_graph(reg: &NodeRegistry, max_depth: usize) -> anyhow::Result<(usize, NodeGraph)> {
  let scalar = reg.new_by_name("Add").expect("add math node");
  let mut graph = NodeGraph::new();

  let position = [max_depth as f32 * X_OFFSET, 0.].into();
  let (size, id) = build_sub_graph(&scalar, &mut graph, position, max_depth)?;

  let mut frag = reg.new_by_name("Fragment").expect("Fragment output node");
  frag.set_position(position);
  let output_id = graph.add(frag);
  graph.set_node_input(output_id, "Color", Input::from(id))?;
  graph.set_output(Some(output_id));

  Ok((size, graph))
}

fn main() -> anyhow::Result<()> {
  let reg = NodeRegistry::build();
  eprintln!("Build shader graph");
  let (size, graph) = build_graph(&reg, 3)?;
  eprintln!("Graph size: {size}");
  eprintln!("Compile shader");
  let mut compiler = NodeGraphCompile::new();
  compiler.define_block("imports");
  compiler.define_block("bindings");

  let frag_block = compiler.push_new_block("fragment");
  compiler.compile_graph(&graph)?;
  compiler.pop(Some(frag_block))?;

  eprintln!("Dump shader code:");
  let shader = compiler.dump();
  eprintln!("{}", shader);

  Ok(())
}
