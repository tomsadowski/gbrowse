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
  pub fn update<T>(&self, page: &mut crate::Page<T>) {
    match self {
      Action::PageDown => {
        page.move_down(page.point_view.get_height() as usize);
      }
      Action::PageUp => {
        page.move_up(page.point_view.get_height() as usize);
      }
      Action::Bottom => {
        page.move_down(page.matrix.data.len());
      }
      Action::Top => {
        page.move_up(page.matrix.data.len());
      }
      Action::MoveDown  => {page.move_down(1);}
      Action::MoveUp    => {page.move_up(1);}
      Action::MoveLeft  => {page.move_left(1);}
      Action::MoveRight => {page.move_right(1);}
      _ => {}
    }
  }


  pub fn update_edit<T>(&self, page: &mut crate::Page<T>) {
    match self {
      Action::PageDown  => {page.move_left(15);}
      Action::PageUp    => {page.move_right(15);}
      Action::Backspace => {page.backspace();}
      Action::Delete    => {page.delete();}
      Action::Insert(c) => {page.insert(*c);}
      Action::MoveLeft  => {page.move_left(1);}
      Action::MoveRight => {page.move_right(1);}
      Action::MoveDown  => {page.move_down(1);}
      Action::MoveUp    => {page.move_up(1);}
      _ => {}
    }
  }
}
