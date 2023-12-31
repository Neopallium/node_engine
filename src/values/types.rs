use anyhow::Result;

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTypeClass {
  Scalar,
  Vector,
  Matrix,
  Dynamic,
  Texture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataType {
  I32,
  U32,
  F32,
  Vec2,
  Vec3,
  Vec4,
  Mat2,
  Mat3,
  Mat4,
  Dynamic,
  DynamicVector,
  DynamicMatrix,
  Texture2D,
  Texture2DArray,
  Texture3D,
  Cubemap,
}

impl DataType {
  /// Is this data type dynamic.
  pub const fn is_dynamic(&self) -> bool {
    match self {
      Self::Dynamic => true,
      Self::DynamicVector => true,
      Self::DynamicMatrix => true,
      _ => false,
    }
  }

  /// Returns the data types class.
  pub const fn class(&self) -> DataTypeClass {
    match self {
      Self::I32 => DataTypeClass::Scalar,
      Self::U32 => DataTypeClass::Scalar,
      Self::F32 => DataTypeClass::Scalar,
      Self::Vec2 => DataTypeClass::Vector,
      Self::Vec3 => DataTypeClass::Vector,
      Self::Vec4 => DataTypeClass::Vector,
      Self::Mat2 => DataTypeClass::Matrix,
      Self::Mat3 => DataTypeClass::Matrix,
      Self::Mat4 => DataTypeClass::Matrix,
      Self::Dynamic => DataTypeClass::Dynamic,
      Self::DynamicVector => DataTypeClass::Vector,
      Self::DynamicMatrix => DataTypeClass::Matrix,
      Self::Texture2D => DataTypeClass::Texture,
      Self::Texture2DArray => DataTypeClass::Texture,
      Self::Texture3D => DataTypeClass::Texture,
      Self::Cubemap => DataTypeClass::Texture,
    }
  }

  /// Get the default value for this data type.
  pub fn default_value(&self) -> Value {
    match self {
      Self::I32 => Value::I32(Default::default()),
      Self::U32 => Value::U32(Default::default()),
      Self::F32 => Value::F32(Default::default()),
      Self::Vec2 => Value::Vec2(Default::default()),
      Self::Vec3 => Value::Vec3(Default::default()),
      Self::Vec4 => Value::Vec4(Default::default()),
      Self::Mat2 => Value::Mat2(Default::default()),
      Self::Mat3 => Value::Mat3(Default::default()),
      Self::Mat4 => Value::Mat4(Default::default()),
      Self::Dynamic => Value::Vec4(Default::default()),
      Self::DynamicVector => Value::Vec4(Default::default()),
      Self::DynamicMatrix => Value::Mat4(Default::default()),
      Self::Texture2D => Value::Texture2D(Default::default()),
      Self::Texture2DArray => Value::Texture2DArray(Default::default()),
      Self::Texture3D => Value::Texture3D(Default::default()),
      Self::Cubemap => Value::Cubemap(Default::default()),
    }
  }

  /// Get the default port/connection color for this data type.
  #[cfg(feature = "egui")]
  pub const fn color(&self) -> egui::Color32 {
    match self {
      Self::I32 => egui::Color32::LIGHT_BLUE,
      Self::U32 => egui::Color32::LIGHT_BLUE,
      Self::F32 => egui::Color32::LIGHT_BLUE,
      Self::Vec2 => egui::Color32::GREEN,
      Self::Vec3 => egui::Color32::YELLOW,
      Self::Vec4 => egui::Color32::LIGHT_RED,
      Self::Mat2 => egui::Color32::BLUE,
      Self::Mat3 => egui::Color32::BLUE,
      Self::Mat4 => egui::Color32::BLUE,
      Self::Dynamic => egui::Color32::BLUE,
      Self::DynamicVector => egui::Color32::LIGHT_BLUE,
      Self::DynamicMatrix => egui::Color32::BLUE,
      Self::Texture2D => egui::Color32::RED,
      Self::Texture2DArray => egui::Color32::RED,
      Self::Texture3D => egui::Color32::RED,
      Self::Cubemap => egui::Color32::RED,
    }
  }

  /// Check if the data type is compatible.
  pub fn is_compatible(&self, other: &DataType) -> bool {
    if self == other {
      // Same data types are compatible.
      true
    } else {
      match (self.class(), other.class()) {
        (class, other) if class == other => match class {
          // Matrix types must have the same size.
          DataTypeClass::Matrix => false,
          _ => true,
        },
        (DataTypeClass::Dynamic, other) | (other, DataTypeClass::Dynamic) => match other {
          DataTypeClass::Scalar => true,
          DataTypeClass::Vector => true,
          DataTypeClass::Matrix => true,
          DataTypeClass::Dynamic => true,
          _ => false,
        },
        (DataTypeClass::Vector, other) | (other, DataTypeClass::Vector) => match other {
          DataTypeClass::Scalar => true,
          DataTypeClass::Vector => true,
          DataTypeClass::Dynamic => true,
          _ => false,
        },
        (DataTypeClass::Matrix, other) | (other, DataTypeClass::Matrix) => match other {
          DataTypeClass::Dynamic => true,
          // Matrix types must have the same size.
          DataTypeClass::Matrix => false,
          _ => false,
        },
        _ => false,
      }
    }
  }
}

pub trait ValueType: core::fmt::Debug {
  fn clone_value(&self) -> Box<dyn ValueType>;

  fn set_value(&mut self, value: Value) -> Result<()>;

  fn to_value(&self) -> Value;

  fn data_type(&self) -> DataType;

  fn is_dynamic(&self) -> bool {
    self.data_type().is_dynamic()
  }

  fn binding(&self) -> Option<&str> {
    None
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    ui.label("No UI for type.");
    false
  }
}

impl Clone for Box<dyn ValueType> {
  fn clone(&self) -> Self {
    self.clone_value()
  }
}
