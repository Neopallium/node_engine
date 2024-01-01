use glam::{Vec2, Vec4};

use anyhow::Result;

use crate::*;

impl_node! {
  mod texture_sample {
    NodeInfo {
      name: "Texture Sample",
      description: "Texture sampler",
      category: ["Input"],
    }

    /// Texture node.
    #[derive(Default)]
    pub struct TextureNode {
      /// UV.
      pub uv: Input<UV>,
      /// Texture.
      pub tex: Input<Texture2DHandle>,
      /// RGBA value.
      pub rgba: Output<Vec4> Color(0xFFFFFF),
    }

    impl TextureNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for TextureNode {
      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let (uv, _tex) = self.resolve_inputs(graph, compile)?;
        // TODO: add context lookups.
        let code = format!("textureSample(texture, texture_sampler, {uv})");
        compile.add_output(id.into(), "texture_node", code, DataType::Vec4)
      }
    }
  }
}

impl_node! {
  mod uv_node {
    NodeInfo {
      name: "UV",
      description: "Vertex or Fragment UV",
      category: ["UV"],
    }

    /// The vertex/fragment UV value.
    #[derive(Default)]
    pub struct UVNode {
      /// UV Channel.
      pub channel: Param<UvChannel>,
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
        // TODO: add context lookups.
        compile.add_output(id.into(), "uv_node", format!("in.uv"), DataType::Vec2)
      }
    }
  }
}

impl_node! {
  mod fragment_output_node {
    NodeInfo {
      name: "Fragment",
      description: "Fragment shader node",
      category: ["Output"],
    }

    /// The fragment shader node.
    #[derive(Default)]
    pub struct FragmentOutputNode {
      /// Fragment color.
      pub color: Input<Color>,
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
        let color = self.resolve_inputs(graph, compile)?;
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
