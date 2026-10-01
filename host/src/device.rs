use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};

use crate::layout::Layout;

const VENDOR_ID: u16 = 0xfeed;
const PRODUCT_ID: u16 = 0x0000;
const USAGE_PAGE: u16 = 0xff60;
const USAGE: u16 = 0x0061;

const REPORT_SIZE: usize = 32;

const CMD_SET_ENCODER: u8 = 0;
const CMD_SET_MACRO: u8 = 1;
const CMD_SET_REPEAT: u8 = 2;
const CMD_GET_LAYOUT: u8 = 3;
const CMD_SET_LAYOUT: u8 = 4;
const CMD_SAVE_LAYOUT: u8 = 5;
const CMD_RESET_LAYOUT: u8 = 6;
const CMD_PING: u8 = 7;

const RSP_GET_LAYOUT: u8 = 0x83;
const RSP_SAVE_LAYOUT: u8 = 0x85;
const RSP_RESET_LAYOUT: u8 = 0x86;
const RSP_PING: u8 = 0x87;
const RSP_ERROR: u8 = 0xff;

const FRAGMENT_HEADER_SIZE: usize = 7;
const FRAGMENT_PAYLOAD_SIZE: usize = REPORT_SIZE - FRAGMENT_HEADER_SIZE;
const RESPONSE_HEADER_SIZE: usize = 7;
const RESPONSE_PAYLOAD_SIZE: usize = REPORT_SIZE - RESPONSE_HEADER_SIZE;

const READ_TIMEOUT_MS: i32 = 250;
const FRAGMENT_ATTEMPTS: usize = 8;
const SAVE_ATTEMPTS: usize = 4;
const MAX_FRAGMENTS: u16 = 1024;

#[derive(Debug)]
pub enum DeviceError {
    NotFound,
    NotConnected,
    Hid(String),
    Protocol(String),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::NotFound => {
                f.write_str("no J-Pad found (check the cable and that the board is flashed)")
            }
            DeviceError::NotConnected => f.write_str("not connected to a J-Pad"),
            DeviceError::Hid(message) => write!(f, "HID error: {message}"),
            DeviceError::Protocol(message) => write!(f, "protocol error: {message}"),
        }
    }
}

impl Error for DeviceError {}

