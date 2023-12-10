#[macro_export]
macro_rules! impl_node {
  (
    mod $mod_name:ident {
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        {}
        {}
        []
        []
        []
        []
        {}
        {}
        {}
        $($rest)*
      }
    }
  };
  // Parse NodeInfo
  (@normalize
    mod $mod_name:ident {
      {}
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      NodeInfo {
        $($node_info:tt)*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        $($rest)*
      }
    }
  };
  // Parse Parameter enum types.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      $(#[$param_enum_meta:meta])*
      pub enum $param_enum_name:ident {
        $(
          $(#[$variant_meta:meta])*
          $variant_name:ident
        ),+ $(,)?
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        { $( $node_info )* }
        {
          $( $extra_code )*

          $(#[$param_enum_meta])*
          #[derive(Copy, Clone, Debug, Default)]
          #[derive(serde::Serialize, serde::Deserialize)]
          pub enum $param_enum_name {
            #[default]
            $(
              $(#[$variant_meta])*
              $variant_name
            ),+
          }

          impl ParameterType for $param_enum_name {
            fn get_param(&self) -> ParameterValue {
              ParameterValue::Selected(format!("{self:?}"))
            }

            fn set_param(&mut self, value: ParameterValue) -> Result<()> {
              match value {
                ParameterValue::Selected(val) => match val.as_str() {
                  $(
                    stringify!($variant_name) => {
                      *self = Self::$variant_name;
                      Ok(())
                    }
                  ),+
                  _ => Err(anyhow::anyhow!("Invalid {val} variant."))
                }
                _ => {
                  Err(anyhow::anyhow!("Unsupport ParameterValue -> Enum conversion."))
                }
              }
            }

            fn parameter_data_type() -> ParameterDataType {
              ParameterDataType::select(&[
                $(
                  stringify!($variant_name)
                ),+
              ])
            }
          }
        }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        $($rest)*
      }
    }
  };
  // Parse Node struct.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      []
      {}
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      $(#[$node_struct_attr:meta])*
      pub struct $node_ty_name:ident {
        $($unparsed_fields:tt)*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize_fields
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        []
        {
          $(#[$node_struct_attr])*
          pub struct $node_ty_name;
        }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        ___internal_parse_fields {
            $($unparsed_fields)*
        }
        $($rest)*
      }
    }
  };
  // Parse Node input field.
  (@normalize_fields
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields {
          $( #[$field_meta:meta] )*
          $field_vis:vis $field_name:ident : Input<$field_ty:ident>,
          $($unparsed_fields:tt)*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize_fields
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [
          $( $node_inputs )*
          $field_name : $field_ty,
        ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [
          $( $node_struct_fields )*
          $( #[$field_meta] )*
          $field_vis $field_name : InputTyped<$field_ty>,
        ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        ___internal_parse_fields {
            $($unparsed_fields)*
        }
        $($rest)*
      }
    }
  };
  // Parse Node parameter field.
  (@normalize_fields
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields {
          $( #[$field_meta:meta] )*
          $field_vis:vis $field_name:ident : Param<$field_ty:ident>,
          $($unparsed_fields:tt)*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize_fields
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [
          $( $node_parameters )*
          $field_name : $field_ty,
        ]
        [ $( $node_outputs )* ]
        [
          $( $node_struct_fields )*
          $( #[$field_meta] )*
          $field_vis $field_name : $field_ty,
        ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        ___internal_parse_fields {
            $($unparsed_fields)*
        }
        $($rest)*
      }
    }
  };
  // Parse Node output field.
  (@normalize_fields
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields {
          $( #[$field_meta:meta] )*
          $field_vis:vis $field_name:ident : Output<$field_ty:ident>,
          $($unparsed_fields:tt)*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize_fields
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [
          $( $node_outputs )*
          $field_name : $field_ty,
        ]
        [
          $( $node_struct_fields )*
          $( #[$field_meta] )*
          #[serde(skip)]
          $field_vis $field_name : OutputTyped<$field_ty>,
        ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        ___internal_parse_fields {
            $($unparsed_fields)*
        }
        $($rest)*
      }
    }
  };
  // Parse Node private field.
  (@normalize_fields
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields {
          $( #[$field_meta:meta] )*
          $field_vis:vis $field_name:ident : $field_ty:ident,
          $($unparsed_fields:tt)*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize_fields
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [
          $( $node_struct_fields )*
          $( #[$field_meta] )*
          $field_vis $field_name : $field_ty,
        ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        ___internal_parse_fields {
            $($unparsed_fields)*
        }
        $($rest)*
      }
    }
  };
  // Finished parsing Node fields
  (@normalize_fields
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields { }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        $($rest)*
      }
    }
  };
  // Parse Node impl.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      $(#[$node_impl_meta:meta])*
      impl $node_ty_name:ident {
        $( $ty_impl_fns:tt )*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        {
          $( $node_impl )*

          $(#[$node_impl_meta])*
          impl $node_ty_name {
            $( $ty_impl_fns )*
          }
        }
        { $( $node_trait_impl )* }
        $($rest)*
      }
    }
  };
  // Parse Node trait impl.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      $(#[$node_impl_meta:meta])*
      impl NodeImpl for $node_ty_name:ident {
        $( $ty_node_impl_fns:tt )*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        {
          $(#[$node_impl_meta])*
          impl NodeImpl for $node_ty_name {
            $( $ty_node_impl_fns )*
          }
        }
        $($rest)*
      }
    }
  };
  // Parse Node trait impls.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      $(#[$node_impl_meta:meta])*
      impl $impl_trait_name:ident for $node_ty_name:ident {
        $( $ty_impl_fns:tt )*
      }
      $($rest:tt)*
    }
  ) => {
    $crate::impl_node! {
      @normalize
      mod $mod_name {
        { $( $node_info )* }
        { $( $extra_code )* }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        {
          $( $node_impl )*

          $(#[$node_impl_meta])*
          impl $impl_trait_name for $node_ty_name {
            $( $ty_impl_fns )*
          }
        }
        { $( $node_trait_impl )* }
        $($rest)*
      }
    }
  };
  // Final matcher.
  (@normalize
    mod $mod_name:ident {
      {
        name: $node_name:literal
        $(
          , description: $node_description:literal
        )?
        $(
          , categories: $node_categories:expr
        )?
        $(
          , custom: {
            $( $custom_field_name:ident: $custom_field_value:literal ),*
            $(,)?
          }
        )?
        $(,)?
      }
      { $( $extra_code:tt )* }
      [ $( $field_input_name:ident: $field_input_ty:ident, )* ]
      [ $( $field_param_name:ident: $field_param_ty:ident, )* ]
      [ $( $field_output_name:ident: $field_output_ty:ident, )* ]
      [ $( $node_struct_fields:tt )* ]
      {
        $(#[$node_struct_attr:meta])*
        pub struct $node_ty_name:ident;
      }
      { $( $node_impl:tt )* }
      {
        $(#[$node_impl_meta:meta])*
        impl NodeImpl for $node_impl_ty_name:ident {
          $( $ty_node_impl_fns:tt )*
        }
      }
      $($rest:tt)*
    }
  ) => {
    mod $mod_name {
      use super::*;

      lazy_static::lazy_static! {
        pub static ref DEFINITION: $crate::NodeDefinition = {
          let mut def = $crate::NodeDefinition::default();
          def.name = $node_name.to_string();
          $( def.description = $node_description.to_string(); )?
          $(
            def.categories = $node_categories.iter().map(|c| c.to_string()).collect();
          )?
          def.uuid = uuid::Uuid::new_v5(&$crate::node::NAMESPACE_NODE_IMPL, $node_name.as_bytes());
          def.inputs = [
            $( InputDefinition::typed::<$field_input_ty>(stringify!($field_input_name)) ),*
          ].into();
          def.parameters = [
            $( ParameterDefinition::typed::<$field_param_ty>(stringify!($field_param_name)) ),*
          ].into();
          def.outputs = [
            $( OutputDefinition::typed::<$field_output_ty>(stringify!($field_output_name)) ),*
          ].into();
          $(
            $(
              def.custom.insert(stringify!($custom_field_name).to_string(), $custom_field_value.to_string());
            )*
          )?
          def.builder = std::sync::Arc::new(Box::new(Builder));

          def
        };
      }

      #[derive(Clone, Debug, Default)]
      #[derive(serde::Serialize, serde::Deserialize)]
      struct Builder;

      #[typetag::serde]
      impl NodeBuilder for Builder {
        fn new_node(&self, _def: &NodeDefinition) -> Box<dyn NodeImpl> {
          Box::new($node_ty_name::new())
        }
      }

      $( $extra_code )*

      $(#[$node_struct_attr])*
      #[derive(Clone, Debug)]
      #[derive(serde::Serialize, serde::Deserialize)]
      pub struct $node_ty_name {
        $( $node_struct_fields )*
      }

      impl GetNodeDefinition for $node_ty_name {
        fn node_definition() -> NodeDefinition {
          DEFINITION.clone()
        }
      }

      $( $node_impl )*

      $(#[$node_impl_meta])*
      #[typetag::serde]
      impl NodeImpl for $node_ty_name {
        fn clone_node(&self) -> Box<dyn $crate::NodeImpl> {
          Box::new(self.clone())
        }

        fn def(&self) -> &$crate::NodeDefinition {
          &DEFINITION
        }

        fn get_node_input(&self, idx: &InputKey) -> Result<Input> {
          match self.def().get_input(idx) {
            Some(input) => match input.field_name.as_str() {
              $(
                stringify!($field_input_name) => {
                  Ok(self.$field_input_name.as_input())
                }
              )*
              field_name => Err(anyhow::anyhow!("Unknown input field: {field_name}")),
            }
            _ => Err(anyhow::anyhow!("Unknown input: {idx:?}")),
          }
        }

        fn set_node_input(&mut self, idx: &InputKey, _value: Input) -> Result<Option<OutputId>> {
          match self.def().get_input(idx) {
            Some(input) => match input.field_name.as_str() {
              $(
                stringify!($field_input_name) => {
                  self.$field_input_name.set_input(_value)
                }
              )*
              field_name => Err(anyhow::anyhow!("Unknown input field: {field_name}")),
            }
            _ => Err(anyhow::anyhow!("Unknown input: {idx:?}")),
          }
        }

        fn get_param(&self, name: &str) -> Result<ParameterValue> {
          match self.def().get_parameter(name) {
            Some(param) => match param.field_name.as_str() {
              $(
                stringify!($field_param_name) => {
                  Ok(self.$field_param_name.get_param())
                }
              )*
              field_name => Err(anyhow::anyhow!("Unknown parameter field: {field_name}")),
            }
            _ => Err(anyhow::anyhow!("Unknown parameter: {name}")),
          }
        }

        fn set_param(&mut self, name: &str, _value: ParameterValue) -> Result<()> {
          match self.def().get_parameter(name) {
            Some(param) => match param.field_name.as_str() {
              $(
                stringify!($field_param_name) => {
                  self.$field_param_name.set_param(_value)
                }
              )*
              field_name => Err(anyhow::anyhow!("Unknown parameter field: {field_name}")),
            }
            _ => Err(anyhow::anyhow!("Unknown param: {name}")),
          }
        }

        $( $ty_node_impl_fns )*
      }

      $($rest)*
    }
    pub use $mod_name::$node_ty_name;
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use glam::Vec2;

  use crate::*;

  impl_node! {
    mod test_node {
      NodeInfo {
        name: "Test Node",
        description: "Some long description",
        categories: ["Test"],
        // Define some custom fields in the node definition.
        custom: {
          // custom field 1.
          test_custom_field1: "Test value",
          // custom field 2.  All values are converted to `String`.
          test_custom_field2: 1234,
        },
      }

      /// Test docs.
      pub enum Op {
        /// Math add op description.
        Add,
        /// Math sub op description.
        Sub,
      }

      /// Docs.
      #[derive(Default)]
      pub struct TestNode {
        /// Input `color`.
        pub color: Input<Vec2>,
        /// Input `scale`.
        pub scale: Input<f32>,
        /// Parameter `param`.
        pub param: Param<Vec2>,
        /// Parameter `op`.
        pub op: Param<Op>,
        /// Output `color`.
        pub out: Output<Vec2>,
        // Internal node field.
        #[serde(skip)]
        _temp: Vec2,
      }

      impl TestNode {
        pub fn new() -> Self {
          Default::default()
        }
      }

      impl NodeImpl for TestNode {
        fn compile(&self, _graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
          let block = compile.current_block()?;
          // TODO: add context lookups.
          block.append_output(id, "in.uv".to_string());
          Ok(())
        }
      }
    }
  }

  #[test]
  fn test_impl_node_macro() {
    let reg = NodeRegistry::new();
    let mut node = TestNode::new();
    node.set_input("Color", Vec2::new(1.0, 2.0).into()).unwrap();
    node.set_input("Scale", 3.14.into()).unwrap();
    node.set_param("Param", Vec2::new(1.0, 2.0).into()).unwrap();
    node.set_param("Op", "Sub".into()).unwrap();
    println!(" - node: {:#?}", node);
    println!("   - def: {:#?}", node.def());
    println!(
      "   - json: {}",
      serde_json::to_string_pretty(&node).unwrap()
    );
    reg.register::<TestNode>();
  }
}
