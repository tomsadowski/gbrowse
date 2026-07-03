// src/action.rs


#[derive(Copy, Clone, Debug)]
pub enum Action {
  // editor
  Insert(char),
  Backspace,
  Enter,
  Delete,
  // tab
  Menu,
  LoadUrl,
  SaveUrl,
  DelTab, 
  NewTab, 
  CycleLeft, 
  CycleRight, 
  // selector
  MoveUp, 
  MoveDown, 
  MoveLeft, 
  MoveRight,
  Top,
  Bottom,
  PageUp,
  PageDown,
  Select, 
  // dialog
  Ack, 
  Yes, 
  No, 
  Cancel,
}
impl std::str::FromStr for Action {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "load_url"    => Ok(Self::LoadUrl),
      "save_url"    => Ok(Self::SaveUrl),
      "move_up"     => Ok(Self::MoveUp),
      "menu"        => Ok(Self::Menu),
      "move_down"   => Ok(Self::MoveDown),
      "move_left"   => Ok(Self::MoveLeft),
      "move_right"  => Ok(Self::MoveRight),
      "cycle_left"  => Ok(Self::CycleLeft),
      "cycle_right" => Ok(Self::CycleRight),
      "delete_tab"  => Ok(Self::DelTab),
      "new_tab"     => Ok(Self::NewTab),
      "select"      => Ok(Self::Select),
      "ack"         => Ok(Self::Ack),
      "yes"         => Ok(Self::Yes),
      "no"          => Ok(Self::No),
      "cancel"      => Ok(Self::Cancel),
      s => Err(format!("Keys table does not contain field {s}")),
    }
  }
}
impl Action {
  pub fn update(&self, view: &mut crate::PageView) {
    match self {
      Action::PageDown  => {view.move_down(15);}
      Action::PageUp    => {view.move_up(15);}
      Action::Bottom    => {view.move_down(view.page.matrix.len());}
      Action::Top       => {view.move_up(view.page.matrix.len());}
      Action::MoveDown  => {view.move_down(1);}
      Action::MoveUp    => {view.move_up(1);}
      Action::MoveLeft  => {view.move_left(1);}
      Action::MoveRight => {view.move_right(1);}
      _ => {}
    }
  }

  pub fn update_edit(&self, textbox: &mut crate::PageView) {
    match self {
      Action::PageDown  => {textbox.move_left(15);}
      Action::PageUp    => {textbox.move_right(15);}
      Action::Backspace => {textbox.backspace();}
      Action::Delete    => {textbox.delete();}
      Action::Insert(c) => {textbox.insert(*c);}
      Action::MoveLeft  => {textbox.move_left(1);}
      Action::MoveRight => {textbox.move_right(1);}
      Action::MoveDown  => {textbox.move_down(1);}
      Action::MoveUp    => {textbox.move_up(1);}
      _ => {}
    }
  }
}
