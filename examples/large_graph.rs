use anyhow::Result;

use node_engine::*;

fn build_sub_graph(
  node: &NodeState,
  graph: &mut NodeGraph,
  depth: usize,
) -> Result<(usize, NodeId)> {
  let depth = depth - 1;
  let (sub_size, a, b) = if depth > 0 {
    let (a_size, a) = build_sub_graph(node, graph, depth)?;
    let (b_size, b) = build_sub_graph(node, graph, depth)?;
    (a_size + b_size, Input::from(a), Input::from(b))
  } else {
    (0, Input::from(1.0), Input::from(1.0))
  };
  let id = graph.add(node.clone());
  let node = graph.get_mut(id)?;
  node.set_input("A", a)?;
  node.set_input("B", b)?;

  Ok((sub_size + 1, id))
}

fn build_graph(reg: &NodeRegistry, max_depth: usize) -> Result<(usize, NodeGraph, f32)> {
  let scalar = reg.new_by_name("Scalar Math").expect("Scalar math node");
  let mut graph = NodeGraph::new();

  let (size, id) = build_sub_graph(&scalar, &mut graph, max_depth)?;
  graph.set_output(Some(id));

  let expected = size as f32 + 1.0;
  Ok((size, graph, expected))
}

fn main() -> anyhow::Result<()> {
  let reg = build_registry();
  let (size, graph, expected) = build_graph(&reg, 20)?;
  eprintln!("Graph size: {size}");

  let count = 10;
  let mut total = 0.0;

  let mut execution = NodeGraphExecution::new();
  for _ in 0..count {
    let res = execution.eval_graph(&graph)?;
    if let Value::Scalar(v) = &res {
      total += *v;
    }
    assert_eq!(res, Value::Scalar(expected));
  }

  eprintln!("total: {total}");
  assert_eq!(total, expected * count as f32);
  Ok(())
}
