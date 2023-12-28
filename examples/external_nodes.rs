use anyhow::Result;

use glam::{Vec2, Vec4};

use node_engine::*;

impl_node! {
  mod custom_node {
    NodeInfo {
      name: "Custom external Node",
      description: "Vertex or Fragment UV",
      category: ["External"],
    }

    /// The vertex/fragment UV value.
    #[derive(Default)]
    pub struct CustomNode {
      /// Fragment color.
      pub color: Input<Vec4>,
      /// UV value.
      pub uv: Output<Vec2>,
    }

    impl CustomNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for CustomNode {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let block = compile.current_block()?;
        // TODO: add context lookups.
        block.append_output(id, "in.uv".to_string());
        Ok(())
      }
    }
  }
}

fn main() -> anyhow::Result<()> {
  let mut node = CustomNode::new();
  node.set_input("Color", Vec4::new(1.0, 2.0, 3.0, 1.0).into())?;

  Ok(())
}
