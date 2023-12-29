use core::any::Any;

use heck::ToTitleCase;
use indexmap::IndexSet;

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

pub mod types;
pub use types::*;

pub mod bindings;
pub use bindings::*;

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
    }
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

pub fn decode_color(color: Option<&str>) -> Option<ecolor::Color32> {
  use ecolor::Color32;
  match color {
    None => None,
    Some("RED") => Some(Color32::RED),
    Some("GREEN") => Some(Color32::GREEN),
    Some("BLUE") => Some(Color32::BLUE),
    Some("YELLOW") => Some(Color32::YELLOW),
    Some("LIGHT_RED") => Some(Color32::LIGHT_RED),
    Some("LIGHT_GREEN") => Some(Color32::LIGHT_GREEN),
    Some("LIGHT_BLUE") => Some(Color32::LIGHT_BLUE),
    Some("LIGHT_YELLOW") => Some(Color32::LIGHT_YELLOW),
    Some("DARK_RED") => Some(Color32::DARK_RED),
    Some("DARK_GREEN") => Some(Color32::DARK_GREEN),
    Some("DARK_BLUE") => Some(Color32::DARK_BLUE),
    Some("WHITE") => Some(Color32::WHITE),
    Some("BLACK") => Some(Color32::BLACK),
    Some(val) => {
      let off = if val.starts_with("#") {
        1
      } else if val.starts_with("0x") {
        2
      } else {
        0
      };
      match hex::decode(&val[off..]) {
        Ok(val) if val.len() > 4 => {
          log::error!("Failed to decode color, hex value too long: {val:?}");
          None
        }
        Ok(val) => {
          let r = *val.get(0).unwrap_or(&0);
          let g = *val.get(1).unwrap_or(&0);
          let b = *val.get(2).unwrap_or(&0);
          if let Some(a) = val.get(3) {
            Some(Color32::from_rgba_premultiplied(r, g, b, *a))
          } else {
            Some(Color32::from_rgb(r, g, b))
          }
        }
        Err(err) => {
          log::error!("Failed to decode color: {val:?}: {err:?}");
          None
        }
      }
    }
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

  pub fn set_color(&mut self, color: Option<&str>) {
    self.color = decode_color(color);
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

  pub fn set_color(&mut self, color: Option<&str>) {
    self.color = decode_color(color);
  }

  pub fn default_value(&self) -> Value {
    self.value_type.default_value()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ParameterDataType {
  Value(DataType),
  Select(IndexSet<String>),
}

impl ParameterDataType {
  pub fn select(values: &[&str]) -> Self {
    Self::Select(values.iter().map(|s| s.to_string()).collect())
  }

  pub fn default_value(&self) -> ParameterValue {
    match self {
      Self::Value(dt) => ParameterValue::Value(dt.default_value()),
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
  Selected(String),
}

impl ParameterValue {
  pub fn parameter_data_type(&self) -> ParameterDataType {
    match self {
      Self::Value(val) => ParameterDataType::Value(val.data_type()),
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
  fn parameter_ui(&mut self, def: &ParameterDefinition, ui: &mut egui::Ui, _id: NodeId) {
    ui.horizontal(|ui| {
      let mut value = self.get_param();
      ui.label(&def.name);
      if def.ui(ui, &mut value) {
        if let Err(err) = self.set_param(value) {
          log::error!("Failed to update node parameter: {err:?}");
        }
      }
    });
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
pub struct OutputTyped<T> {
  _phantom: core::marker::PhantomData<T>,
}

#[cfg(feature = "egui")]
impl<T: ValueType + Default> OutputTyped<T> {
  #[cfg(feature = "egui")]
  pub fn ui(&mut self, idx: usize, def: &OutputDefinition, ui: &mut egui::Ui, id: NodeId) {
    ui.horizontal(|ui| {
      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add(NodeSocket::output(id, idx, def));
        ui.label(&def.name);
      });
    });
  }
}
