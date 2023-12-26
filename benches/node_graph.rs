use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use node_engine::*;

fn bench_graph_dynamic_eval(c: &mut Criterion, graphes: &[(usize, NodeGraph, f32, bool)]) {
  let mut group = c.benchmark_group("Dynamic Eval graph");
  let mut execution = NodeGraphExecution::new();
  for (size, graph, expected, common_input) in graphes {
    group.bench_with_input(
      BenchmarkId::new("graph", format!("size: {}, common: {}", size, common_input)),
      graph,
      |b, graph| {
        b.iter(|| {
          let res = execution.eval_graph(&graph).unwrap();
          assert_eq!(res, Value::Scalar(*expected));
          res
        })
      },
    );
  }
  group.finish();
}

fn build_sub_graph(
  node: &Node,
  graph: &mut NodeGraph,
  depth: usize,
  input: &Input,
) -> (usize, NodeId) {
  let depth = depth - 1;
  let (sub_size, a, b) = if depth > 0 {
    let (a_size, a) = build_sub_graph(node, graph, depth, input);
    let (b_size, b) = build_sub_graph(node, graph, depth, input);
    (a_size + b_size, Input::from(a), Input::from(b))
  } else {
    (0, input.clone(), input.clone())
  };
  let id = graph.add(node.clone());
  graph.set_node_input(id, "A", a).expect("set input");
  graph.set_node_input(id, "B", b).expect("set input");

  (sub_size + 1, id)
}

fn build_graph(
  reg: &NodeRegistry,
  max_depth: usize,
  common_input: bool,
) -> (usize, NodeGraph, f32, bool) {
  let scalar = reg.new_by_name("Scalar Math").expect("Scalar math node");
  let mut graph = NodeGraph::new();

  let input = if common_input {
    let id = graph.add(scalar.clone());
    graph
      .set_node_input(id, "A", Input::from(1.0))
      .expect("set input");
    graph
      .set_node_input(id, "B", Input::from(0.0))
      .expect("set input");
    Input::from(id)
  } else {
    Input::from(1.0)
  };
  let (size, id) = build_sub_graph(&scalar, &mut graph, max_depth, &input);
  graph.set_output(Some(id));

  let expected = size as f32 + 1.0;
  (size, graph, expected, common_input)
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let reg = NodeRegistry::build();
  let graphes: Vec<_> = [
    (6, false),
    (8, false),
    (10, false),
    (18, false),
    (6, true),
    (8, true),
    (10, true),
    (18, true),
  ]
  .into_iter()
  .map(|(size, common_input)| build_graph(&reg, size, common_input))
  .collect();

  bench_graph_dynamic_eval(c, graphes.as_slice());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
