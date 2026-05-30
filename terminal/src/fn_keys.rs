#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnAction {
    None,
    Copy,
    Paste,
    PasteSelection,
    ScrollUp,
    ScrollDown,
    ScrollReset,
    FontIncrease,
    FontDecrease,
    NewWindow,
    CloseWindow,
}

pub struct FnKeyHandler {
    fn_held: bool,
    ctrl_held: bool,
    shift_held: bool,
}

impl FnKeyHandler {
    pub fn new() -> Self {
        Self {
            fn_held: false,
            ctrl_held: false,
            shift_held: false,
        }
    }

    pub fn handle_key(&mut self, keycode: u32) -> Option<FnAction> {
        match keycode {
            29 => { self.ctrl_held = true; }
            42 => { self.shift_held = true; }
            56 => { self.fn_held = true; }
            _ => {}
        }

        if self.ctrl_held && self.shift_held {
            match keycode {
                103 => return Some(FnAction::ScrollUp),
                108 => return Some(FnAction::ScrollDown),
                102 => return Some(FnAction::ScrollReset),
                46 => return Some(FnAction::Copy),
                47 => return Some(FnAction::Paste),
                _ => {}
            }
        }

        if self.fn_held {
            match keycode {
                104 => return Some(FnAction::FontIncrease),
                109 => return Some(FnAction::FontDecrease),
                28 => return Some(FnAction::NewWindow),
                1 => return Some(FnAction::CloseWindow),
                _ => {}
            }
        }

        None
    }

    pub fn handle_key_release(&mut self, keycode: u32) {
        match keycode {
            29 => self.ctrl_held = false,
            42 => self.shift_held = false,
            56 => self.fn_held = false,
            _ => {}
        }
    }
}