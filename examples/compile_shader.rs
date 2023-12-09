use anyhow::Result;

use glam::Vec4;

use node_engine::*;

fn build_sub_graph(
  node: &Box<dyn NodeImpl>,
  graph: &mut NodeGraph,
  depth: usize,
) -> Result<(usize, NodeId)> {
  let depth = depth - 1;
  let (sub_size, a, b) = if depth > 0 {
    let (a_size, a) = build_sub_graph(node, graph, depth)?;
    let (b_size, b) = build_sub_graph(node, graph, depth)?;
    (a_size + b_size, Input::from(a), Input::from(b))
  } else {
    (0, Input::from(Vec4::ONE), Input::from(Vec4::ONE))
  };
  let id = graph.add(node.clone());
  let node = graph.get_mut(id)?;
  node.set_input("A", a)?;
  node.set_input("B", b)?;

  Ok((sub_size + 1, id))
}

fn build_graph(reg: &NodeRegistry, max_depth: usize) -> Result<(usize, NodeGraph)> {
  let scalar = reg.new_by_name("Vec4 Math").expect("Scalar math node");
  let mut graph = NodeGraph::new();

  let (size, id) = build_sub_graph(&scalar, &mut graph, max_depth)?;

  let frag = reg
    .new_by_name("Fragment output")
    .expect("Fragment output node");
  let output_id = graph.add(frag);
  let node = graph.get_mut(output_id)?;
  node.set_input("Color", Input::from(id))?;
  graph.set_output(Some(output_id));

  Ok((size, graph))
}

fn main() -> anyhow::Result<()> {
  let reg = build_registry();
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
