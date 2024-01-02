use anyhow::Result;

use crate::*;

impl_enum_parameter_type!(
  #[derive(PartialEq, Eq)]
  pub enum CoordSpace {
    Object,
    View,
    World,
    Tangent,
  }
);

impl_enum_parameter_type!(
  #[derive(PartialEq, Eq)]
  pub enum UvChannel {
    UV0,
    UV1,
    UV2,
    UV3,
  }
);

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SwizzleMask(pub String);

pub fn is_swizzle(ch: char) -> bool {
  match ch  {
    'x' | 'y' | 'z' | 'w' => true,
    'r' | 'g' | 'b' | 'a' => true,
    _ => false,
  }
}

pub fn is_swizzle_limit(len: usize, ch: char) -> bool {
  match ch  {
    'x' | 'r' => true,
    'y' | 'g' if len > 1 => true,
    'z' | 'b' if len > 2 => true,
    'w' | 'a' if len > 3 => true,
    _ => false,
  }
}

impl SwizzleMask {
  pub fn compile(&self, input: CompiledValue) -> Result<CompiledValue> {
    let len = match input.dt {
      DataType::I32 | DataType::U32 | DataType::F32 => 1,
      DataType::Vec2 => 2,
      DataType::Vec3 => 3,
      DataType::Vec4 => 4,
      _ => {
        return Err(anyhow::anyhow!("Unsupport input for Swizzle: {input:?}"));
      }
    };
    // Validate mask.
    let mask = self.0.replace(|ch| !is_swizzle_limit(len, ch), "");
    if mask != self.0 {
      return Err(anyhow::anyhow!("Invalid swizzle mask: contains components not in the input: {}", self.0));
    }
    let out_dt = match self.0.len() {
      4 => DataType::Vec4,
      3 => DataType::Vec3,
      2 => DataType::Vec2,
      _ => input.dt,
    };
    let out = if len == 1 {
      // Special case for scalar inputs.
      match out_dt {
        DataType::Vec4 => format!("vec4<f32>({input}, {input}, {input}, {input})"),
        DataType::Vec3 => format!("vec3<f32>({input}, {input}, {input})"),
        DataType::Vec2 => format!("vec2<f32>({input}, {input})"),
        _ => input.value,
      }
    } else {
      format!("{input}.{}", self.0)
    };
    Ok(CompiledValue {
      value: out,
      dt: out_dt,
    })
  }

  pub fn filter(&mut self) {
    // Remove any non-swizzle digit.
    let mut mask = self.0.replace(|ch| !is_swizzle(ch), "");
    // Limit to 4 digits long.
    if mask.len() > 4 {
      mask = mask[0..4].to_string();
    }
    self.0 = mask;
  }
}

impl ParameterType for SwizzleMask {
  fn get_param(&self) -> ParameterValue {
    ParameterValue::Text(self.0.clone())
  }

  fn set_param(&mut self, value: ParameterValue) -> Result<()> {
    match value {
      ParameterValue::Text(val) => {
        self.0 = val;
        self.filter();
        Ok(())
      }
      _ => Err(anyhow::anyhow!("Unsupport ParameterValue -> Value conversion.")),
    }
  }

  fn parameter_data_type() -> ParameterDataType {
    ParameterDataType::Text("xyz".to_string())
  }

  #[cfg(feature = "egui")]
  fn parameter_ui(&mut self, _def: &ParameterDefinition, ui: &mut egui::Ui, _id: NodeId) -> bool {
    ui.horizontal(|ui| {
      ui.label("Swizzle mask");
      let resp = ui.add(egui::TextEdit::singleline(&mut self.0).hint_text("Mask"));
      if resp.changed() {
        self.filter();
        true
      } else {
        false
      }
    })
    .inner
  }
}
