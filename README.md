# J-Pad

A custom 4x4 handwired macropad with rotary encoder and dynamic runtime macro updates.

<div style="display: flex; justify-content: center; gap: 20px;">
  <img src="images/top_view.jpg" alt="Macropad Top View" style="width: 100%; max-width: 400px;">
  <img src="images/side_view.jpg" alt="Macropad Side View" style="width: 100%; max-width: 400px;">
</div>

## Overview

This project is a handwired 4x4 macropad built around a Raspberry Pi Pico and QMK firmware. The firmware maps each physical key to a macro slot, and the `host/` scripts can update macros at runtime over raw HID.

The encoder has layer-aware behavior, and the firmware supports 16 logical layers with 16 macros per layer.

## Requirements

### Hardware

- 1x Raspberry Pi Pico
- 16x Cherry MX compatible switches (or 15 switches plus encoder push if available)
- 1x Rotary encoder (EC11)
- 16x 1N4148 diodes
- 4x 3M 10mm DIN 965 screws (or similar)
- Hookup wire for handwiring
- Keycaps for 16 keys
- MX encoder adapter: [Thingiverse Thing:3770166](https://www.thingiverse.com/thing:3770166)

### Software

- [QMK Firmware](https://qmk.fm/) for firmware compilation/customization
- Python 3 for the UI scripts
- `hid` Python package for raw HID communication
- Computer with USB port

### Tools (for assembly)

- Soldering iron and solder
- Wire strippers
- 3D printer or printing service
- Basic hand tools

## Quick Start

### Flash Firmware

1. Hold the top-left key while plugging in USB.
2. Copy `jacco_4x4_macropad_default.uf2` to the `RPI-RP2` drive.
3. The board will reboot with the loaded firmware.

### Default Behavior

The firmware maps the 16 keys to macro indices `0..15` on every layer. The actual key behavior is defined by macros, so the board is designed to be configured at runtime via the host scripts in `host/`.

**Encoder**: Layer-dependent actions, configured in firmware.

## Host Tool / Dynamic Macro Support

This repo includes a simple Python host-side tool for sending macro updates to the board over raw HID.

- `host/main.py` - sends fragmented raw HID reports to update macro definitions on the device.
- `host/keycode_converter.py` - converts human-readable macro expressions like `LCTL(KC_C)` and `SWITCH_LAYER_5` into QMK keycodes.
- `host/qmk_keycodes.py` - contains the QMK keycode constants used by the converter.

### How it works

The firmware expects raw HID packets to update:

- encoder actions per layer
- macro contents per layer and slot
- repeating macro intervals

The sample script in `host/main.py` demonstrates:

- setting layer 0 key 4 to switch to layer 5
- setting layer 5 key 5 to send `Ctrl+C` and erpeating every 100ms until pressed again
- setting layer 5 rotary encoder actions to send `a` and `b`
- setting layer 5 key 4 to return to layer 0

### Run the host script

Install dependencies:

```bash
python3 -m pip install hid
```

Then run:

```bash
python3 host/main.py
```

Edit `host/main.py` to send the macro updates you want.

## Project Files

- `qmk-config/` - QMK keyboard definition and keymap sources
  - `qmk-config/jacco_4x4_macropad/keyboard.json` declares keyboard features
  - `qmk-config/jacco_4x4_macropad/keymaps/default/keymap.c` implements the dynamic macro engine and encoder behavior
- `hardware/` - Handwiring guide, BOM, and assembly instructions
- `case/` - 3D printable case files
  - `case.3mf` - main case design
  - `case.f3d` - Fusion 360 design file
  - `stl/` - STL files for printing
- `images/` - Assembly and wiring reference photos
- `host/` - Python scripts for sending macro updates to the device

## Building Your Own

### 1. Print Required Parts

- Print `case.3mf`
- Print the MX encoder adapter from [Thingiverse Thing:3770166](https://www.thingiverse.com/thing:3770166)
- Recommended settings: 0.2mm layer height, 20% infill, PLA or PETG

### 2. Gather Components

See `hardware/README.md` for the full Bill of Materials.

### 3. Handwire Assembly

Follow `hardware/assembly.md` to:

- wire the switch matrix
- connect the Raspberry Pi Pico
- wire the encoder to GP26/GPIO28
- install the case and finish assembly

### 4. Flash Firmware

Use the Quick Start guide above or compile from QMK after copying the keyboard folder.

## Customizing

Copy `qmk-config/jacco_4x4_macropad/` into your local QMK firmware tree under `keyboards/`.

Example:

```bash
cp -r qmk-config/jacco_4x4_macropad ~/qmk_firmware/keyboards/
```

Then build and flash:

```bash
qmk compile -kb jacco_4x4_macropad -km default
qmk flash -kb jacco_4x4_macropad -km default
```

## Notes

- The current firmware stores macro definitions in RAM only. Custom macros are lost after reboot unless resent by the host tool.
- Layer switching is handled by macro keycodes such as `SWITCH_LAYER_0..SWITCH_LAYER_15`.

## License

Open source hardware and software. See individual files for specific licenses.
