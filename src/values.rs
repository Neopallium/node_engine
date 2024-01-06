use core::any::Any;
use std::sync::Arc;

use heck::ToTitleCase;
use indexmap::IndexSet;

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

pub mod types;
pub use types::*;

pub mod vector;
pub use vector::*;

pub mod scalar;

pub mod matrix;
pub use matrix::*;

pub mod texture;

pub mod bindings;
pub use bindings::*;

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextureHandleInner {
  pub id: uuid::Uuid,
  pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Texture2DHandle(Option<Arc<TextureHandleInner>>);

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Texture2DArrayHandle(Option<Arc<TextureHandleInner>>);

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Texture3DHandle(Option<Arc<TextureHandleInner>>);

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CubemapHandle(Option<Arc<TextureHandleInner>>);

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
  I32(i32),
  U32(u32),
  F32(f32),
  Vec2(Vec2),
  Vec3(Vec3),
  Vec4(Vec4),
  Mat2(Mat2),
  Mat3(Mat3),
  Mat4(Mat4),
  Texture2D(Texture2DHandle),
  Texture2DArray(Texture2DArrayHandle),
  Texture3D(Texture3DHandle),
  Cubemap(CubemapHandle),
}

impl Default for Value {
  fn default() -> Self {
    Self::F32(Default::default())
  }
}

impl Value {
  pub fn as_any(&self) -> &dyn Any {
    match self {
      Self::I32(v) => v,
      Self::U32(v) => v,
      Self::F32(v) => v,
      Self::Vec2(v) => v,
      Self::Vec3(v) => v,
      Self::Vec4(v) => v,
      Self::Mat2(v) => v,
      Self::Mat3(v) => v,
      Self::Mat4(v) => v,
      Self::Texture2D(v) => v,
      Self::Texture2DArray(v) => v,
      Self::Texture3D(v) => v,
      Self::Cubemap(v) => v,
    }
  }

  pub fn data_type(&self) -> DataType {
    match self {
      Self::I32(_) => DataType::I32,
      Self::U32(_) => DataType::U32,
      Self::F32(_) => DataType::F32,
      Self::Vec2(_) => DataType::Vec2,
      Self::Vec3(_) => DataType::Vec3,
      Self::Vec4(_) => DataType::Vec4,
      Self::Mat2(_) => DataType::Mat2,
      Self::Mat3(_) => DataType::Mat3,
      Self::Mat4(_) => DataType::Mat4,
      Self::Texture2D(_) => DataType::Texture2D,
      Self::Texture2DArray(_) => DataType::Texture2DArray,
      Self::Texture3D(_) => DataType::Texture3D,
      Self::Cubemap(_) => DataType::Cubemap,
    }
  }

