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

pub const fn u32_to_color(c: u32) -> ecolor::Color32 {
  let a = c.to_be_bytes();
  ecolor::Color32::from_rgb(a[0], a[1], a[2])
}

pub const fn color_rgb_to_u32(a: [u8; 3]) -> u32 {
  u32::from_be_bytes([0, a[0], a[1], a[2]])
}

#[macro_export]
macro_rules! const_color {
  (RED) => {
    0xFF0000
  };
  (GREEN) => {
    0x00FF00
  };
  (BLUE) => {
    0x0000FF
  };
  (WHITE) => {
    0xFFFFFF
  };
  ($c:literal) => {
    $c
  };
}
