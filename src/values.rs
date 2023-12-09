use core::any::Any;
use core::fmt;

use heck::ToTitleCase;
use indexmap::IndexSet;

use glam::{Vec2, Vec3, Vec4};

use anyhow::{anyhow, Result};

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataType {
  Scalar,
  Vec2,
  Vec3,
  Vec4,
}

impl DataType {
  pub fn default_value(&self) -> Value {
    match self {
      Self::Scalar => Value::Scalar(Default::default()),
      Self::Vec2 => Value::Vec2(Default::default()),
      Self::Vec3 => Value::Vec3(Default::default()),
      Self::Vec4 => Value::Vec4(Default::default()),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
  Scalar(f32),
  Vec2(Vec2),
  Vec3(Vec3),
  Vec4(Vec4),
}

impl Default for Value {
  fn default() -> Self {
    Self::Scalar(Default::default())
  }
}

impl Value {
  pub fn as_f32(&self) -> Result<f32> {
    match self {
      Self::Scalar(v) => Ok(*v),
      _ => Err(anyhow!("Expected a Scalar got: {self:?}")),
    }
  }

  pub fn as_vec2(&self) -> Result<Vec2> {
    match self {
      Self::Vec2(v) => Ok(*v),
      _ => Err(anyhow!("Expected a Vec2 got: {self:?}")),
    }
  }

  pub fn as_vec3(&self) -> Result<Vec3> {
    match self {
      Self::Vec3(v) => Ok(*v),
      _ => Err(anyhow!("Expected a Vec3 got: {self:?}")),
    }
  }

  pub fn as_vec4(&self) -> Result<Vec4> {
    match self {
      Self::Vec4(v) => Ok(*v),
      _ => Err(anyhow!("Expected a Vec4 got: {self:?}")),
    }
  }

  pub fn as_any(&self) -> &dyn Any {
    match self {
      Self::Scalar(v) => v,
      Self::Vec2(v) => v,
      Self::Vec3(v) => v,
      Self::Vec4(v) => v,
    }
  }

  pub fn data_type(&self) -> DataType {
    match self {
      Self::Scalar(_) => DataType::Scalar,
      Self::Vec2(_) => DataType::Vec2,
      Self::Vec3(_) => DataType::Vec3,
      Self::Vec4(_) => DataType::Vec4,
    }
  }
}

impl From<f32> for Value {
  fn from(v: f32) -> Self {
    Self::Scalar(v)
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InputDefinition {
  pub field_name: String,
  pub value_type: DataType,
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
      },
    )
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
      Input::Connect(_) => (),
    }
    Ok(())
  }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutputDefinition {
  pub field_name: String,
  pub value_type: DataType,
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
      },
    )
  }

  pub fn default_value(&self) -> Value {
    self.value_type.default_value()
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputKey {
  Idx(u32),
  Name(String),
}

impl From<InputId> for InputKey {
  fn from(id: InputId) -> Self {
    Self::Idx(id.1)
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Input {
  Disconnect,
  Connect(OutputId),
  Value(Value),
}

impl<T: Into<Value>> From<T> for Input {
  fn from(v: T) -> Self {
    Self::Value(v.into())
  }
}

impl From<NodeId> for Input {
  fn from(n: NodeId) -> Self {
    Self::Connect(OutputId(n, 0))
  }
}

pub trait ValueType: Sized + Default + Clone + fmt::Debug + 'static {
  fn to_value(&self) -> Value;

  fn from_value(value: Value) -> Result<Self>;

  fn data_type() -> DataType;
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
      Some(id) => Input::Connect(*id),
      None => Input::Value(self.value.to_value()),
    }
  }

  pub fn eval(&self, graph: &NodeGraph, execution: &mut NodeGraphExecution) -> Result<T> {
    match &self.connected {
      Some(OutputId(id, _)) => T::from_value(execution.eval_node(graph, *id)?),
      None => Ok(self.value.clone()),
    }
  }

  pub fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile) -> Result<String> {
    match &self.connected {
      Some(OutputId(id, _)) => compile.resolve_node(graph, *id),
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
      Input::Connect(id) => {
        self.connected = Some(id);
      }
    }
    Ok(old)
  }
}

impl ValueType for f32 {
  fn to_value(&self) -> Value {
    Value::Scalar(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    value.as_f32()
  }

  fn data_type() -> DataType {
    DataType::Scalar
  }
}

impl ValueType for Vec2 {
  fn to_value(&self) -> Value {
    Value::Vec2(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    value.as_vec2()
  }

  fn data_type() -> DataType {
    DataType::Vec2
  }
}

impl ValueType for Vec3 {
  fn to_value(&self) -> Value {
    Value::Vec3(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    value.as_vec3()
  }

  fn data_type() -> DataType {
    DataType::Vec3
  }
}

impl ValueType for Vec4 {
  fn to_value(&self) -> Value {
    Value::Vec4(*self)
  }

  fn from_value(value: Value) -> Result<Self> {
    value.as_vec4()
  }

  fn data_type() -> DataType {
    DataType::Vec4
  }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutputTyped<T> {
  _phantom: core::marker::PhantomData<T>,
}
