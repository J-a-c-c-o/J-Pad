use std::fmt;

pub const NUM_LAYERS: usize = 16;
pub const NUM_MACROS: usize = 16;
pub const MAX_STEPS: usize = 8;
pub const MAX_STEP_KEYS: usize = 4;
pub const MAX_DELAY_MS: u16 = 5000;

pub const BLOB_MAGIC: u16 = 0x504A;
pub const BLOB_VERSION: u8 = 1;
pub const BLOB_HEADER_SIZE: usize = 10;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Step {
    pub keycodes: Vec<u16>,
}

impl Step {
    pub fn new(keycodes: Vec<u16>) -> Self {
        Self { keycodes }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroSlot {
    pub steps: Vec<Step>,

    pub delay_ms: u16,

    pub repeat_ms: u16,
}

impl MacroSlot {
    pub fn step_text(&self, index: usize) -> String {
        self.steps
            .get(index)
            .map(|step| {
                step.keycodes
                    .iter()
                    .map(|keycode| crate::keycode_converter::keycode_to_expr(*keycode))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layer {
    pub macros: Vec<MacroSlot>,
    pub counterclockwise: u16,
    pub clockwise: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layout {
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutError(String);

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for LayoutError {}

impl LayoutError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Layout {
    pub fn empty() -> Self {
        Self {
            layers: (0..NUM_LAYERS)
                .map(|_| Layer {
                    macros: (0..NUM_MACROS).map(|_| MacroSlot::default()).collect(),
                    ..Layer::default()
                })
                .collect(),
        }
    }

    pub fn normalize(&mut self) {
        self.layers.truncate(NUM_LAYERS);
        while self.layers.len() < NUM_LAYERS {
            self.layers.push(Layer {
                macros: (0..NUM_MACROS).map(|_| MacroSlot::default()).collect(),
                ..Layer::default()
            });
        }

        for layer in &mut self.layers {
            layer.macros.truncate(NUM_MACROS);
            while layer.macros.len() < NUM_MACROS {
                layer.macros.push(MacroSlot::default());
            }

            for slot in &mut layer.macros {
                slot.delay_ms = slot.delay_ms.min(MAX_DELAY_MS);
                slot.steps.truncate(MAX_STEPS);
                for step in &mut slot.steps {
                    step.keycodes.truncate(MAX_STEP_KEYS);
                }
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(BLOB_HEADER_SIZE + NUM_LAYERS * 64);
        out.extend_from_slice(&BLOB_MAGIC.to_le_bytes());
        out.push(BLOB_VERSION);
        out.push(0);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.push(NUM_LAYERS as u8);
        out.push(NUM_MACROS as u8);
        out.push(MAX_STEPS as u8);
        out.push(MAX_STEP_KEYS as u8);

        for layer in &self.layers {
            for slot in &layer.macros {
                out.push(slot.steps.len() as u8);
                out.extend_from_slice(&slot.repeat_ms.to_le_bytes());
                out.extend_from_slice(&slot.delay_ms.to_le_bytes());

                for step in &slot.steps {
                    out.push(step.keycodes.len() as u8);
                    for keycode in &step.keycodes {
                        out.extend_from_slice(&keycode.to_le_bytes());
                    }
                }
            }
        }

        for layer in &self.layers {
            out.extend_from_slice(&layer.counterclockwise.to_le_bytes());
            out.extend_from_slice(&layer.clockwise.to_le_bytes());
        }

        let length = out.len() as u16;
        out[4..6].copy_from_slice(&length.to_le_bytes());
        out
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, LayoutError> {
        let mut cursor = Cursor::new(data);

        let magic = cursor.u16()?;
        let version = cursor.u8()?;
        let flags = cursor.u8()?;
        let total_length = cursor.u16()?;
        let layer_count = cursor.u8()? as usize;
        let macro_count = cursor.u8()? as usize;
        let max_steps = cursor.u8()? as usize;
        let max_step_keys = cursor.u8()? as usize;

        if magic != BLOB_MAGIC {
            return Err(LayoutError::new("layout has an unexpected magic value"));
        }
        if version != BLOB_VERSION {
            return Err(LayoutError::new(format!(
                "unsupported layout version {version}, this build speaks version {BLOB_VERSION}"
            )));
        }
        if flags != 0 {
            return Err(LayoutError::new("layout has unsupported flags"));
        }
        if total_length as usize != data.len() {
            return Err(LayoutError::new(format!(
                "layout length mismatch: header says {total_length}, got {}",
                data.len()
            )));
        }
        if layer_count > NUM_LAYERS || macro_count > NUM_MACROS {
            return Err(LayoutError::new(
                "layout has more layers or macros than supported",
            ));
        }
        if max_steps > MAX_STEPS || max_step_keys > MAX_STEP_KEYS {
            return Err(LayoutError::new("layout uses larger steps than supported"));
        }

        let mut layout = Layout::empty();
        for layer in layout.layers.iter_mut().take(layer_count) {
            for slot in layer.macros.iter_mut().take(macro_count) {
                let step_count = cursor.u8()? as usize;
                let repeat_ms = cursor.u16()?;
                let delay_ms = cursor.u16()?;

                if step_count > max_steps {
                    return Err(LayoutError::new("layout step count out of range"));
                }

                slot.repeat_ms = if repeat_ms == u16::MAX { 0 } else { repeat_ms };
                slot.delay_ms = delay_ms;
                slot.steps = Vec::with_capacity(step_count);

                for _ in 0..step_count {
                    let key_count = cursor.u8()? as usize;
                    if key_count > max_step_keys {
                        return Err(LayoutError::new("layout key count out of range"));
                    }

                    let mut keycodes = Vec::with_capacity(key_count);
                    for _ in 0..key_count {
                        keycodes.push(cursor.u16()?);
                    }
                    slot.steps.push(Step { keycodes });
                }
            }
        }

        for layer in layout.layers.iter_mut().take(layer_count) {
            layer.counterclockwise = cursor.u16()?;
            layer.clockwise = cursor.u16()?;
        }

        if !cursor.is_done() {
            return Err(LayoutError::new("layout contains trailing data"));
        }

        Ok(layout)
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], LayoutError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| LayoutError::new("layout is longer than the addressable range"))?;
        if end > self.data.len() {
            return Err(LayoutError::new("layout is truncated"));
        }
        let slice = &self.data[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, LayoutError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, LayoutError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn is_done(&self) -> bool {
        self.offset == self.data.len()
    }
}