use anyhow::Result;

use crate::*;

#[macro_export]
macro_rules! impl_dyn_vec_trinary_node {
  ( $mod_name:ident, $ty_name:ident, $docs:expr, $op:expr ) => {
    $crate::impl_dyn_vec_trinary_node!($mod_name, $ty_name, $docs, a, "Input `A`.", b, "Input `B`.", c, "Input `C`.", $op);
  };
  ( $mod_name:ident, $ty_name:ident, $docs:expr, $a:ident, $a_doc:expr, $b:ident, $b_doc:expr, $c:ident, $c_doc:expr, $op:expr ) => {
    $crate::impl_node! {
      mod $mod_name {
        NodeInfo {
          name: $ty_name,
          category: ["Math", "Basic"],
        }

        #[doc = $docs]
        #[derive(Default)]
        pub struct $ty_name {
          #[doc = $a_doc]
          pub $a: Input<DynamicVector>,
          #[doc = $b_doc]
          pub $b: Input<DynamicVector>,
          #[doc = $c_doc]
          pub $c: Input<DynamicVector>,
          /// Output `out`.
          pub out: Output<DynamicVector>,
        }

        impl $ty_name {
          pub fn new() -> Self {
            Default::default()
          }
        }

        impl NodeImpl for $ty_name {
          fn compile(
            &self,
            graph: &NodeGraph,
            compile: &mut NodeGraphCompile,
            id: NodeId,
          ) -> Result<()> {
            let (a, b, c) = self.resolve_inputs(graph, compile)?;
            let code = format!($op, a, b, c);
            self.out.compile(compile, id, stringify!($mod_name), code, a.dt)
          }
        }
      }
    }
  };
}
impl_dyn_vec_trinary_node!(
  lerp_node, LerpNode, "Linearly interpolating between inputs A and B by input T.",
  a, "Input A",
  b, "Input B",
  t, "Input T",
  "mix({}, {}, {})"
);
impl_dyn_vec_trinary_node!(
  clamp_node, ClampNode, "Clamp input between `min` and `max`.",
  input, "Unclamped input value",
  min, "Minimum value",
  max, "Maximum value",
  "clamp({}, {}, {})"
);

#[macro_export]
macro_rules! impl_dyn_vec_binary_node {
  ( $mod_name:ident, $ty_name:ident, $name:expr, $docs:expr, $op:expr ) => {
    $crate::impl_node! {
      mod $mod_name {
        NodeInfo {
          name: $ty_name,
          category: ["Math", "Basic"],
        }

        #[doc = $docs]
        #[derive(Default)]
        pub struct $ty_name {
          /// Input `A`.
          pub a: Input<DynamicVector>,
          /// Input `B`.
          pub b: Input<DynamicVector>,
          /// Output `out`.
          pub out: Output<DynamicVector>,
        }

        impl $ty_name {
          pub fn new() -> Self {
            Default::default()
          }
        }

        impl NodeImpl for $ty_name {
          fn compile(
            &self,
            graph: &NodeGraph,
            compile: &mut NodeGraphCompile,
            id: NodeId,
          ) -> Result<()> {
            let (a, b) = self.resolve_inputs(graph, compile)?;
            let code = format!($op, a, b);
            self.out.compile(compile, id, stringify!($mod_name), code, a.dt)
          }
        }
      }
    }
  };
}

impl_dyn_vec_binary_node!(add_node, AddNode, "Add", "Add two vectors.", "({} + {})");
impl_dyn_vec_binary_node!(subtract_node, SubtractNode, "Subtract", "Subtract two vectors.", "({} - {})");
impl_dyn_vec_binary_node!(divide_node, DivideNode, "Divide", "Divide two vectors.", "({} / {})");
impl_dyn_vec_binary_node!(power_node, PowerNode, "Power", "Output input `a` to the power of input `b`.", "pow({}, {})");
impl_dyn_vec_binary_node!(min_node, MinNode, "Minimum", "Output the smallest of two inputs.", "min({}, {})");
impl_dyn_vec_binary_node!(max_node, MaxNode, "Maximum", "Output the largest of two inputs.", "max({}, {})");

#[macro_export]
macro_rules! impl_dyn_vec_unary_node {
  ( $mod_name:ident, $ty_name:ident, $name:expr, $desp:expr, $op:expr ) => {
    $crate::impl_node! {
      mod $mod_name {
        NodeInfo {
          name: $name,
          description: $desp,
          category: ["Math", "Basic"],
        }

        #[doc = $desp]
        #[derive(Default)]
        pub struct $ty_name {
          /// Input `A`.
          pub a: Input<DynamicVector>,
          /// Output `out`.
          pub out: Output<DynamicVector>,
        }

        impl $ty_name {
          pub fn new() -> Self {
            Default::default()
          }
        }

        impl NodeImpl for $ty_name {
          fn compile(
            &self,
            graph: &NodeGraph,
            compile: &mut NodeGraphCompile,
            id: NodeId,
          ) -> Result<()> {
            let a = self.resolve_inputs(graph, compile)?;
            let code = format!($op, a);
            self.out.compile(compile, id, stringify!($mod_name), code, a.dt)
          }
        }
      }
    }
  };
}
impl_dyn_vec_unary_node!(sqrt_node, SquareRootNode, "Square Root", "Output the square root of input `a`.", "sqrt({})");
impl_dyn_vec_unary_node!(round_node, RoundNode, "Round", "Round input `a` to the nearest integer.", "round({})");
impl_dyn_vec_unary_node!(floor_node, FloorNode, "Floor", "Floor input `a`.", "floor({})");
impl_dyn_vec_unary_node!(fract_node, FractionNode, "Fraction", "Fraction input `a`.", "fract({})");
impl_dyn_vec_unary_node!(ceiling_node, CeilingNode, "Ceiling", "Ceiling input `a`.", "ceil({})");
impl_dyn_vec_unary_node!(truncate_node, TruncateNode, "Truncate", "Truncate input `a`.", "trunc({})");
impl_dyn_vec_unary_node!(absolute_node, AbsoluteNode, "Absolute", "Absolute input `a`.", "abs({})");

impl_node! {
  mod multiply_node {
    NodeInfo {
      name: "Multiply",
      category: ["Math", "Basic"],
    }

    /// Multiply vectors and matrixes.
    #[derive(Default)]
    pub struct MultiplyNode {
      /// Input `A`.
      pub a: Input<Dynamic>,
      /// Input `B`.
      pub b: Input<Dynamic>,
      /// Output.
      pub out: Output<Dynamic>,
    }

    impl MultiplyNode {
      pub fn new() -> Self {
        Default::default()
      }
    }

    impl NodeImpl for MultiplyNode {
      fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
        let (a, b) = self.resolve_inputs(graph, compile)?;
        let (code, out_dt) = match (a.dt.class(), b.dt.class()) {
          // Re-order so the vector is first.
          (DataTypeClass::Matrix, DataTypeClass::Vector) =>
            (format!("({b} * {a})"), b.dt),
          // default to using the type of `a`.
          _ => (format!("({a} * {b})"), a.dt),
        };
        self.out.compile(compile, id, "multiply_node", code, out_dt)
      }
    }
  }
}
