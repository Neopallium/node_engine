use core::any::Any;
use core::fmt;

use heck::ToTitleCase;
use indexmap::IndexSet;

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTypeClass {
  Scalar,
  Vector,
  Matrix,
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
}

impl DataType {
  pub fn class(&self) -> DataTypeClass {
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
    }
  }

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
    }
  }

  #[cfg(feature = "egui")]
  pub fn color(&self) -> egui::Color32 {
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
    }
  }

  pub fn is_compatible(&self, other: &DataType) -> bool {
    if self == other {
      // Same type is compatible.
      true
    } else if self.class() == other.class() {
      // Same class is compatible.
      true
    } else {
      match (self.class(), other.class()) {
        (DataTypeClass::Scalar, DataTypeClass::Vector)
        | (DataTypeClass::Vector, DataTypeClass::Scalar) => true,
        _ => false,
      }
    }
  }
}

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
  pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
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
  pub field_name: String,
  pub value_type: DataType,
  pub color: Option<ecolor::Color32>,
}

impl InputDefinition {
  pub fn typed<T: ValueType>(field_name: &str) -> (String, Self) {
    Self::new(field_name, T::data_type())
  }

  pub fn new(field_name: &str, value_type: DataType) -> (String, Self) {
    (
      field_name.to_title_case(),
      Self {
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
  pub field_name: String,
  pub value_type: DataType,
  pub color: Option<ecolor::Color32>,
}

impl OutputDefinition {
  pub fn typed<T: ValueType>(field_name: &str) -> (String, Self) {
    Self::new(field_name, T::data_type())
  }

  pub fn new(field_name: &str, value_type: DataType) -> (String, Self) {
    (
      field_name.to_title_case(),
      Self {
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

pub trait ParameterType: Sized + Default + Clone + fmt::Debug + 'static {
  fn get_param(&self) -> ParameterValue;

  fn set_param(&mut self, value: ParameterValue) -> Result<()>;

  fn parameter_data_type() -> ParameterDataType;
}

impl<T> ParameterType for T
where
  T: ValueType,
{
  fn get_param(&self) -> ParameterValue {
    ParameterValue::Value(self.to_value())
  }

  fn set_param(&mut self, value: ParameterValue) -> Result<()> {
    match value {
      ParameterValue::Value(val) => {
        *self = T::from_value(val)?;
        Ok(())
      }
      _ => Err(anyhow!("Unsupport ParameterValue -> Value conversion.")),
    }
  }

  fn parameter_data_type() -> ParameterDataType {
    ParameterDataType::Value(T::data_type())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParameterDefinition {
  pub field_name: String,
  pub param_type: ParameterDataType,
}

impl ParameterDefinition {
  pub fn typed<T: ParameterType>(field_name: &str) -> (String, Self) {
    Self::new(field_name, T::parameter_data_type())
  }

  pub fn new(field_name: &str, param_type: ParameterDataType) -> (String, Self) {
    (
      field_name.to_title_case(),
      Self {
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
      (ParameterDataType::Value(_), ParameterValue::Value(value)) => value.ui(ui).changed(),
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum InputKey {
  Idx(u32),
  Name(String),
}

impl From<InputId> for InputKey {
  fn from(id: InputId) -> Self {
    Self::Idx(id.idx)
  }
}

impl From<u32> for InputKey {
  fn from(idx: u32) -> Self {
    Self::Idx(idx)
  }
}

impl From<String> for InputKey {
  fn from(name: String) -> Self {
    Self::Name(name)
  }
}

impl From<&String> for InputKey {
  fn from(name: &String) -> Self {
    Self::Name(name.clone())
  }
}

impl From<&str> for InputKey {
  fn from(name: &str) -> Self {
    Self::Name(name.to_string())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Input {
  Disconnect,
  Connect(OutputId, Option<DataType>),
  Value(Value),
}

impl<T: Into<Value>> From<T> for Input {
  fn from(v: T) -> Self {
    Self::Value(v.into())
  }
}

impl From<NodeId> for Input {
  fn from(n: NodeId) -> Self {
    Self::Connect(OutputId::new(n, 0), None)
  }
}

pub trait ValueType: Sized + Default + Clone + fmt::Debug + 'static {
  fn to_value(&self) -> Value;

  fn from_value(value: Value) -> Result<Self>;

  fn data_type() -> DataType;

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.label("No UI for type.")
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct InputTyped<T> {
  value: T,
  connected: Option<OutputId>,
}

impl<T: ValueType> InputTyped<T> {
  pub fn new(value: T) -> Self {
    Self {
      value,
      connected: None,
    }
  }

  pub fn is_connected(&self) -> bool {
    self.connected.is_some()
  }

  pub fn as_input(&self) -> Input {
    match &self.connected {
      Some(id) => Input::Connect(*id, Some(T::data_type())),
      None => Input::Value(self.value.to_value()),
    }
  }

  pub fn eval(&self, graph: &NodeGraph, execution: &mut NodeGraphExecution) -> Result<T> {
    match &self.connected {
      Some(OutputId { node: id, .. }) => T::from_value(execution.eval_node(graph, *id)?),
      None => Ok(self.value.clone()),
    }
  }

  pub fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile) -> Result<String> {
    match &self.connected {
      Some(OutputId { node: id, .. }) => compile.resolve_node(graph, *id),
      None => compile.compile_value(&self.value.to_value()),
    }
  }

  pub fn set_input(&mut self, input: Input) -> Result<Option<OutputId>> {
    let old = self.connected.take();
    match input {
      Input::Disconnect => (),
      Input::Value(val) => {
        self.value = T::from_value(val)?;
      }
      Input::Connect(id, dt) => {
        if let Some(output_dt) = dt {
          if !T::data_type().is_compatible(&output_dt) {
            return Err(anyhow!("Incompatible output"));
          }
        }
        self.connected = Some(id);
      }
    }
    Ok(old)
  }
}

impl ValueType for i32 {
  fn to_value(&self) -> Value {
    Value::I32(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::I32(v) => Ok(v),
      _ => Err(anyhow!("Expected a I32 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::I32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.add(egui::DragValue::new(self).speed(1))
  }
}

impl ValueType for u32 {
  fn to_value(&self) -> Value {
    Value::U32(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::U32(v) => Ok(v),
      _ => Err(anyhow!("Expected a U32 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::U32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.add(egui::DragValue::new(self).speed(1))
  }
}

impl ValueType for f32 {
  fn to_value(&self) -> Value {
    Value::F32(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::F32(v) => Ok(v),
      _ => Err(anyhow!("Expected a F32 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::F32
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    ui.add(egui::DragValue::new(self).speed(0.1))
  }
}

#[cfg(feature = "egui")]
const COLUMNS: [&str; 4] = ["X", "Y", "Z", "W"];

#[cfg(feature = "egui")]
fn f32_table_ui(
  ui: &mut egui::Ui,
  columns: &[&str],
  rows: usize,
  values: &mut [f32],
) -> egui::Response {
  use egui_extras::{Column, TableBuilder};

  let node_style = ui.node_style();
  let mut resp = ui.interact(ui.min_rect(), ui.id(), egui::Sense::click());
  let height = 20.0 * node_style.zoom;
  let width = 40.0 * node_style.zoom;

  // Allocate space for the table.
  ui.set_min_height(height * (rows + 1) as f32);

  // Make sure the layout is in vertical mode.
  ui.vertical(|ui| {
    TableBuilder::new(ui)
      .columns(Column::exact(width), columns.len())
      .vscroll(false)
      .header(height, |mut header| {
        for col in columns {
          header.col(|ui| {
            ui.heading(*col);
          });
        }
      })
      .body(|body| {
        body.rows(height, rows, |row_index, mut row| {
          for col in 0..columns.len() {
            row.col(|ui| {
              if values[col * rows + row_index].ui(ui).changed() {
                resp.mark_changed();
              }
            });
          }
        });
      });
  });
  resp
}

#[cfg(feature = "egui")]
fn vector_ui(ui: &mut egui::Ui, values: &mut [f32]) -> egui::Response {
  let len = values.len();
  f32_table_ui(ui, &COLUMNS[0..len], 1, values)
}

#[cfg(feature = "egui")]
fn matrix_ui(ui: &mut egui::Ui, dim: usize, values: &mut [f32]) -> egui::Response {
  f32_table_ui(ui, &COLUMNS[0..dim], dim, values)
}

impl ValueType for Vec2 {
  fn to_value(&self) -> Value {
    Value::Vec2(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec2(v) => Ok(v),
      _ => Err(anyhow!("Expected a Vec2 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Vec3 {
  fn to_value(&self) -> Value {
    Value::Vec3(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec3(v) => Ok(v),
      _ => Err(anyhow!("Expected a Vec3 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Vec4 {
  fn to_value(&self) -> Value {
    Value::Vec4(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Vec4(v) => Ok(v),
      _ => Err(anyhow!("Expected a Vec4 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Vec4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    vector_ui(ui, self.as_mut())
  }
}

impl ValueType for Mat2 {
  fn to_value(&self) -> Value {
    Value::Mat2(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Mat2(v) => Ok(v),
      _ => Err(anyhow!("Expected a Mat2 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Mat2
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    matrix_ui(ui, 2, &mut self.as_mut()[..])
  }
}

impl ValueType for Mat3 {
  fn to_value(&self) -> Value {
    Value::Mat3(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Mat3(v) => Ok(v),
      _ => Err(anyhow!("Expected a Mat3 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Mat3
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    matrix_ui(ui, 3, &mut self.as_mut()[..])
  }
}

impl ValueType for Mat4 {
  fn to_value(&self) -> Value {
    Value::Mat4(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    match value {
      Value::Mat4(v) => Ok(v),
      _ => Err(anyhow!("Expected a Mat4 got: {value:?}")),
    }
  }

  fn data_type() -> DataType {
    DataType::Mat4
  }

  #[cfg(feature = "egui")]
  fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
    matrix_ui(ui, 4, &mut self.as_mut()[..])
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OutputTyped<T> {
  _phantom: core::marker::PhantomData<T>,
}
