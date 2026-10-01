use std::collections::HashMap;
use std::error::Error;
use std::io::Error as IoError;
use std::sync::OnceLock;

const QK_LCTL: u16 = 0x0100;
const QK_LSFT: u16 = 0x0200;
const QK_LALT: u16 = 0x0400;
const QK_LGUI: u16 = 0x0800;
const QK_RCTL: u16 = 0x1100;
const QK_RSFT: u16 = 0x1200;
const QK_RALT: u16 = 0x1400;
const QK_RGUI: u16 = 0x1800;
const QK_USER: u16 = 0x7E40;

static CONSTANTS: OnceLock<Vec<(String, u16)>> = OnceLock::new();
static NAMES: OnceLock<HashMap<u16, String>> = OnceLock::new();
static MOD_EXPRESSIONS: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();

const MOD_NAMES: &[(&str, &[&str])] = &[
    ("LCTL", &["QK_LCTL"]),
    ("LSFT", &["QK_LSFT"]),
    ("LALT", &["QK_LALT"]),
    ("LGUI", &["QK_LGUI"]),
    ("RCTL", &["QK_RCTL"]),
    ("RSFT", &["QK_RSFT"]),
    ("RALT", &["QK_RALT"]),
    ("RGUI", &["QK_RGUI"]),
    ("LCS", &["QK_LCTL", "QK_LSFT"]),
    ("LCA", &["QK_LCTL", "QK_LALT"]),
    ("LCG", &["QK_LCTL", "QK_LGUI"]),
    ("LSA", &["QK_LSFT", "QK_LALT"]),
    ("LSG", &["QK_LSFT", "QK_LGUI"]),
    ("LAG", &["QK_LALT", "QK_LGUI"]),
    ("LCSG", &["QK_LCTL", "QK_LSFT", "QK_LGUI"]),
    ("LCAG", &["QK_LCTL", "QK_LALT", "QK_LGUI"]),
    ("LSAG", &["QK_LSFT", "QK_LALT", "QK_LGUI"]),
    ("RCS", &["QK_RCTL", "QK_RSFT"]),
    ("RCA", &["QK_RCTL", "QK_RALT"]),
    ("RCG", &["QK_RCTL", "QK_RGUI"]),
    ("RSA", &["QK_RSFT", "QK_RALT"]),
    ("RSG", &["QK_RSFT", "QK_RGUI"]),
    ("RAG", &["QK_RALT", "QK_RGUI"]),
    ("RCSG", &["QK_RCTL", "QK_RSFT", "QK_RGUI"]),
    ("RCAG", &["QK_RCTL", "QK_RALT", "QK_RGUI"]),
    ("RSAG", &["QK_RSFT", "QK_RALT", "QK_RGUI"]),
    ("HYPR", &["QK_LCTL", "QK_LSFT", "QK_LALT", "QK_LGUI"]),
    ("MEH", &["QK_LCTL", "QK_LSFT", "QK_LALT"]),
];

const MOD_ALIASES: &[(&str, &str)] = &[("C", "LCTL"), ("S", "LSFT"), ("A", "LALT"), ("G", "LGUI")];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
    Delete,
    Transparent,
    No,
}

