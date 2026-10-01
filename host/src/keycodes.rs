use eframe::egui;

use crate::keycode_converter;

const QK_LCTL: u16 = 0x0100;
const QK_LSFT: u16 = 0x0200;
const QK_LALT: u16 = 0x0400;
const QK_LGUI: u16 = 0x0800;

const SHIFTED: [(egui::Key, &str); 7] = [
    (egui::Key::Pipe, "KC_BACKSLASH"),
    (egui::Key::Questionmark, "KC_SLASH"),
    (egui::Key::Colon, "KC_SEMICOLON"),
    (egui::Key::Exclamationmark, "KC_1"),
    (egui::Key::Plus, "KC_EQUAL"),
    (egui::Key::OpenCurlyBracket, "KC_LEFT_BRACKET"),
    (egui::Key::CloseCurlyBracket, "KC_RIGHT_BRACKET"),
];

pub fn key_to_keycode(key: egui::Key) -> Option<u16> {
    if let Some((_, name)) = SHIFTED.iter().find(|(candidate, _)| *candidate == key) {
        return keycode_converter::keycode(name).ok();
    }

    if (egui::Key::A as u16..=egui::Key::Z as u16).contains(&(key as u16)) {
        return Some(0x0004 + (key as u16 - egui::Key::A as u16));
    }

    if key >= egui::Key::F1 && key <= egui::Key::F12 {
        return Some(0x003A + (key as u16 - egui::Key::F1 as u16));
    }
    if key >= egui::Key::F13 && key <= egui::Key::F24 {
        return Some(0x0068 + (key as u16 - egui::Key::F13 as u16));
    }

    let value = match key {
        egui::Key::ArrowUp => 0x0052,
        egui::Key::ArrowDown => 0x0051,
        egui::Key::ArrowLeft => 0x0050,
        egui::Key::ArrowRight => 0x004F,
        egui::Key::Escape => 0x0029,
        egui::Key::Tab => 0x002B,
        egui::Key::Backspace => 0x002A,
        egui::Key::Enter => 0x0028,
        egui::Key::Space => 0x002C,
        egui::Key::Insert => 0x0049,
        egui::Key::Delete => 0x004C,
        egui::Key::Home => 0x004A,
        egui::Key::End => 0x004D,
        egui::Key::PageUp => 0x004B,
        egui::Key::PageDown => 0x004E,
        egui::Key::Copy => 0x009D,
        egui::Key::Cut => 0x009C,
        egui::Key::Paste => 0x009B,
        egui::Key::Num0 => 0x0027,
        egui::Key::Num1 => 0x001E,
        egui::Key::Num2 => 0x001F,
        egui::Key::Num3 => 0x0020,
        egui::Key::Num4 => 0x0021,
        egui::Key::Num5 => 0x0022,
        egui::Key::Num6 => 0x0023,
        egui::Key::Num7 => 0x0024,
        egui::Key::Num8 => 0x0025,
        egui::Key::Num9 => 0x0026,
        egui::Key::Minus => 0x002D,
        egui::Key::Equals => 0x002E,
        egui::Key::Comma => 0x0036,
        egui::Key::Period => 0x0037,
        egui::Key::Slash => 0x0038,
        egui::Key::Backslash => 0x0031,
        egui::Key::Semicolon => 0x0033,
        egui::Key::Quote => 0x0034,
        egui::Key::Backtick => 0x0035,
        egui::Key::OpenBracket => 0x002F,
        egui::Key::CloseBracket => 0x0030,
        _ => return None,
    };

    Some(value)
}

pub fn is_modifier_key(key: egui::Key) -> bool {
    matches!(key.name(), "Ctrl" | "Shift" | "Alt" | "Super" | "Meta")
}

pub fn modifier_mask(modifiers: egui::Modifiers) -> u16 {
    let mut mask = 0;
    if modifiers.ctrl {
        mask |= QK_LCTL;
    }
    if modifiers.shift {
        mask |= QK_LSFT;
    }
    if modifiers.alt {
        mask |= QK_LALT;
    }
    if modifiers.mac_cmd {
        mask |= QK_LGUI;
    }
    mask
}

pub fn captured_expression(key: egui::Key, modifiers: egui::Modifiers) -> Option<String> {
    let name = keycode_converter::keycode_to_expr(key_to_keycode(key)?);

    let mut mask = modifier_mask(modifiers);
    if SHIFTED.iter().any(|(symbol, _)| *symbol == key) {
        mask |= QK_LSFT;
    }

    let wrapped = match keycode_converter::mask_to_expression(mask) {
        Some(expression) if mask != 0 => format!("{expression}({name})"),
        _ => name,
    };

    Some(wrapped)
}

pub const ALIASES: [&str; 16] = [
    "KC_TRNS",
    "KC_ENT",
    "KC_ESC",
    "KC_BSPC",
    "KC_SPC",
    "KC_DEL",
    "KC_MUTE",
    "KC_VOLU",
    "KC_VOLD",
    "KC_MNXT",
    "KC_MPRV",
    "KC_MPLY",
    "KC_BRIU",
    "KC_BRID",
    "KC_MIC_MUTE",
    "QK_USER",
];

pub fn group_order(group: &str) -> usize {
    [
        "Letters",
        "Digits",
        "Function keys",
        "Editing",
        "Navigation and modifiers",
        "Modifiers",
        "Media",
        "Audio",
        "Brightness",
        "Consumer keys",
        "Mouse",
        "Layer switches",
        "Internet",
        "Keypad",
        "Other",
    ]
    .iter()
    .position(|known| *known == group)
    .unwrap_or(99)
}

pub fn group_of(name: &str) -> &'static str {
    match name {
        _ if name.starts_with("SWITCH_LAYER_") => "Layer switches",
        _ if name.starts_with("MS_") || name.starts_with("QK_MOUSE") => "Mouse",
        _ if name.starts_with("QK_L") || name.starts_with("QK_R") => "Modifiers",
        _ if name.starts_with("KC_AUDIO_") => "Audio",
        _ if name.starts_with("KC_MEDIA_") => "Media",
        _ if name.starts_with("KC_BRIGHTNESS_") => "Brightness",
        _ if name.starts_with("KC_KB_") => "Consumer keys",
        _ if name.starts_with("KC_WWW_") || name.starts_with("KC_MY_") => "Internet",
        _ if name.starts_with("KC_F") && name[3..].chars().all(|c| c.is_ascii_digit()) => {
            "Function keys"
        }
        _ if name.starts_with("KC_KP_") => "Keypad",
        _ if name.starts_with("KC_ENTER")
            || name.starts_with("KC_ESC")
            || name.starts_with("KC_BSPC")
            || name.starts_with("KC_SPC")
            || name.starts_with("KC_DEL")
            || name.starts_with("KC_TAB")
            || name.starts_with("KC_BKSP") =>
        {
            "Editing"
        }
        _ if name.starts_with("KC_")
            && name.len() == 4
            && name.as_bytes()[3].is_ascii_uppercase()
            && !name.as_bytes()[3].is_ascii_digit() =>
        {
            "Letters"
        }
        _ if name.starts_with("KC_") && name.len() == 4 && name.as_bytes()[3].is_ascii_digit() => {
            "Digits"
        }
        _ if name.starts_with("KC_") => "Navigation and modifiers",
        _ => "Other",
    }
}
