use glam::{Vec2, Vec3};

use anyhow::Result;

use crate::*;

impl_node! {
  mod texture_sample {
    NodeInfo {
      name: "Texture Sample",
      category: ["Input"],
    }

    /// Texture sampler node.
    #[derive(Default)]
    pub struct TextureNode {
      /// UV.
      pub uv: Input<UV>,
      /// Texture. TODO: implement.
      pub tex: Input<Texture2DHandle>,
      /// RGBA value.
      pub rgba: Output<Color>,
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
        let code = format!(r#"
textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, {uv}, view.mip_bias)
"#);
        self.rgba.compile(compile, id, "texture_node", code, DataType::Vec4)
      }
    }
  }
}

impl_node! {
  mod view_direction_node {
    NodeInfo {
      name: "View direction",
      category: ["Input", "Geometry"],
    }

    /// Vertex or Fragment View Direction vector.
    #[derive(Default)]
    pub struct ViewDirectionNode {
      /// Coordinate space of view direction.
      pub space: Param<CoordSpace>,
      /// View Direction.
      pub view_direction: Output<Vec3>,
    }

    impl ViewDirectionNode {
      pub fn new() -> Self {
        Default::default()
      }

      fn world(&self, compile: &mut NodeGraphCompile) -> Result<String> {
        compile.add_local("world_view_dir", "in.world_position.xyz - view.world_position".into(), DataType::Vec3)
      }

      fn tangent(&self, compile: &mut NodeGraphCompile) -> Result<String> {
        let world = self.world(compile)?;
        compile.add_local("tangent_view_dir", format!(r#"vec3(
	dot({world}, in.world_tangent.xyz),
	dot({world}, normalize(cross(in.world_tangent.xyz, in.world_normal))),
	dot({world}, in.world_normal)
)"#), DataType::Vec3)
      }
    }

    impl NodeImpl for ViewDirectionNode {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let code = match self.space {
          CoordSpace::World => {
            self.world(compile)?
          }
          CoordSpace::Tangent => {
            self.tangent(compile)?
          }
          _ => "todo".to_string(),
        };
        // TODO: add context lookups.
        self.view_direction.compile(compile, id, "view_direction_node", code, DataType::Vec3)
      }
    }
  }
}

impl_node! {
  mod uv_node {
    NodeInfo {
      name: "UV",
      category: ["UV"],
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
        // TODO: add context lookups.
        self.uv.compile(compile, id, "uv_node", format!("in.uv"), DataType::Vec2)
      }
    }
  }
}

impl_node! {
  mod tiling_offset_node {
    NodeInfo {
      name: "Tiling and Offset",
      category: ["UV"],
    }

    /// Tiling and Offset Node.
    #[derive(Default)]
    pub struct TilingOffsetNode {
      /// Input UV.
      pub uv: Input<UV>,
      /// The tiling to apply.
      pub tiling: Input<Vec2>,
      /// The offset to apply.
      pub offset: Input<Vec2>,
      /// Output UV value.
      pub out: Output<Vec2>,
    }

    impl TilingOffsetNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for TilingOffsetNode {
      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let (uv, tiling, offset) = self.resolve_inputs(graph, compile)?;
        self.out.compile(compile, id, "tiling_offset_node", format!("fract({uv} * {tiling} + {offset})"), DataType::Vec2)
      }
    }
  }
}

impl_node! {
  mod fragment_output_node {
    NodeInfo {
      name: "Fragment",
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
          "imports",
          r#"
#import bevy_pbr::{
	pbr_fragment::pbr_input_from_standard_material,
	pbr_functions::alpha_discard,
	pbr_bindings,
	mesh_view_bindings::view,
	mesh_functions,
	skinning,
	view_transformations::position_world_to_clip,
}
#import bevy_render::instance_index::get_instance_index

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
	prepass_io::{Vertex, VertexOutput, FragmentOutput},
	pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
	forward_io::{Vertex, VertexOutput, FragmentOutput},
	pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
	pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif
"#
          .to_string(),
        )?;
        compile.append_code(
          "bindings",
          r#"
struct ShaderGraphMaterialUniform {
  prop_vec4: vec4<f32>,
};

@group(2) @binding(100) var<uniform> material: ShaderGraphMaterialUniform;
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
  v_in: VertexOutput,
  @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
	var in = v_in;

	// get PbrInput from StandardMaterial bindings.
	var pbr_input = pbr_input_from_standard_material(in, is_front);
"##
              .to_string(),
          );
        }
        let color = self.resolve_inputs(graph, compile)?;
        let block = compile.current_block()?;
        block.append(format!(r#"
  // Color from graph input `color`.
  pbr_input.material.base_color = {color};
"#));

        block.append(r#"
	// alpha discard
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
	// No lighting in deferred mode.
	let out = deferred_output(in, pbr_input);
#else
	var out: FragmentOutput;
  if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
		out.color = apply_pbr_lighting(pbr_input);
	} else {
		out.color = pbr_input.material.base_color;
  }

	// Apply PBR post processing.
	out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

  return out;
}
"#.to_string()
        );
        compile.pop(Some(frag_block))?;
        Ok(())
      }
    }
  }
}