impl Key {
    fn as_name(self) -> &'static str {
        match self {
            Key::A => "KC_A",
            Key::B => "KC_B",
            Key::C => "KC_C",
            Key::D => "KC_D",
            Key::E => "KC_E",
            Key::F => "KC_F",
            Key::G => "KC_G",
            Key::H => "KC_H",
            Key::I => "KC_I",
            Key::J => "KC_J",
            Key::K => "KC_K",
            Key::L => "KC_L",
            Key::M => "KC_M",
            Key::N => "KC_N",
            Key::O => "KC_O",
            Key::P => "KC_P",
            Key::Q => "KC_Q",
            Key::R => "KC_R",
            Key::S => "KC_S",
            Key::T => "KC_T",
            Key::U => "KC_U",
            Key::V => "KC_V",
            Key::W => "KC_W",
            Key::X => "KC_X",
            Key::Y => "KC_Y",
            Key::Z => "KC_Z",
            Key::Num0 => "KC_0",
            Key::Num1 => "KC_1",
            Key::Num2 => "KC_2",
            Key::Num3 => "KC_3",
            Key::Num4 => "KC_4",
            Key::Num5 => "KC_5",
            Key::Num6 => "KC_6",
            Key::Num7 => "KC_7",
            Key::Num8 => "KC_8",
            Key::Num9 => "KC_9",
            Key::Enter => "KC_ENTER",
            Key::Escape => "KC_ESCAPE",
            Key::Backspace => "KC_BACKSPACE",
            Key::Tab => "KC_TAB",
            Key::Space => "KC_SPACE",
            Key::Delete => "KC_DELETE",
            Key::Transparent => "KC_TRANSPARENT",
            Key::No => "KC_NO",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    Lctl,
    Lsft,
    Lalt,
    Lgui,
    Rctl,
    Rsft,
    Ralt,
    Rgui,
}

impl Modifier {
    fn as_qk_name(self) -> &'static str {
        match self {
            Modifier::Lctl => "QK_LCTL",
            Modifier::Lsft => "QK_LSFT",
            Modifier::Lalt => "QK_LALT",
            Modifier::Lgui => "QK_LGUI",
            Modifier::Rctl => "QK_RCTL",
            Modifier::Rsft => "QK_RSFT",
            Modifier::Ralt => "QK_RALT",
            Modifier::Rgui => "QK_RGUI",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ModCombo {
    Lcs,
    Lca,
    Lcg,
    Lsa,
    Lsg,
    Lag,
    Lcsg,
    Lcag,
    Lsag,
    Rcs,
    Rca,
    Rcg,
    Rsa,
    Rsg,
    Rag,
    Rcsg,
    Rcag,
    Rsag,
    Hypr,
    Meh,
}

impl ModCombo {
    fn as_expression(self) -> &'static str {
        match self {
            ModCombo::Lcs => "LCS",
            ModCombo::Lca => "LCA",
            ModCombo::Lcg => "LCG",
            ModCombo::Lsa => "LSA",
            ModCombo::Lsg => "LSG",
            ModCombo::Lag => "LAG",
            ModCombo::Lcsg => "LCSG",
            ModCombo::Lcag => "LCAG",
            ModCombo::Lsag => "LSAG",
            ModCombo::Rcs => "RCS",
            ModCombo::Rca => "RCA",
            ModCombo::Rcg => "RCG",
            ModCombo::Rsa => "RSA",
            ModCombo::Rsg => "RSG",
            ModCombo::Rag => "RAG",
            ModCombo::Rcsg => "RCSG",
            ModCombo::Rcag => "RCAG",
            ModCombo::Rsag => "RSAG",
            ModCombo::Hypr => "HYPR",
            ModCombo::Meh => "MEH",
        }
    }

    fn mod_names(self) -> &'static [&'static str] {
        mod_names(self.as_expression())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum KeyExpr {
    Key(Key),
    SwitchLayer(u8),
    Raw(u16),
    Mod(Modifier, Box<KeyExpr>),
    Combo(ModCombo, Box<KeyExpr>),
    Named(&'static str),
}

#[allow(dead_code)]
pub struct KeyExprBuilder;

#[allow(dead_code)]
impl KeyExprBuilder {
    pub fn key(key: Key) -> KeyExpr {
        KeyExpr::Key(key)
    }

    pub fn switch_layer(layer: u8) -> KeyExpr {
        KeyExpr::SwitchLayer(layer)
    }

    pub fn raw(value: u16) -> KeyExpr {
        KeyExpr::Raw(value)
    }

    pub fn named(name: &'static str) -> KeyExpr {
        KeyExpr::Named(name)
    }

    pub fn with_mod(modifier: Modifier, expr: KeyExpr) -> KeyExpr {
        KeyExpr::Mod(modifier, Box::new(expr))
    }

    pub fn with_combo(combo: ModCombo, expr: KeyExpr) -> KeyExpr {
        KeyExpr::Combo(combo, Box::new(expr))
    }
}

fn constants() -> &'static [(String, u16)] {
    CONSTANTS.get_or_init(build_constants)
}

fn build_constants() -> Vec<(String, u16)> {
    let mut constants: Vec<(String, u16)> = Vec::new();
    let mut push = |name: String, value: u16| constants.push((name, value));

    push("KC_NO".to_string(), 0x0000);
    push("KC_TRANSPARENT".to_string(), 0x0001);

    for (offset, letter) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().enumerate() {
        push(format!("KC_{letter}"), 0x0004 + offset as u16);
    }

    for (offset, number) in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]
        .iter()
        .enumerate()
    {
        push(format!("KC_{number}"), 0x001E + offset as u16);
    }

    for (name, value) in [
        ("KC_ENTER", 0x0028),
        ("KC_ESCAPE", 0x0029),
        ("KC_BACKSPACE", 0x002A),
        ("KC_TAB", 0x002B),
        ("KC_SPACE", 0x002C),
        ("KC_MINUS", 0x002D),
        ("KC_EQUAL", 0x002E),
        ("KC_LEFT_BRACKET", 0x002F),
        ("KC_RIGHT_BRACKET", 0x0030),
        ("KC_BACKSLASH", 0x0031),
        ("KC_NONUS_HASH", 0x0032),
        ("KC_SEMICOLON", 0x0033),
        ("KC_QUOTE", 0x0034),
        ("KC_GRAVE", 0x0035),
        ("KC_COMMA", 0x0036),
        ("KC_DOT", 0x0037),
        ("KC_SLASH", 0x0038),
        ("KC_CAPS_LOCK", 0x0039),
        ("KC_F1", 0x003A),
        ("KC_F2", 0x003B),
        ("KC_F3", 0x003C),
        ("KC_F4", 0x003D),
        ("KC_F5", 0x003E),
        ("KC_F6", 0x003F),
        ("KC_F7", 0x0040),
        ("KC_F8", 0x0041),
        ("KC_F9", 0x0042),
        ("KC_F10", 0x0043),
        ("KC_F11", 0x0044),
        ("KC_F12", 0x0045),
        ("KC_PRINT_SCREEN", 0x0046),
        ("KC_SCROLL_LOCK", 0x0047),
        ("KC_PAUSE", 0x0048),
        ("KC_INSERT", 0x0049),
        ("KC_HOME", 0x004A),
        ("KC_PAGE_UP", 0x004B),
        ("KC_DELETE", 0x004C),
        ("KC_END", 0x004D),
        ("KC_PAGE_DOWN", 0x004E),
        ("KC_RIGHT", 0x004F),
        ("KC_LEFT", 0x0050),
        ("KC_DOWN", 0x0051),
        ("KC_UP", 0x0052),
        ("KC_NUM_LOCK", 0x0053),
        ("KC_KP_SLASH", 0x0054),
        ("KC_KP_ASTERISK", 0x0055),
        ("KC_KP_MINUS", 0x0056),
        ("KC_KP_PLUS", 0x0057),
        ("KC_KP_ENTER", 0x0058),
        ("KC_KP_1", 0x0059),
        ("KC_KP_2", 0x005A),
        ("KC_KP_3", 0x005B),
        ("KC_KP_4", 0x005C),
        ("KC_KP_5", 0x005D),
        ("KC_KP_6", 0x005E),
        ("KC_KP_7", 0x005F),
        ("KC_KP_8", 0x0060),
        ("KC_KP_9", 0x0061),
        ("KC_KP_0", 0x0062),
        ("KC_KP_DOT", 0x0063),
        ("KC_NONUS_BACKSLASH", 0x0064),
        ("KC_APPLICATION", 0x0065),
        ("KC_KB_POWER", 0x0066),
        ("KC_KP_EQUAL", 0x0067),
        ("KC_F13", 0x0068),
        ("KC_F14", 0x0069),
        ("KC_F15", 0x006A),
        ("KC_F16", 0x006B),
        ("KC_F17", 0x006C),
        ("KC_F18", 0x006D),
        ("KC_F19", 0x006E),
        ("KC_F20", 0x006F),
        ("KC_F21", 0x0070),
        ("KC_F22", 0x0071),
        ("KC_F23", 0x0072),
        ("KC_F24", 0x0073),
        ("KC_EXECUTE", 0x0074),
        ("KC_HELP", 0x0075),
        ("KC_MENU", 0x0076),
        ("KC_SELECT", 0x0077),
        ("KC_STOP", 0x0078),
        ("KC_AGAIN", 0x0079),
        ("KC_UNDO", 0x007A),
        ("KC_CUT", 0x007B),
        ("KC_COPY", 0x007C),
        ("KC_PASTE", 0x007D),
        ("KC_FIND", 0x007E),
        ("KC_KB_MUTE", 0x007F),
        ("KC_KB_VOLUME_UP", 0x0080),
        ("KC_KB_VOLUME_DOWN", 0x0081),
        ("KC_LOCKING_CAPS_LOCK", 0x0082),
        ("KC_LOCKING_NUM_LOCK", 0x0083),
        ("KC_LOCKING_SCROLL_LOCK", 0x0084),
        ("KC_KP_COMMA", 0x0085),
        ("KC_KP_EQUAL_AS400", 0x0086),
        ("KC_INTERNATIONAL_1", 0x0087),
        ("KC_INTERNATIONAL_2", 0x0088),
        ("KC_INTERNATIONAL_3", 0x0089),
        ("KC_INTERNATIONAL_4", 0x008A),
        ("KC_INTERNATIONAL_5", 0x008B),
        ("KC_INTERNATIONAL_6", 0x008C),
        ("KC_INTERNATIONAL_7", 0x008D),
        ("KC_INTERNATIONAL_8", 0x008E),
        ("KC_INTERNATIONAL_9", 0x008F),
        ("KC_LANGUAGE_1", 0x0090),
        ("KC_LANGUAGE_2", 0x0091),
        ("KC_LANGUAGE_3", 0x0092),
        ("KC_LANGUAGE_4", 0x0093),
        ("KC_LANGUAGE_5", 0x0094),
        ("KC_LANGUAGE_6", 0x0095),
        ("KC_LANGUAGE_7", 0x0096),
        ("KC_LANGUAGE_8", 0x0097),
        ("KC_LANGUAGE_9", 0x0098),
        ("KC_ALTERNATE_ERASE", 0x0099),
        ("KC_SYSTEM_REQUEST", 0x009A),
        ("KC_CANCEL", 0x009B),
        ("KC_CLEAR", 0x009C),
        ("KC_PRIOR", 0x009D),
        ("KC_RETURN", 0x009E),
        ("KC_SEPARATOR", 0x009F),
        ("KC_OUT", 0x00A0),
        ("KC_OPER", 0x00A1),
        ("KC_CLEAR_AGAIN", 0x00A2),
        ("KC_CRSEL", 0x00A3),
        ("KC_EXSEL", 0x00A4),
        ("KC_SYSTEM_POWER", 0x00A5),
        ("KC_SYSTEM_SLEEP", 0x00A6),
        ("KC_SYSTEM_WAKE", 0x00A7),
        ("KC_AUDIO_MUTE", 0x00A8),
        ("KC_AUDIO_VOL_UP", 0x00A9),
        ("KC_AUDIO_VOL_DOWN", 0x00AA),
        ("KC_MEDIA_NEXT_TRACK", 0x00AB),
        ("KC_MEDIA_PREV_TRACK", 0x00AC),
        ("KC_MEDIA_STOP", 0x00AD),
        ("KC_MEDIA_PLAY_PAUSE", 0x00AE),
        ("KC_MEDIA_SELECT", 0x00AF),
        ("KC_MEDIA_EJECT", 0x00B0),
        ("KC_MAIL", 0x00B1),
        ("KC_CALCULATOR", 0x00B2),
        ("KC_MY_COMPUTER", 0x00B3),
        ("KC_WWW_SEARCH", 0x00B4),
        ("KC_WWW_HOME", 0x00B5),
        ("KC_WWW_BACK", 0x00B6),
        ("KC_WWW_FORWARD", 0x00B7),
        ("KC_WWW_STOP", 0x00B8),
        ("KC_WWW_REFRESH", 0x00B9),
        ("KC_WWW_FAVORITES", 0x00BA),
        ("KC_MEDIA_FAST_FORWARD", 0x00BB),
        ("KC_MEDIA_REWIND", 0x00BC),
        ("KC_BRIGHTNESS_UP", 0x00BD),
        ("KC_BRIGHTNESS_DOWN", 0x00BE),
        ("KC_CONTROL_PANEL", 0x00BF),
        ("KC_ASSISTANT", 0x00C0),
        ("KC_MISSION_CONTROL", 0x00C1),
        ("KC_LAUNCHPAD", 0x00C2),
        ("KC_MOUSE_CURSOR_UP", 0x00CD),
        ("KC_MOUSE_CURSOR_DOWN", 0x00CE),
        ("KC_MOUSE_CURSOR_LEFT", 0x00CF),
        ("KC_MOUSE_CURSOR_RIGHT", 0x00D0),
        ("KC_LEFT_CTRL", 0x00E0),
        ("KC_LEFT_SHIFT", 0x00E1),
        ("KC_LEFT_ALT", 0x00E2),
        ("KC_LEFT_GUI", 0x00E3),
        ("KC_RIGHT_CTRL", 0x00E4),
        ("KC_RIGHT_SHIFT", 0x00E5),
        ("KC_RIGHT_ALT", 0x00E6),
        ("KC_RIGHT_GUI", 0x00E7),
    ] {
        push(name.to_string(), value);
    }

    push("KC_MUTE".to_string(), 0x00A8);
    push("KC_VOLU".to_string(), 0x00A9);
    push("KC_VOLD".to_string(), 0x00AA);
    push("KC_MNXT".to_string(), 0x00AB);
    push("KC_MPRV".to_string(), 0x00AC);
    push("KC_MPLY".to_string(), 0x00AE);
    push("KC_BRIU".to_string(), 0x00BD);
    push("KC_BRID".to_string(), 0x00BE);
    push("MS_BTN1".to_string(), 0x00D1);
    push("MS_BTN2".to_string(), 0x00D2);
    push("MS_WHLU".to_string(), 0x00D9);
    push("MS_WHLD".to_string(), 0x00DA);
    push("MS_LEFT".to_string(), 0x00DB);
    push("MS_RIGHT".to_string(), 0x00DC);

    push("KC_TRNS".to_string(), 0x0001);
    push("KC_ENT".to_string(), 0x0028);
    push("KC_ESC".to_string(), 0x0029);
    push("KC_BSPC".to_string(), 0x002A);
    push("KC_SPC".to_string(), 0x002C);
    push("KC_DEL".to_string(), 0x004C);

    push("QK_LCTL".to_string(), QK_LCTL);
    push("QK_LSFT".to_string(), QK_LSFT);
    push("QK_LALT".to_string(), QK_LALT);
    push("QK_LGUI".to_string(), QK_LGUI);
    push("QK_RCTL".to_string(), QK_RCTL);
    push("QK_RSFT".to_string(), QK_RSFT);
    push("QK_RALT".to_string(), QK_RALT);
    push("QK_RGUI".to_string(), QK_RGUI);
    push("QK_USER".to_string(), QK_USER);

    for layer in 0..16 {
        push(format!("SWITCH_LAYER_{layer}"), QK_USER + layer);
    }

    constants
}

pub fn mask_to_expression(mask: u16) -> Option<&'static str> {
    mod_expression()
        .iter()
        .find(|(_, value)| **value == mask)
        .map(|(name, _)| *name)
}

pub fn catalog() -> &'static [(String, u16)] {
    constants()
}

