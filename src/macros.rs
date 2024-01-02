#[macro_export]
macro_rules! replace_expr {
  ($_t:tt $($sub:tt)*) => {$($sub)*};
}

#[macro_export]
macro_rules! impl_enum_parameter_type {
  // Parse Parameter enum types.
  ($(#[$param_enum_meta:meta])*
  pub enum $param_enum_name:ident {
    $(
      $(#[$variant_meta:meta])*
      $variant_name:ident
    ),+ $(,)?
  }) => {
    $(#[$param_enum_meta])*
    #[derive(Copy, Clone, Debug, Default)]
    #[derive($crate::serde::Serialize, $crate::serde::Deserialize)]
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
}

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
        [0] [0] [0]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      $(#[$param_enum_meta:meta])*
      pub enum $param_enum_name:ident {
        $($param_enum_fields:tt)*
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

          $crate::impl_enum_parameter_type!(
            $(#[$param_enum_meta])*
            pub enum $param_enum_name {
              $($param_enum_fields)*
            }
          );
        }
        [ $( $node_inputs )* ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
        [ $( $node_struct_fields )* ]
        { $( $node_struct )* }
        { $( $node_impl )* }
        { $( $node_trait_impl )* }
        $($rest)*
      }
    }
  };
  // Parse Node struct with docs.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
      []
      {}
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      #[doc = $node_struct_doc:expr]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
        []
        {
          #[doc = $node_struct_doc]
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
  // Parse Node struct without docs.
  (@normalize
    mod $mod_name:ident {
      { $( $node_info:tt )* }
      { $( $extra_code:tt )* }
      [ $( $node_inputs:tt )* ]
      [ $( $node_parameters:tt )* ]
      [ $( $node_outputs:tt )* ]
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
        []
        {
          #[doc = ""]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields {
          $( #[$field_meta:meta] )*
          $field_vis:vis $field_name:ident : Input<$field_ty:ident> $(Color($color:tt))?,
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
          $field_name : $field_ty Idx($( $count_inputs )*) $(Color($color))?,
        ]
        [ $( $node_parameters )* ]
        [ $( $node_outputs )* ]
        [ $( $count_inputs )* + 1 ] [ $( $count_params )* ] [ $( $count_outputs )* ]
        [
          $( $node_struct_fields )*
          $( #[$field_meta] )*
          $field_vis $field_name : InputTyped<$field_ty, { $( $count_inputs )* }>,
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      { $( $node_struct:tt )* }
      { $( $node_impl:tt )* }
      { $( $node_trait_impl:tt )* }
      ___internal_parse_fields {
          $( #[$field_meta:meta] )*
          $field_vis:vis $field_name:ident : Output<$field_ty:ident> $(Color($color:tt))?,
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
          $field_name : $field_ty Idx($( $count_outputs )*) $(Color($color))?,
        ]
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* + 1 ]
        [
          $( $node_struct_fields )*
          $( #[$field_meta] )*
          #[serde(skip)]
          $field_vis $field_name : OutputTyped<$field_ty, { $( $count_outputs )* }$(, { $color })?>,
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
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
        [ $( $count_inputs )* ] [ $( $count_params )* ] [ $( $count_outputs )* ]
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
          , package: $node_package:expr
        )?
        $(
          , category: $node_category:expr
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
      [ $( $field_input_name:ident: $field_input_ty:ident Idx($field_input_idx:expr) $(Color($field_input_color:tt))?,)* ]
      [ $( $field_param_name:ident: $field_param_ty:ident, )* ]
      [ $( $field_output_name:ident: $field_output_ty:ident Idx($field_output_idx:expr) $(Color($field_output_color:tt))?, )* ]
      [ $( $count_inputs:tt )* ] [ $( $count_params:tt )* ] [ $( $count_outputs:tt )* ]
      [ $( $node_struct_fields:tt )* ]
      {
        #[doc = $node_struct_doc:expr]
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

      $crate::lazy_static::lazy_static! {
        pub static ref DEFINITION: $crate::NodeDefinition = {
          // Use the module_path to generate the node definition id.
          let path = module_path!();
          let mut def = $crate::NodeDefinition::new($node_name, path, |_, data| {
            use $crate::serde::Deserialize;
            Ok(Box::new(match data {
              Some(data) => $node_ty_name::deserialize(data)?,
              None => $node_ty_name::new(),
            }))
          });
          def.set_docs($node_struct_doc);
          $( def.description = $node_description.to_string(); )?
          $(
            def.category = $node_category.iter().map(|c| c.to_string()).collect();
          )?
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
          $( def.package = $node_package.to_string(); )?
          // Save source file to help with debugging duplicates (uuid clashes).
          def.source_file = file!().to_string();

          // Set custom input/output colors
          $(
            $(
              {
                def.set_input_color($field_input_idx, Some($field_input_color));
              }
            )?
          )*
          $(
            $(
              {
                def.set_output_color($field_output_idx, Some($field_output_color));
              }
            )?
          )*
          def
        };
      }

      $crate::register_node! {
        DEFINITION.clone()
      }

      $( $extra_code )*

      #[doc = $node_struct_doc]
      $(#[$node_struct_attr])*
      #[derive(Clone, Debug)]
      #[derive($crate::serde::Serialize, $crate::serde::Deserialize)]
      pub struct $node_ty_name {
        $( $node_struct_fields )*
      }

      $( $node_impl )*

      $crate::impl_node! {
        @impl_resolve_inputs
        $node_ty_name [ $( $field_input_name ),* ]
      }

      $(#[$node_impl_meta])*
      impl NodeImpl for $node_ty_name {
        fn clone_node(&self) -> Box<dyn $crate::NodeImpl> {
          Box::new(self.clone())
        }

        fn def(&self) -> &$crate::NodeDefinition {
          &DEFINITION
        }

        fn get_node_input(&self, key: &InputKey) -> Result<Input> {
          $(
            #[allow(non_upper_case_globals)]
            const $field_input_name: u32 = $field_input_idx;
          )*
          #[allow(non_upper_case_globals)]
          match self.get_input_idx(key)? {
            $(
              $field_input_name => {
                Ok(self.$field_input_name.as_input())
              }
            )*
            _ => Err(anyhow::anyhow!("Invalid input key: {key:?}")),
          }
        }

        fn set_node_input(&mut self, key: &InputKey, _value: Input) -> Result<Option<OutputId>> {
          $(
            #[allow(non_upper_case_globals)]
            const $field_input_name: u32 = $field_input_idx;
          )*
          #[allow(non_upper_case_globals)]
          match self.get_input_idx(key)? {
            $(
              $field_input_name => {
                self.$field_input_name.set_input(_value)
              }
            )*
            _ => Err(anyhow::anyhow!("Invalid input key: {key:?}")),
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

        #[cfg(feature = "egui")]
        fn inputs_ui(&mut self, _concrete_type: &mut NodeConcreteType, _ui: &mut egui::Ui, _id: NodeId) -> bool {
          let mut _defs = DEFINITION.inputs.values();
          let mut _updated = false;
          $(
            if let Some(def) = _defs.next() {
              if self.$field_input_name.ui(_concrete_type, def, _ui, _id) {
                _updated = true;
              }
            }
          )*
          _updated
        }

        #[cfg(feature = "egui")]
        fn outputs_ui(&mut self, _concrete_type: &mut NodeConcreteType, _ui: &mut egui::Ui, _id: NodeId) -> bool {
          let mut _defs = DEFINITION.outputs.values();
          $(
            if let Some(def) = _defs.next() {
              self.$field_output_name.ui(_concrete_type, def, _ui, _id);
            }
          )*
          false
        }

        #[cfg(feature = "egui")]
        fn parameters_ui(&mut self, _concrete_type: &mut NodeConcreteType, _ui: &mut egui::Ui, _id: NodeId) -> bool {
          let mut _defs = DEFINITION.parameters.values();
          let mut _updated = false;
          $(
            if let Some(def) = _defs.next() {
              if self.$field_param_name.parameter_ui(def, _ui, _id) {
                _updated = true;
              }
            }
          )*
          _updated
        }

        $( $ty_node_impl_fns )*
      }

      $($rest)*
    }
    pub use $mod_name::$node_ty_name;
  };
  // Implement compile inputs helper.  No inputs.
  (@impl_resolve_inputs
    $node_ty_name:ident []
  ) => {
  };
  // Implement compile inputs helper.  Only one input.
  (@impl_resolve_inputs
    $node_ty_name:ident [ $field_input_name:ident ]
  ) => {
    impl $node_ty_name {
      pub fn resolve_inputs(&self, graph: &$crate::NodeGraph, compile: &mut $crate::NodeGraphCompile)
        -> Result<$crate::CompiledValue>
      {
        self.$field_input_name.compile(graph, compile)
      }
    }
  };
  // Implement compile inputs helper.
  (@impl_resolve_inputs
    $node_ty_name:ident [ $( $field_input_name:ident),* ]
  ) => {
    impl $node_ty_name {
      pub fn resolve_inputs(&self, graph: &$crate::NodeGraph, compile: &mut $crate::NodeGraphCompile)
        -> Result<($(
            $crate::replace_expr!(
              $field_input_name
              $crate::CompiledValue
            )
      ),*)>
      {
        let mut concrete_type = $crate::NodeConcreteType::default();
        $(
          let mut $field_input_name = self.$field_input_name.resolve(&mut concrete_type, graph, compile)?;
        )*
        if concrete_type.has_dynamic() {
          $(
            if self.$field_input_name.is_dynamic() {
              concrete_type.convert(&mut $field_input_name)?;
            }
          )*
        }
        Ok(($(
          $field_input_name
        ),*))
      }
    }
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
        category: ["Test"],
        // Define some custom fields in the node definition.
        custom: {
          // custom field 1.
          test_custom_field1: "Test value",
          // custom field 2.  All values are converted to `String`.
          test_custom_field2: 1234,
        },
      }

      /// Document for `Op` parameter enum.
      pub enum Op {
        /// Math add op description.
        Add,
        /// Math sub op description.
        Sub,
      }

      /// Node short description.
      ///
      /// Longer node docs...
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
        fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile, id: NodeId) -> Result<()> {
          let (_color, _scale) = self.resolve_inputs(graph, compile)?;
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
  }
}
