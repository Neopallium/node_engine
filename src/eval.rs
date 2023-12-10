use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::graph::*;
use crate::node::*;
use crate::values::*;

#[derive(Clone, Default, Debug)]
pub enum NodeEvalState {
  #[default]
  Processing,
  Cached(Value),
}

#[derive(Clone, Default, Debug)]
pub struct NodeGraphExecution {
  nodes: HashMap<NodeId, NodeEvalState>,
}

impl NodeGraphExecution {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn clear(&mut self) {
    self.nodes.clear();
  }

  pub fn eval_graph(&mut self, graph: &NodeGraph) -> Result<Value> {
    self.clear();
    let id = graph
      .output()
      .ok_or_else(|| anyhow!("Graph missing output node"))?;
    self.eval_node(graph, id)
  }

  pub fn eval_node(&mut self, graph: &NodeGraph, id: NodeId) -> Result<Value> {
    let node = graph.get(id)?;
    if node.cache_output() {
      use std::collections::hash_map::Entry;
      // Check for cached value or recursive connections.
      match self.nodes.entry(id) {
        Entry::Occupied(entry) => match entry.get() {
          NodeEvalState::Processing => {
            Err(anyhow!("Recursive node connection"))?;
          }
          NodeEvalState::Cached(value) => {
            return Ok(value.clone());
          }
        },
        Entry::Vacant(entry) => {
          entry.insert(NodeEvalState::Processing);
        }
      }
      // Evaluate node.
      let value = node.eval(graph, self, id)?;
      // Cache results.
      self.nodes.insert(id, NodeEvalState::Cached(value.clone()));
      Ok(value)
    } else {
      // Evaluate node.
      node.eval(graph, self, id)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::build_registry;

  #[test]
  fn compile_graph() -> Result<()> {
    println!("Build node registry");
    let reg = build_registry();
    println!("Build node graph");
    let mut graph = NodeGraph::new();
    let scalar = reg.new_by_name("Scalar Math").expect("Scalar Math node");
    let node1 = graph.add(scalar.duplicate());
    graph.set_node_input(node1, "A", 1.0.into())?;
    graph.set_node_input(node1, "B", 2.0.into())?;
    let node2 = graph.add(scalar.duplicate());
    graph.set_node_input(node2, "A", node1.into())?;
    graph.set_node_input(node2, "B", 2.0.into())?;
    let node3 = graph.add(scalar.duplicate());
    graph.set_node_input(node3, "A", node1.into())?;
    graph.set_node_input(node3, "B", node2.into())?;
    graph.set_output(Some(node3));

    println!("Dynamic eval graph (no compile)");
    let mut execution = NodeGraphExecution::new();
    let val = execution.eval_graph(&graph)?;
    println!("eval val={val:?}");
    assert_eq!(val, Value::Scalar(8.0));
    Ok(())
  }
}
