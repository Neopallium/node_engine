use glam::{Vec2, Vec3, Vec4};

use anyhow::Result;

use crate::*;

impl_node! {
  mod bool_node {
    NodeInfo {
      name: "Boolean",
      category: ["Input", "Basic"],
    }

    pub enum Boolean {
      False,
      True,
    }

    /// A constant boolean value.
    #[derive(Default)]
    pub struct BoolNode {
      pub value: Param<Boolean>,
      /// Output.
      pub out: Output<f32>,
    }

    impl BoolNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for BoolNode {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let value = match self.value {
          Boolean::False => "0.0",
          Boolean::True => "1.0",
        }.to_string();
        self.out.compile(compile, id, "bool_node", value, DataType::F32)
      }
    }
  }
}

impl_node! {
  mod color_node {
    NodeInfo {
      name: "Color",
      category: ["Input", "Basic"],
    }

    /// A constant color value.
    #[derive(Default)]
    pub struct ColorNode {
      pub value: Param<Color>,
      /// Output.
      pub out: Output<Color>,
    }

    impl ColorNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for ColorNode {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let value = self.value.compile()?.to_string();
        self.out.compile(compile, id, "color_node", value, DataType::Vec4)
      }
    }
  }
}

impl_node! {
  mod float_node {
    NodeInfo {
      name: "Float",
      category: ["Input", "Basic"],
    }

    /// A constant float value.
    #[derive(Default)]
    pub struct FloatNode {
      pub value: Param<f32>,
      /// Output.
      pub out: Output<f32>,
    }

    impl FloatNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for FloatNode {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let value = self.value.compile()?.to_string();
        self.out.compile(compile, id, "float_node", value, DataType::F32)
      }
    }
  }
}

impl_node! {
  mod vector2_node {
    NodeInfo {
      name: "Vector 2",
      category: ["Input", "Basic"],
    }

    /// A constant Vec2 value.
    #[derive(Default)]
    pub struct Vec2Node {
      pub value: Param<Vec2>,
      /// Output.
      pub out: Output<Vec2>,
    }

    impl Vec2Node {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for Vec2Node {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let value = self.value.compile()?.to_string();
        self.out.compile(compile, id, "vector2_node", value, DataType::Vec2)
      }
    }
  }
}

impl_node! {
  mod vector3_node {
    NodeInfo {
      name: "Vector 3",
      category: ["Input", "Basic"],
    }

    /// A constant Vec3 value.
    #[derive(Default)]
    pub struct Vec3Node {
      pub value: Param<Vec3>,
      /// Output.
      pub out: Output<Vec3>,
    }

    impl Vec3Node {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for Vec3Node {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let value = self.value.compile()?.to_string();
        self.out.compile(compile, id, "vector3_node", value, DataType::Vec3)
      }
    }
  }
}

impl_node! {
  mod vector4_node {
    NodeInfo {
      name: "Vector 4",
      category: ["Input", "Basic"],
    }

    /// A constant Vec4 value.
    #[derive(Default)]
    pub struct Vec4Node {
      pub value: Param<Vec4>,
      /// Output.
      pub out: Output<Vec4>,
    }

    impl Vec4Node {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for Vec4Node {
      fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let value = self.value.compile()?.to_string();
        self.out.compile(compile, id, "vector4_node", value, DataType::Vec4)
      }
    }
  }
}

