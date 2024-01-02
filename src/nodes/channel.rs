use glam::{Vec2, Vec3, Vec4};

use anyhow::Result;

use crate::*;

impl_node! {
  mod split_node {
    NodeInfo {
      name: "Split",
      category: ["Channel"],
    }

    /// Split the input vector into it's components.
    #[derive(Default)]
    pub struct SplitNode {
      pub input: Input<DynamicVector>,
      /// Output R.
      pub r: Output<f32>,
      /// Output G.
      pub g: Output<f32>,
      /// Output B.
      pub b: Output<f32>,
      /// Output A.
      pub a: Output<f32>,
    }

    impl SplitNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for SplitNode {
      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let input = self.resolve_inputs(graph, compile)?;
        let (r, g, b, a) = match input.dt {
          DataType::F32 => {
            (format!("{input}"), "0.".to_string(), "0.".to_string(), "0.".to_string())
          },
          DataType::Vec2 => {
            (format!("{input}.r"), format!("{input}.g"), "0.".to_string(), "0.".to_string())
          },
          DataType::Vec3 => {
            (format!("{input}.r"), format!("{input}.g"), format!("{input}.b"), "0.".to_string())
          },
          DataType::Vec4 => {
            (format!("{input}.r"), format!("{input}.g"), format!("{input}.b"), format!("{input}.a"))
          },
          _ => {
            return Err(anyhow::anyhow!("Unsupported input data type: {input:?}"));
          }
        };
        self.r.compile(compile, id, "split_node_r", r, DataType::F32)?;
        self.g.compile(compile, id, "split_node_g", g, DataType::F32)?;
        self.b.compile(compile, id, "split_node_b", b, DataType::F32)?;
        self.a.compile(compile, id, "split_node_a", a, DataType::F32)?;
        Ok(())
      }
    }
  }
}

impl_node! {
  mod swizzle_node {
    NodeInfo {
      name: "Swizzle",
      category: ["Channel"],
    }

    /// Swizzle the input vector into it's components.
    #[derive(Default)]
    pub struct SwizzleNode {
      /// Input value.
      pub input: Input<DynamicVector>,
      /// Swizzle mask.
      pub swizzle: Param<SwizzleMask>,
      /// Output value.
      pub out: Output<DynamicVector>,
    }

    impl SwizzleNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for SwizzleNode {
      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let input = self.resolve_inputs(graph, compile)?;
        let out = self.swizzle.compile(input)?;
        self.out.compile(compile, id, "swizzle_node", out.value, out.dt)?;
        Ok(())
      }
    }
  }
}

impl_node! {
  mod combine_node {
    NodeInfo {
      name: "Combine",
      category: ["Channel"],
    }

    /// Combine components into a vector.
    #[derive(Default)]
    pub struct CombineNode {
      /// Input R.
      pub r: Input<f32>,
      /// Input G.
      pub g: Input<f32>,
      /// Input B.
      pub b: Input<f32>,
      /// Input A.
      pub a: Input<f32>,
      /// Output RGBA.
      pub rgba: Output<Vec4>,
      /// Output RGB.
      pub rgb: Output<Vec3>,
      /// Output RG.
      pub rg: Output<Vec2>,
    }

    impl CombineNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for CombineNode {
      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let (r, g, b, a) = self.resolve_inputs(graph, compile)?;
        self.rgba.compile(compile, id, "combine_node_rgba", format!("vec4<f32>({r}, {g}, {b}, {a})"), DataType::Vec4)?;
        self.rgb.compile(compile, id, "combine_node_rgb", format!("vec3<f32>({r}, {g}, {b})"), DataType::Vec3)?;
        self.rg.compile(compile, id, "combine_node_rg", format!("vec2<f32>({r}, {g})"), DataType::Vec2)?;
        Ok(())
      }
    }
  }
}

