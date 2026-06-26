// src/constants.rs


// square
pub const A_SQR: char = '\u{250C}';
pub const B_SQR: char = '\u{2510}';
pub const C_SQR: char = '\u{2514}';
pub const D_SQR: char = '\u{2518}';

// round
pub const A_RND: char = '\u{256D}';
pub const B_RND: char = '\u{256E}';
pub const C_RND: char = '\u{2570}';
pub const D_RND: char = '\u{256F}';

// lines
pub const X_LINE: char = '\u{2500}';
pub const Y_LINE: char = '\u{2502}';

// tortoise shell square bracket (hot)
pub const OPEN_TORT:  char = '\u{2997}';
pub const CLOSE_TORT: char = '\u{2998}';

// super square bracket (hot)
pub const OPEN_SQR:  char = '\u{27E6}';
pub const CLOSE_SQR: char = '\u{27E7}';

// brack with quill (pretty good)
pub const OPEN_E:  char = '\u{2045}';
pub const CLOSE_E: char = '\u{2046}';

// integrals (not bad)
pub const OPEN_INT:  char = '\u{2320}';
pub const CLOSE_INT: char = '\u{2321}';

// ceiling / floor (not bad)
pub const OPEN_L:  char = '\u{2308}';
pub const CLOSE_L: char = '\u{230B}';


pub const MANUAL:        &str = "User manual";
pub const CHANGE_KEYS:   &str = "Change keys";
pub const CHANGE_STYLE:  &str = "Change style";
pub const VIEW_SETTINGS: &str = "View settings";
pub const MENU: [&str; 4] = [
  MANUAL, 
  CHANGE_KEYS, 
  CHANGE_STYLE,
  VIEW_SETTINGS, 
];

pub const TAB: u8 = 0;
pub const DLG: u8 = 1;
pub const MSG: u8 = 2;

pub const DATA_PATH:   &str = "gdata";
pub const SAVE_FILE:   &str = "gdata/urls";
pub const INIT_FILE:   &str = "gdata/init";
pub const STYLES_PATH: &str = "gdata/styles";
pub const KEYS_PATH:   &str = "gdata/keys";