fn error(msg: impl Into<String>) -> Box<dyn Error> {
    IoError::other(msg.into()).into()
}

fn constant(name: &str) -> Option<u16> {
    constants()
        .iter()
        .find(|(constant, _)| constant == name)
        .map(|(_, value)| *value)
}

fn require_constant(name: &str) -> Result<u16, Box<dyn Error>> {
    constant(name).ok_or_else(|| error(format!("Unknown keycode or constant: {name}")))
}

fn mod_expression() -> &'static HashMap<&'static str, u16> {
    MOD_EXPRESSIONS.get_or_init(|| {
        let mut expressions = HashMap::new();
        for (name, mod_names) in MOD_NAMES {
            let mut mask = 0u16;
            for mod_name in *mod_names {
                mask |=
                    constant(mod_name).expect("every modifier of MOD_NAMES is a known constant");
            }
            expressions.insert(*name, mask);
        }
        expressions
    })
}

fn mod_names(expression: &str) -> &'static [&'static str] {
    MOD_NAMES
        .iter()
        .find(|(name, _)| *name == expression)
        .map(|(_, names)| *names)
        .expect("only known modifier expressions are turned into names")
}

fn canonical_mod_expression(expression: &str) -> Option<&'static str> {
    let canonical = MOD_ALIASES
        .iter()
        .find(|(alias, _)| *alias == expression)
        .map(|(_, canonical)| *canonical)
        .unwrap_or(expression);

    mod_expression()
        .get_key_value(canonical)
        .map(|(name, _)| *name)
}

