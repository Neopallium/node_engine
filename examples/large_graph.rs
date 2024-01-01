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
    (0, Input::from(1.0), Input::from(1.0))
  };
  let mut node = node.duplicate();
  node.set_position(position);
  let id = graph.add(node);
  graph.set_node_input(id, "A", a)?;
  graph.set_node_input(id, "B", b)?;

  Ok((sub_size + 1, id))
}

fn build_graph(reg: &NodeRegistry, max_depth: usize) -> anyhow::Result<(usize, NodeGraph, f32)> {
  let scalar = reg.new_by_name("Add").expect("add math node");
  let mut graph = NodeGraph::new();

  let position = [max_depth as f32 * X_OFFSET, 0.].into();
  let (size, id) = build_sub_graph(&scalar, &mut graph, position, max_depth)?;
  graph.set_output(Some(id));

  let expected = size as f32 + 1.0;
  Ok((size, graph, expected))
}

fn main() -> anyhow::Result<()> {
  let reg = NodeRegistry::build();
  let (size, graph, expected) = build_graph(&reg, 20)?;
  eprintln!("Graph size: {size}");

  let count = 10;
  let mut total = 0.0;

  let mut execution = NodeGraphExecution::new();
  for _ in 0..count {
    let res = execution.eval_graph(&graph)?;
    if let Value::F32(v) = &res {
      total += *v;
    }
    assert_eq!(res, Value::F32(expected));
  }

  eprintln!("total: {total}");
  assert_eq!(total, expected * count as f32);
  Ok(())
}