impl From<hidapi::HidError> for DeviceError {
    fn from(error: hidapi::HidError) -> Self {
        DeviceError::Hid(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutSource {
    Unknown,
    Defaults,
    Flash,
}

impl LayoutSource {
    fn describe(self) -> &'static str {
        match self {
            LayoutSource::Unknown => "layout source not reported by this firmware",
            LayoutSource::Defaults => "factory defaults, nothing in flash",
            LayoutSource::Flash => "loaded from flash",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceStatus {
    pub layers: u8,
    pub macros: u8,

    pub unsaved_changes: bool,
    pub layout_source: LayoutSource,

    pub stored_bytes: u16,
}

impl DeviceStatus {
    pub fn summary(&self) -> String {
        format!(
            "{} layers, {} macros per layer, {}",
            self.layers,
            self.macros,
            self.layout_source.describe()
        )
    }
}

pub struct Device {
    handle: HidDevice,
    transaction: u8,
    description: String,
}

impl Device {
    pub fn open() -> Result<Self, DeviceError> {
        let api = HidApi::new()?;
        let info = api
            .device_list()
            .find(|device| {
                device.vendor_id() == VENDOR_ID
                    && device.product_id() == PRODUCT_ID
                    && device.usage_page() == USAGE_PAGE
                    && device.usage() == USAGE
            })
            .ok_or(DeviceError::NotFound)?;

        let path = info.path().to_owned();
        let handle = api.open_path(&path)?;

        let manufacturer = handle
            .get_manufacturer_string()?
            .unwrap_or_else(|| "unknown".to_string());
        let product = handle
            .get_product_string()?
            .unwrap_or_else(|| "J-Pad".to_string());

        Ok(Self {
            handle,
            transaction: 0,
            description: format!("{manufacturer} {product}"),
        })
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    fn next_transaction(&mut self) -> u8 {
        self.transaction = self.transaction.wrapping_add(1).max(1);
        self.transaction
    }

    fn write_report(&self, report: &[u8]) -> Result<(), DeviceError> {
        let mut buffer = vec![0u8; REPORT_SIZE];
        let length = report.len().min(REPORT_SIZE);
        buffer[..length].copy_from_slice(&report[..length]);
        self.handle.write(&buffer)?;
        Ok(())
    }

    fn read_report(&self, timeout_ms: i32) -> Result<Option<Vec<u8>>, DeviceError> {
        let mut buffer = [0u8; REPORT_SIZE + 1];
        let read = self.handle.read_timeout(&mut buffer, timeout_ms)?;
        if read == 0 {
            return Ok(None);
        }

        let report = if read > REPORT_SIZE {
            &buffer[1..read]
        } else {
            &buffer[..read]
        };
        Ok(Some(report.to_vec()))
    }

    fn read_report_retry(
        &self,
        attempts: usize,
        timeout_ms: i32,
    ) -> Result<Option<Vec<u8>>, DeviceError> {
        for attempt in 0..attempts.max(1) {
            match self.read_report(timeout_ms) {
                Ok(Some(report)) => return Ok(Some(report)),
                Ok(None) => {}
                Err(error) => {
                    if attempt + 1 >= attempts.max(1) {
                        return Err(error);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(None)
    }

    fn await_response(
        &self,
        expected: u8,
        transaction: u8,
        attempts: usize,
        timeout_ms: i32,
    ) -> Result<Vec<u8>, DeviceError> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64 * attempts as u64);

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(DeviceError::Protocol(format!(
                    "the board did not answer 0x{expected:02x}, reflash the firmware if it is older than the layout protocol"
                )));
            }

            let read_timeout = remaining.as_millis().min(i32::MAX as u128) as i32;
            let Some(report) = self.read_report(read_timeout)? else {
                continue;
            };

            if report[0] == RSP_ERROR {
                return Err(DeviceError::Protocol(describe_error(report[1])));
            }

            if report[0] == expected && report[1] == transaction {
                return Ok(report);
            }
        }
    }

    pub fn ping(&mut self) -> Result<DeviceStatus, DeviceError> {
        let transaction = self.next_transaction();
        self.write_report(&simple_request(CMD_PING, transaction))?;

        let report =
            self.await_response(RSP_PING, transaction, FRAGMENT_ATTEMPTS, READ_TIMEOUT_MS)?;
        parse_ping(&report, transaction).map_err(DeviceError::Protocol)
    }

    pub fn read_layout(&mut self) -> Result<Layout, DeviceError> {
        let transaction = self.next_transaction();
        let mut blob: Vec<u8> = Vec::new();
        let mut expected_total: Option<u16> = None;

        for index in 0..MAX_FRAGMENTS {
            let fragment = self.request_layout_fragment(transaction, index)?;

            match expected_total {
                Some(expected) if expected != fragment.total => {
                    return Err(DeviceError::Protocol(
                        "layout changed while it was being read, please try again".to_string(),
                    ));
                }
                Some(_) => {}
                None => expected_total = Some(fragment.total),
            }

            blob.extend_from_slice(&fragment.payload);
            if fragment.index + 1 == fragment.total {
                return Layout::from_bytes(&blob)
                    .map_err(|error| DeviceError::Protocol(error.to_string()));
            }
        }

        Err(DeviceError::Protocol(
            "device did not return a complete layout".to_string(),
        ))
    }

    fn request_layout_fragment(
        &mut self,
        transaction: u8,
        index: u16,
    ) -> Result<LayoutFragment, DeviceError> {
        for attempt in 0..FRAGMENT_ATTEMPTS {
            self.write_report(&layout_request(transaction, index))?;

            let report = match self.read_report_retry(1, READ_TIMEOUT_MS) {
                Ok(Some(report)) => report,
                Ok(None) | Err(DeviceError::Hid(_)) if attempt + 1 < FRAGMENT_ATTEMPTS => {
                    continue;
                }
                Ok(None) => break,
                Err(error) => return Err(error),
            };

            if report[0] == RSP_ERROR {
                if attempt + 1 < FRAGMENT_ATTEMPTS {
                    continue;
                }
                return Err(DeviceError::Protocol(describe_error(report[1])));
            }

            if report[0] != RSP_GET_LAYOUT || report[1] != transaction {
                continue;
            }

            return parse_layout_fragment(&report, transaction, index)
                .map_err(DeviceError::Protocol);
        }

        Err(DeviceError::Protocol(format!(
            "timed out while reading layout fragment {index}"
        )))
    }

    pub fn write_layout(&mut self, layout: &Layout) -> Result<(), DeviceError> {
        let mut layout = layout.clone();
        layout.normalize();
        let blob = layout.to_bytes();
        self.write_fragmented(CMD_SET_LAYOUT, &blob)
    }

    pub fn save_layout(&mut self) -> Result<(), DeviceError> {
        let mut last_error = None;

        for attempt in 0..SAVE_ATTEMPTS {
            let transaction = self.next_transaction();
            self.write_report(&simple_request(CMD_SAVE_LAYOUT, transaction))?;

            match self.await_response(RSP_SAVE_LAYOUT, transaction, 2, 2_000) {
                Ok(_) => return Ok(()),
                Err(error) => {
                    if attempt + 1 < SAVE_ATTEMPTS {
                        eprintln!("save attempt {} failed ({error}), retrying", attempt + 1);
                    }
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.expect("at least one save attempt always runs"))
    }

    pub fn reset_layout(&mut self) -> Result<(), DeviceError> {
        let transaction = self.next_transaction();
        self.write_report(&simple_request(CMD_RESET_LAYOUT, transaction))?;
        self.await_response(
            RSP_RESET_LAYOUT,
            transaction,
            FRAGMENT_ATTEMPTS,
            READ_TIMEOUT_MS,
        )?;
        Ok(())
    }

    pub fn set_macro(
        &mut self,
        layer: u8,
        macro_index: u8,
        steps: &[Vec<u16>],
        delay_ms: u16,
    ) -> Result<(), DeviceError> {
        let mut payload = vec![layer, macro_index, steps.len() as u8];
        payload.extend_from_slice(&delay_ms.to_le_bytes());

        for step in steps {
            payload.push(step.len() as u8);
            for keycode in step {
                payload.extend_from_slice(&keycode.to_le_bytes());
            }
        }

        self.write_fragmented(CMD_SET_MACRO, &payload)
    }

    pub fn set_repeat(
        &mut self,
        layer: u8,
        macro_index: u8,
        repeat_ms: u16,
    ) -> Result<(), DeviceError> {
        self.write_report(&padded(&[
            CMD_SET_REPEAT,
            layer,
            macro_index,
            repeat_ms as u8,
            (repeat_ms >> 8) as u8,
        ]))
    }

    pub fn set_encoder(
        &mut self,
        layer: u8,
        clockwise: u16,
        counterclockwise: u16,
    ) -> Result<(), DeviceError> {
        self.write_report(&padded(&[
            CMD_SET_ENCODER,
            layer,
            (clockwise >> 8) as u8,
            clockwise as u8,
            (counterclockwise >> 8) as u8,
            counterclockwise as u8,
        ]))
    }

    fn write_fragmented(&mut self, command: u8, payload: &[u8]) -> Result<(), DeviceError> {
        let transaction = self.next_transaction();
        let total = (payload.len().div_ceil(FRAGMENT_PAYLOAD_SIZE).max(1)) as u16;

        for (index, chunk) in payload.chunks(FRAGMENT_PAYLOAD_SIZE).enumerate() {
            self.write_report(&fragment_request(
                command,
                transaction,
                total,
                index as u16,
                chunk,
            ))?;
        }

        self.ping()?;
        Ok(())
    }
}

fn describe_error(code: u8) -> String {
    match code {
        1 => "device reported an unsupported command".to_string(),
        2 => "device reported a malformed packet".to_string(),
        3 => "device rejected the layout".to_string(),
        other => format!("device reported error {other}"),
    }
}

struct LayoutFragment {
    index: u16,
    total: u16,
    payload: Vec<u8>,
}

fn padded(command: &[u8]) -> Vec<u8> {
    let mut report = vec![0u8; REPORT_SIZE];
    report[..command.len()].copy_from_slice(command);
    report
}

fn simple_request(command: u8, transaction: u8) -> Vec<u8> {
    padded(&[command, transaction])
}

fn layout_request(transaction: u8, index: u16) -> Vec<u8> {
    padded(&[CMD_GET_LAYOUT, transaction, index as u8, (index >> 8) as u8])
}

fn fragment_request(command: u8, transaction: u8, total: u16, index: u16, chunk: &[u8]) -> Vec<u8> {
    let mut report = padded(&[
        command,
        transaction,
        total as u8,
        (total >> 8) as u8,
        index as u8,
        (index >> 8) as u8,
        chunk.len() as u8,
    ]);
    report[FRAGMENT_HEADER_SIZE..FRAGMENT_HEADER_SIZE + chunk.len()].copy_from_slice(chunk);
    report
}

fn parse_layout_fragment(
    report: &[u8],
    transaction: u8,
    index: u16,
) -> Result<LayoutFragment, String> {
    if report.len() < RESPONSE_HEADER_SIZE
        || report[0] != RSP_GET_LAYOUT
        || report[1] != transaction
    {
        return Err("device sent a report that does not belong to this request".to_string());
    }

    let fragment = LayoutFragment {
        index: u16::from(report[2]) | (u16::from(report[3]) << 8),
        total: u16::from(report[4]) | (u16::from(report[5]) << 8),
        payload: Vec::new(),
    };

    let length = report[RESPONSE_HEADER_SIZE - 1] as usize;
    if fragment.index != index || fragment.total == 0 || length > RESPONSE_PAYLOAD_SIZE {
        return Err("device sent a malformed layout fragment".to_string());
    }

    Ok(LayoutFragment {
        payload: report[RESPONSE_HEADER_SIZE..RESPONSE_HEADER_SIZE + length].to_vec(),
        ..fragment
    })
}

const PING_PAYLOAD_MIN: u8 = 3;
const PING_PAYLOAD_MAX: u8 = 6;
const PING_PAYLOAD_MIN_LEN: usize = PING_PAYLOAD_MIN as usize;

fn parse_ping(report: &[u8], transaction: u8) -> Result<DeviceStatus, String> {
    if report.len() < RESPONSE_HEADER_SIZE + PING_PAYLOAD_MIN_LEN
        || report[0] != RSP_PING
        || report[1] != transaction
    {
        return Err("device sent a report that does not belong to this ping".to_string());
    }

    let length = report[RESPONSE_HEADER_SIZE - 1];
    if !(PING_PAYLOAD_MIN..=PING_PAYLOAD_MAX).contains(&length) {
        return Err(format!("device sent a status report of {length} bytes"));
    }

    let byte = |index: usize| report[RESPONSE_HEADER_SIZE + index];

    Ok(DeviceStatus {
        layers: byte(0),
        macros: byte(1),
        unsaved_changes: byte(2) != 0,
        layout_source: match length >= 4 {
            true if byte(3) == 0 => LayoutSource::Defaults,
            true => LayoutSource::Flash,
            false => LayoutSource::Unknown,
        },
        stored_bytes: match length >= 6 {
            true => u16::from(byte(4)) | (u16::from(byte(5)) << 8),
            false => 0,
        },
    })
}