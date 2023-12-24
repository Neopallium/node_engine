use glam::{Vec2, Vec4};

use anyhow::Result;

use crate::*;

impl_node! {
  mod uv_node {
    NodeInfo {
      name: "UV Node",
      description: "Vertex or Fragment UV",
      categories: ["UV", "Shader"],
    }

    /// The vertex/fragment UV value.
    #[derive(Default)]
    pub struct UVNode {
      /// UV value.
      pub uv: Output<Vec2>,
    }

    impl UVNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for UVNode {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let block = compile.current_block()?;
        // TODO: add context lookups.
        block.append_output(id, "in.uv".to_string());
        Ok(())
      }
    }
  }
}

impl_node! {
  mod fragment_output_node {
    NodeInfo {
      name: "Fragment output",
      description: "Fragment shader node",
      categories: ["UV", "Shader", "Fragment"],
    }

    /// The fragment shader node.
    #[derive(Default)]
    pub struct FragmentOutputNode {
      /// Fragment color.
      pub color: Input<Vec4>,
    }

    impl FragmentOutputNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for FragmentOutputNode {
      fn eval(
        &self,
        graph: &NodeGraph,
        execution: &mut NodeGraphExecution,
        _id: NodeId,
      ) -> Result<Value> {
        self.color.eval(graph, execution).map(|v| v.to_value())
      }

      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, _id: NodeId) -> Result<()> {
        compile.append_code(
          "bindings",
          r#"
@group(1) @binding(0) var<uniform> material_color: vec4<f32>;
@group(1) @binding(1) var material_color_texture: texture_2d<f32>;
@group(1) @binding(2) var material_color_sampler: sampler;
"#
          .to_string(),
        )?;
        let frag_block = compile.push_new_block("fragment");
        {
          let block = compile.current_block()?;
          block.append(
            r##"
@fragment
fn fragment(
    in: bevy_pbr::forward_io::VertexOutput,
) -> @location(0) vec4<f32> {"##
              .to_string(),
          );
        }
        let color = self.color.compile(graph, compile)?;
        let block = compile.current_block()?;
        block.append(format!(
          r#"
  return {color};
}}
"#
        ));
        compile.pop(Some(frag_block))?;
        Ok(())
      }
    }
  }
}