fn split_call(expr: &str) -> Option<(&str, &str)> {
    if !expr.ends_with(')') {
        return None;
    }

    let open_idx = expr.find('(')?;
    if open_idx == 0 {
        return None;
    }

    Some((&expr[..open_idx], &expr[open_idx + 1..expr.len() - 1]))
}

pub fn keycode(expr: &str) -> Result<u16, Box<dyn Error>> {
    let expr = expr.trim();

    if let Some(value) = constant(expr) {
        return Ok(value);
    }

    let is_hex = expr.to_ascii_lowercase().starts_with("0x");
    if is_hex || expr.chars().all(|c| c.is_ascii_digit()) {
        let number = if is_hex { &expr[2..] } else { expr };
        let radix = if is_hex { 16 } else { 10 };
        return Ok(u16::from_str_radix(number, radix)?);
    }

    let (name, raw_args) =
        split_call(expr).ok_or_else(|| error(format!("Unsupported keycode expression: {expr}")))?;

    let args: Vec<&str> = if raw_args.trim().is_empty() {
        Vec::new()
    } else {
        raw_args
            .split(',')
            .map(str::trim)
            .filter(|arg| !arg.is_empty())
            .collect()
    };

    if let Some(expression) = canonical_mod_expression(name) {
        if args.len() != 1 {
            return Err(error(format!("{name}() expects exactly one argument")));
        }

        return Ok(mod_expression()[expression] | (keycode(args[0])? & 0x00FF));
    }

    Err(error(format!(
        "Unsupported function in keycode expression: {name}"
    )))
}

