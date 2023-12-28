use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
use crate::ui::*;
use crate::*;

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
      None => self.value.eval(graph, execution),
    }
  }

  pub fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile) -> Result<String> {
    match &self.connected {
      Some(OutputId { node: id, .. }) => compile.resolve_node(graph, *id),
      None => self.value.compile(graph, compile),
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

  #[cfg(feature = "egui")]
  pub fn ui(&mut self, idx: u32, def: &InputDefinition, ui: &mut egui::Ui, id: NodeId) {
    ui.horizontal(|ui| {
      let connected = self.is_connected();
      let input_id = NodeSocketId::input(0, id, idx, T::data_type());
      ui.add(NodeSocket::new(input_id, connected, def.color));
      if connected {
        ui.label(&def.name);
      } else {
        ui.collapsing(&def.name, |ui| {
          self.value.ui(ui);
        });
      }
    });
  }
}
