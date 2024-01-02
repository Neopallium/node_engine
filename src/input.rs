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

#[derive(Clone, Debug)]
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
pub struct InputTyped<T, const N: u32> {
  value: T,
  connected: Option<(OutputId, Option<DataType>)>,
}

impl<T: ValueType, const N: u32> InputTyped<T, N> {
  pub fn new(value: T) -> Self {
    Self {
      value,
      connected: None,
    }
  }

  pub fn is_connected(&self) -> bool {
    self.connected.is_some()
  }

  pub fn is_dynamic(&self) -> bool {
    self.value.is_dynamic()
  }

  pub fn as_input(&self) -> Input {
    match &self.connected {
      Some((id, dt)) => Input::Connect(*id, *dt),
      None => Input::Value(self.value.to_value()),
    }
  }

  pub fn resolve(&self, concrete_type: &mut NodeConcreteType, graph: &NodeGraph, compile: &mut NodeGraphCompile) -> Result<CompiledValue> {
    let mut value = match &self.connected {
      Some((id, _)) => {
        let value = compile.resolve_output(graph, *id)?;
        if self.is_dynamic() {
          // Collect info about dynamic inputs.
          concrete_type.add_input_type(value.dt);
        }
        value
      }
      None => self.value.compile()?,
    };
    // Make sure the value is in our type.
    value.convert(self.value.data_type())?;
    Ok(value)
  }

  pub fn compile(&self, graph: &NodeGraph, compile: &mut NodeGraphCompile) -> Result<CompiledValue> {
    let mut value = match &self.connected {
      Some((id, _)) => compile.resolve_output(graph, *id)?,
      None => self.value.compile()?,
    };
    // Make sure the value is in our type.
    value.convert(self.value.data_type())?;
    Ok(value)
  }

  pub fn set_input(&mut self, input: Input) -> Result<Option<OutputId>> {
    let old = self.connected.take().map(|(id, _)| id);
    match input {
      Input::Disconnect => (),
      Input::Value(val) => {
        self.value.set_value(val)?;
      }
      Input::Connect(id, dt) => {
        if let Some(output_dt) = dt {
          if !self.value.data_type().is_compatible(&output_dt) {
            return Err(anyhow!("Incompatible output"));
          }
        }
        self.connected = Some((id, dt));
      }
    }
    Ok(old)
  }

  #[cfg(feature = "egui")]
  pub fn ui(&mut self, concrete_type: &mut NodeConcreteType, def: &InputDefinition, ui: &mut egui::Ui, id: NodeId) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
      match self.connected {
        Some((output_id, _)) => {
          if self.is_dynamic() {
            let dt = NodeGraphMeta::get(ui).and_then(|g| g.resolve_output(&output_id));
            if let Some(dt) = dt {
              concrete_type.add_input_type(dt);
            }
          }
          ui.add(NodeSocket::input(id, N, true, def));
          ui.label(&def.name);
        },
        None => {
          ui.add(NodeSocket::input(id, N, false, def));
          ui.collapsing(&def.name, |ui| {
            changed = self.value.ui(ui);
          });
        },
      }
    });
    changed
  }
}

impl<T: ValueType + Clone + Default, const N: u32> InputTyped<T, N> {
  pub fn eval(&self, graph: &NodeGraph, execution: &mut NodeGraphExecution) -> Result<T> {
    match &self.connected {
      Some((OutputId { node: id, .. }, _)) => {
        let mut val = T::default();
        val.set_value(execution.eval_node(graph, *id)?)?;
        Ok(val)
      }
      None => Ok(self.value.clone()),
    }
  }
}
