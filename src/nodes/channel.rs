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
        self.r.compile(compile, id, "split_node", format!("{input}.r"), DataType::F32)?;
        self.g.compile(compile, id, "split_node", format!("{input}.g"), DataType::F32)?;
        self.b.compile(compile, id, "split_node", format!("{input}.b"), DataType::F32)?;
        self.a.compile(compile, id, "split_node", format!("{input}.a"), DataType::F32)?;
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
        self.rgba.compile(compile, id, "combine_node", format!("vec4<f32>({r}, {g}, {b}, {a})"), DataType::Vec4)?;
        self.rgb.compile(compile, id, "combine_node", format!("vec3<f32>({r}, {g}, {b})"), DataType::Vec3)?;
        self.rg.compile(compile, id, "combine_node", format!("vec2<f32>({r}, {g})"), DataType::Vec2)?;
        Ok(())
      }
    }
  }
}
