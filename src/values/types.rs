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
    // Same data type, no conversion.
    if self == other {
      return true;
    }
    match self {
      Self::I32 => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::U32 => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::F32 => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::Vec2 => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::Vec3 => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::Vec4 => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::Mat2 => match other {
        Self::Mat2 => true,
        Self::Dynamic => true,
        Self::DynamicMatrix => true,
        _ => false,
      }
      Self::Mat3 => match other {
        Self::Mat2 => true,
        Self::Mat3 => true,
        Self::Dynamic => true,
        Self::DynamicMatrix => true,
        _ => false,
      }
      Self::Mat4 => match other {
        Self::Mat2 => true,
        Self::Mat3 => true,
        Self::Mat4 => true,
        Self::Dynamic => true,
        Self::DynamicMatrix => true,
        _ => false,
      }
      Self::Dynamic => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Mat2 => true,
        Self::Mat3 => true,
        Self::Mat4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        Self::DynamicMatrix => true,
        _ => false,
      }
      Self::DynamicVector => match other {
        Self::I32 => true,
        Self::U32 => true,
        Self::F32 => true,
        Self::Vec2 => true,
        Self::Vec3 => true,
        Self::Vec4 => true,
        Self::Dynamic => true,
        Self::DynamicVector => true,
        _ => false,
      }
      Self::DynamicMatrix => match other {
        Self::Mat2 => true,
        Self::Mat3 => true,
        Self::Mat4 => true,
        Self::Dynamic => true,
        Self::DynamicMatrix => true,
        _ => false,
      }
      _ => false,
    }
  }
}

pub trait ValueType: core::fmt::Debug {
  fn clone_value(&self) -> Box<dyn ValueType>;

  fn set_value(&mut self, value: Value) -> Result<()>;

  fn to_value(&self) -> Value;

  fn data_type(&self) -> DataType;

  fn compile(&self) -> Result<CompiledValue> {
    let value = match self.to_value() {
      Value::I32(val) => {
        format!("{val:?}")
      }
      Value::U32(val) => {
        format!("{val:?}")
      }
      Value::F32(val) => {
        format!("{val:?}")
      }
      Value::Vec2(v) => {
        format!("vec2<f32>({:?}, {:?})", v.x, v.y)
      }
      Value::Vec3(v) => {
        format!("vec3<f32>({:?}, {:?}, {:?})", v.x, v.y, v.z)
      }
      Value::Vec4(v) => {
        format!("vec4<f32>({:?}, {:?}, {:?}, {:?})", v.x, v.y, v.z, v.w)
      }
      Value::Mat2(m) => {
        let col0 = m.col(0).compile()?;
        let col1 = m.col(1).compile()?;
        format!("mat2x2({col0}, {col1})")
      }
      Value::Mat3(m) => {
        let col0 = m.col(0).compile()?;
        let col1 = m.col(1).compile()?;
        let col2 = m.col(2).compile()?;
        format!("mat3x3({col0}, {col1}, {col2})")
      }
      Value::Mat4(m) => {
        let col0 = m.col(0).compile()?;
        let col1 = m.col(1).compile()?;
        let col2 = m.col(2).compile()?;
        let col3 = m.col(3).compile()?;
        format!("mat4x4({col0}, {col1}, {col2}, {col3})")
      }
      Value::Texture2D(_) => {
        // TODO: Convert to wgsl syntax.
        format!("vec4<f32>(0.5, 0.5, 0., 1.)")
      }
      Value::Texture2DArray(_) => {
        // TODO: Convert to wgsl syntax.
        format!("vec4<f32>(0.5, 0.5, 0., 1.)")
      }
      Value::Texture3D(_) => {
        // TODO: Convert to wgsl syntax.
        format!("vec4<f32>(0.5, 0.5, 0., 1.)")
      }
      Value::Cubemap(_) => {
        // TODO: Convert to wgsl syntax.
        format!("vec4<f32>(0.5, 0.5, 0., 1.)")
      }
    };
    Ok(CompiledValue {
      value,
      dt: self.data_type(),
    })
  }

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
