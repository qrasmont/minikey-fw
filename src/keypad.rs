use crate::keycode::KeyCode;

// Represents a Key position in the matrix
pub type Key = (u8, u8);

// A list of keys to press simultanously
// Could be one or many
pub type Press = &'static [KeyCode];

// A list of presses to execute
pub type Macro = &'static [Press];

// The macro matrix:
// Macro at (0,0) is triggered by key (0,0), etc.
pub type MacroMatrix = &'static [Macro];


// Define all of our macros
pub const TMUX_LEAD: Press = &[KeyCode::LEFTCTRL, KeyCode::B];
pub const TMUX_NEXT: Press = &[KeyCode::N];
pub const TMUX_PREV: Press = &[KeyCode::P];

pub const TMUX_NEXT_MACRO: Macro = &[TMUX_LEAD, TMUX_NEXT];
pub const TMUX_PREV_MACRO: Macro = &[TMUX_LEAD, TMUX_PREV];

// Map macro actions to key matrix
#[rustfmt::skip]
pub const MACRO_MATRIX: MacroMatrix = &[
    TMUX_PREV_MACRO, TMUX_NEXT_MACRO //, Key2, Key3,
    //Key4, Key5, Key6, Key7,
    //Key8, Key9, Key10, Key11,
    //Key12, Key13, Key14, Key15,
];