  pub fn compile(&self) -> Result<CompiledValue> {
    let value = match self {
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

  #[cfg(feature = "egui")]
  pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
    match self {
      Self::I32(v) => v.ui(ui),
      Self::U32(v) => v.ui(ui),
      Self::F32(v) => v.ui(ui),
      Self::Vec2(v) => v.ui(ui),
      Self::Vec3(v) => v.ui(ui),
      Self::Vec4(v) => v.ui(ui),
      Self::Mat2(v) => v.ui(ui),
      Self::Mat3(v) => v.ui(ui),
      Self::Mat4(v) => v.ui(ui),
      Self::Texture2D(v) => v.ui(ui),
      Self::Texture2DArray(v) => v.ui(ui),
      Self::Texture3D(v) => v.ui(ui),
      Self::Cubemap(v) => v.ui(ui),
    }
  }
}

impl From<i32> for Value {
  fn from(v: i32) -> Self {
    Self::I32(v)
  }
}

impl From<u32> for Value {
  fn from(v: u32) -> Self {
    Self::U32(v)
  }
}

impl From<f32> for Value {
  fn from(v: f32) -> Self {
    Self::F32(v)
  }
}

impl From<Vec2> for Value {
  fn from(v: Vec2) -> Self {
    Self::Vec2(v)
  }
}

impl From<Vec3> for Value {
  fn from(v: Vec3) -> Self {
    Self::Vec3(v)
  }
}

impl From<Vec4> for Value {
  fn from(v: Vec4) -> Self {
    Self::Vec4(v)
  }
}

impl From<Mat2> for Value {
  fn from(v: Mat2) -> Self {
    Self::Mat2(v)
  }
}

impl From<Mat3> for Value {
  fn from(v: Mat3) -> Self {
    Self::Mat3(v)
  }
}

impl From<Mat4> for Value {
  fn from(v: Mat4) -> Self {
    Self::Mat4(v)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InputDefinition {
  pub name: String,
  pub field_name: String,
  pub value_type: DataType,
  pub color: Option<ecolor::Color32>,
}

impl InputDefinition {
  pub fn typed<T: ValueType + Default>(field_name: &str) -> (String, Self) {
    let val = T::default();
    Self::new(field_name, val.data_type())
  }

  pub fn new(field_name: &str, value_type: DataType) -> (String, Self) {
    let name = field_name.to_title_case();
    (
      name.clone(),
      Self {
        name,
        field_name: field_name.to_string(),
        value_type,
        color: None,
      },
    )
  }

  pub fn set_color(&mut self, color: Option<u32>) {
    self.color = color.map(u32_to_color);
  }

  pub fn default_value(&self) -> Value {
    self.value_type.default_value()
  }

  pub fn validate(&self, input: &Input) -> Result<()> {
    match input {
      Input::Disconnect => (),
      Input::Value(val) => {
        let in_type = val.data_type();
        if self.value_type != in_type {
          return Err(anyhow::anyhow!(
            "Wrong input data type: expected {:?} got {:?}",
            self.value_type,
            in_type
          ));
        }
      }
      Input::Connect(_, _) => (),
    }
    Ok(())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OutputDefinition {
  pub name: String,
  pub field_name: String,
  pub value_type: DataType,
  pub color: Option<ecolor::Color32>,
}

impl OutputDefinition {
  pub fn typed<T: ValueType + Default>(field_name: &str) -> (String, Self) {
    let val = T::default();
    Self::new(field_name, val.data_type())
  }

  pub fn new(field_name: &str, value_type: DataType) -> (String, Self) {
    let name = field_name.to_title_case();
    (
      name.clone(),
      Self {
        name,
        field_name: field_name.to_string(),
        value_type,
        color: None,
      },
    )
  }

  pub fn set_color(&mut self, color: Option<u32>) {
    self.color = color.map(u32_to_color);
  }

  pub fn default_value(&self) -> Value {
    self.value_type.default_value()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ParameterDataType {
  Value(DataType),
  Text(String),
  Select(IndexSet<String>),
}

impl ParameterDataType {
  pub fn select(values: &[&str]) -> Self {
    Self::Select(values.iter().map(|s| s.to_string()).collect())
  }

  pub fn default_value(&self) -> ParameterValue {
    match self {
      Self::Value(dt) => ParameterValue::Value(dt.default_value()),
      Self::Text(val) => ParameterValue::Text(val.clone()),
      Self::Select(values) => {
        let val = values.first().cloned().unwrap_or_default();
        ParameterValue::Selected(val)
      }
    }
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ParameterValue {
  Value(Value),
  Text(String),
  Selected(String),
}

impl ParameterValue {
  pub fn parameter_data_type(&self) -> ParameterDataType {
    match self {
      Self::Value(val) => ParameterDataType::Value(val.data_type()),
      Self::Text(val) => ParameterDataType::Text(val.clone()),
      Self::Selected(val) => ParameterDataType::Select([val].into_iter().cloned().collect()),
    }
  }
}

impl<T: Into<Value>> From<T> for ParameterValue {
  fn from(v: T) -> Self {
    Self::Value(v.into())
  }
}

impl From<&'static str> for ParameterValue {
  fn from(v: &'static str) -> Self {
    Self::Selected(v.to_string())
  }
}

pub trait ParameterType {
  fn get_param(&self) -> ParameterValue;

  fn set_param(&mut self, value: ParameterValue) -> Result<()>;

  fn parameter_data_type() -> ParameterDataType;

  #[cfg(feature = "egui")]
  fn parameter_ui(&mut self, def: &ParameterDefinition, ui: &mut egui::Ui, _id: NodeId, _details: bool) -> bool {
    ui.horizontal(|ui| {
      let mut value = self.get_param();
      ui.label(&def.name);
      if def.ui(ui, &mut value) {
        if let Err(err) = self.set_param(value) {
          log::error!("Failed to update node parameter: {err:?}");
        }
        true
      } else {
        false
      }
    })
    .inner
  }
}

impl<T> ParameterType for T
where
  T: ValueType + Default,
{
  fn get_param(&self) -> ParameterValue {
    ParameterValue::Value(self.to_value())
  }

  fn set_param(&mut self, value: ParameterValue) -> Result<()> {
    match value {
      ParameterValue::Value(val) => {
        self.set_value(val)?;
        Ok(())
      }
      _ => Err(anyhow!("Unsupport ParameterValue -> Value conversion.")),
    }
  }

  fn parameter_data_type() -> ParameterDataType {
    let val = T::default();
    ParameterDataType::Value(val.data_type())
  }

  #[cfg(feature = "egui")]
  fn parameter_ui(&mut self, def: &ParameterDefinition, ui: &mut egui::Ui, _id: NodeId, _details: bool) -> bool {
    ui.horizontal(|ui| {
      ui.label(&def.name);
      self.ui(ui)
    })
    .inner
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParameterDefinition {
  pub name: String,
  pub field_name: String,
  pub param_type: ParameterDataType,
}

impl ParameterDefinition {
  pub fn typed<T: ParameterType>(field_name: &str) -> (String, Self) {
    Self::new(field_name, T::parameter_data_type())
  }

  pub fn new(field_name: &str, param_type: ParameterDataType) -> (String, Self) {
    let name = field_name.to_title_case();
    (
      name.clone(),
      Self {
        name,
        field_name: field_name.to_string(),
        param_type,
      },
    )
  }

  pub fn value(name: &str, data_type: DataType) -> (String, Self) {
    Self::new(name, ParameterDataType::Value(data_type))
  }

  pub fn select(name: &str, values: &[&str]) -> (String, Self) {
    Self::new(name, ParameterDataType::select(values))
  }

  pub fn default_value(&self) -> ParameterValue {
    self.param_type.default_value()
  }

  pub fn validate(&self, value: &ParameterValue) -> Result<()> {
    match (&self.param_type, value) {
      (ParameterDataType::Value(data_type), ParameterValue::Value(val)) => {
        let in_type = val.data_type();
        if data_type != &in_type {
          Err(anyhow::anyhow!(
            "Wrong parameter type: expected {:?} got {:?}",
            data_type,
            in_type
          ))
        } else {
          Ok(())
        }
      }
      (ParameterDataType::Select(values), ParameterValue::Selected(val)) => {
        if values.contains(val) {
          Ok(())
        } else {
          Err(anyhow::anyhow!(
            "Invalid parameter selected value: {:?}",
            val
          ))
        }
      }
      (expected, got) => Err(anyhow::anyhow!(
        "Wrong parameter type: expected {:?} got {:?}",
        expected,
        got
      )),
    }
  }

  #[cfg(feature = "egui")]
  pub fn ui(&self, ui: &mut egui::Ui, value: &mut ParameterValue) -> bool {
    ui.horizontal(|ui| match (&self.param_type, value) {
      (ParameterDataType::Value(_), ParameterValue::Value(value)) => value.ui(ui),
      (ParameterDataType::Select(values), ParameterValue::Selected(selected)) => {
        let mut changed = false;
        egui::ComboBox::from_id_source(&self.field_name)
          .selected_text(selected.as_str())
          .show_ui(ui, |ui| {
            for value in values {
              if ui
                .selectable_value(selected, value.to_string(), value)
                .changed()
              {
                changed = true;
              }
            }
          });
        changed
      }
      _ => {
        ui.label("Invalid node parameter.  The value and definition don't match.");
        false
      }
    })
    .inner
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OutputTyped<T, const N: u32, const C: u32 = 0> {
  _phantom: core::marker::PhantomData<T>,
  /// Used for Dynamic outputs.
  concrete_type: Option<DataType>,
}

impl<T: ValueType + Default, const N: u32, const C: u32> OutputTyped<T, N, C> {
  pub fn data_type(&self) -> DataType {
    self.concrete_type.unwrap_or_else(|| T::default().data_type())
  }

  pub fn is_dynamic(&self) -> bool {
    T::default().data_type().is_dynamic()
  }

  pub fn update_concrete_type(&mut self, concrete_type: &NodeConcreteType) -> bool {
    let dt = T::default().data_type();
    let new_type = match dt {
      DataType::Dynamic => concrete_type.data_type(),
      DataType::DynamicVector => match concrete_type.min {
        Some(DynamicSize::D2) => Some(DataType::Vec2),
        Some(DynamicSize::D3) => Some(DataType::Vec3),
        Some(DynamicSize::D4) => Some(DataType::Vec4),
        _ => Some(DataType::F32),
      },
      DataType::DynamicMatrix => match concrete_type.min {
        Some(DynamicSize::D2) => Some(DataType::Mat2),
        Some(DynamicSize::D3) => Some(DataType::Mat3),
        Some(DynamicSize::D4) => Some(DataType::Mat4),
        _ => None,
      },
      _ => None,
    };
    if new_type != self.concrete_type {
      self.concrete_type = new_type;
      true
    } else {
      false
    }
  }

  pub fn compile(&self, compile: &mut NodeGraphCompile, node: NodeId, prefix: &str, code: String, dt: DataType) -> Result<()> {
    compile.add_output(OutputId::new(node, N), prefix, code, dt)
  }
}

#[cfg(feature = "egui")]
impl<T: ValueType + Default, const N: u32, const C: u32> OutputTyped<T, N, C> {
  #[cfg(feature = "egui")]
  pub fn ui(&mut self, concrete_type: &mut NodeConcreteType, def: &OutputDefinition, ui: &mut egui::Ui, id: NodeId, details: bool) {
    ui.horizontal(|ui| {
      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if self.is_dynamic() && self.update_concrete_type(concrete_type) {
          if let Some(graph) = NodeGraphMeta::get(ui) {
            graph.update_output(OutputId::new(id, N));
          }
        }
        if !details {
          ui.add(NodeSocket::output(id, N, def, self.concrete_type));
        }
        ui.label(&def.name);
      });
    });
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[derive(Clone, Debug, Default)]
  pub struct TestOutput {
    pub out0: OutputTyped<f32, { 0 + 0 }, 0>,
    pub out1: OutputTyped<f32, { 0 + 1 }, 0>,
  }

  #[test]
  fn test_typed_outputs() {
    let test = TestOutput::default();
    eprintln!("{test:?}");
  }
}
