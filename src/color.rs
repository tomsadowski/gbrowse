// src/color.rs

use crossterm::style::Color;


pub fn parse_color(v: &toml::Value) -> Result<Color, String> {
  match v {
    toml::Value::String(s) => 
      if let Some('#') = s.chars().next() {
        parse_hex_color(&s[1..])
      } else {
        parse_color_name(&s)
      }
    _ => Err(format!("could not parse color from value {v}")),
  }
}


pub fn parse_color_name(s: &str) -> Result<Color, String> {
  match s {
    "Red"         | "red"         => Ok(Color::Red),
    "Yellow"      | "yellow"      => Ok(Color::Yellow),
    "Green"       | "green"       => Ok(Color::Green),
    "Cyan"        | "cyan"        => Ok(Color::Cyan),
    "Blue"        | "blue"        => Ok(Color::Blue),
    "Magenta"     | "magenta"     => Ok(Color::Magenta),
    "Black"       | "black"       => Ok(Color::Black),
    "White"       | "white"       => Ok(Color::White),
    "Grey"        | "grey"     | 
    "Gray"        | "gray"        => Ok(Color::Grey),
    "DarkGrey"    | "darkgrey" | 
    "DarkGray"    | "darkgray"    => Ok(Color::DarkGrey),
    "DarkRed"     | "darkred"     => Ok(Color::DarkRed),
    "DarkYellow"  | "darkyellow"  => Ok(Color::DarkYellow),
    "DarkGreen"   | "darkgreen"   => Ok(Color::DarkGreen),
    "DarkCyan"    | "darkcyan"    => Ok(Color::DarkCyan),
    "DarkBlue"    | "darkblue"    => Ok(Color::DarkBlue),
    "DarkMagenta" | "darkmagenta" => Ok(Color::DarkMagenta),
    _ => Err(format!("could not parse color from value {s}"))
  }
}


pub fn parse_hex_color(s: &str) -> Result<Color, String> {

  fn try_hex(c: char) -> Result<u8, String> {
    match c {
      '0' => Ok(0),  '1' => Ok(1),  '2' => Ok(2),  '3' => Ok(3),
      '4' => Ok(4),  '5' => Ok(5),  '6' => Ok(6),  '7' => Ok(7),
      '8' => Ok(8),  '9' => Ok(9),  'a' => Ok(10), 'b' => Ok(11),
      'c' => Ok(12), 'd' => Ok(13), 'e' => Ok(14), 'f' => Ok(15),
      _   => Err(format!("{c} is not a hex character")),
    }
  }

  let mut c = s.chars();
  let r1 = c
    .next()
    .ok_or("missing first red".into())
    .and_then(|c| try_hex(c))?;
  let r2 = c
    .next()
    .ok_or("missing second red".into())
    .and_then(|c| try_hex(c))?;
  let g1 = c
    .next()
    .ok_or("missing first green".into())
    .and_then(|c| try_hex(c))?;
  let g2 = c
    .next()
    .ok_or("missing second green".into())
    .and_then(|c| try_hex(c))?;
  let b1 = c
    .next()
    .ok_or("missing first blue".into())
    .and_then(|c| try_hex(c))?;
  let b2 = c
    .next()
    .ok_or("missing second blue".into())
    .and_then(|c| try_hex(c))?;

  let r = 16 * r1 + r2;
  let g = 16 * g1 + g2;
  let b = 16 * b1 + b2;

  Ok(Color::Rgb {r, g, b})
}


#[derive(Copy, Clone, Debug, Default)]
pub struct Style {
  pub underline: bool,
  pub bold:      bool,
  pub fg:        Option<Color>,
  pub bg:        Option<Color>,
}


impl From<&crate::TextParams> for Style {
  fn from(item: &crate::TextParams) -> Self {
    item.style
  }
}


impl crossterm::Command for Style {
  fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
    use crossterm::style;
    let mut contentstyle = style::ContentStyle::new();
    contentstyle.foreground_color = self.fg;
    contentstyle.background_color = self.bg;
    let mut attributes = style::Attributes::none();
    if self.bold {
      attributes.set(style::Attribute::Bold);
    }
    if self.underline {
      attributes.set(style::Attribute::Underlined);
    }
    contentstyle.attributes = attributes;
    style::SetStyle(contentstyle).write_ansi(f)?;
    Ok(())
  }
}