pub fn keycode_to_expr(keycode: u16) -> String {
    let layer = keycode.wrapping_sub(QK_USER);
    if layer < 16 {
        return format!("SWITCH_LAYER_{layer}");
    }

    let mask = keycode & 0xFF00;
    let base = keycode & 0x00FF;
    if let Some((expression, _)) = mod_expression().iter().find(|(_, value)| **value == mask) {
        return format!("{expression}({})", name_of(base));
    }

    name_of(keycode)
}

fn name_of(keycode: u16) -> String {
    names()
        .get(&keycode)
        .cloned()
        .unwrap_or_else(|| format!("0x{keycode:04X}"))
}

fn names() -> &'static HashMap<u16, String> {
    NAMES.get_or_init(|| {
        constants()
            .iter()
            .fold(HashMap::new(), |mut names, (name, value)| {
                names.entry(*value).or_insert_with(|| name.clone());
                names
            })
    })
}

#[allow(dead_code)]
pub fn step(expressions: &[&str]) -> Result<Vec<u16>, Box<dyn Error>> {
    expressions.iter().map(|expr| keycode(expr)).collect()
}

pub fn keycode_expr(expr: &KeyExpr) -> Result<u16, Box<dyn Error>> {
    match expr {
        KeyExpr::Key(key) => require_constant(key.as_name()),
        KeyExpr::SwitchLayer(layer) => {
            if *layer > 15 {
                return Err(error("Layer out of range (0..15)"));
            }
            Ok(QK_USER + u16::from(*layer))
        }
        KeyExpr::Raw(value) => Ok(*value),
        KeyExpr::Mod(modifier, inner) => {
            Ok(require_constant(modifier.as_qk_name())? | (keycode_expr(inner)? & 0x00FF))
        }
        KeyExpr::Combo(combo, inner) => {
            let mut mask: u16 = 0;
            for mod_name in combo.mod_names() {
                mask |= require_constant(mod_name)?;
            }
            Ok(mask | (keycode_expr(inner)? & 0x00FF))
        }
        KeyExpr::Named(name) => keycode(name),
    }
}

pub fn step_expr(expressions: &[KeyExpr]) -> Result<Vec<u16>, Box<dyn Error>> {
    expressions.iter().map(keycode_expr).collect()
}
