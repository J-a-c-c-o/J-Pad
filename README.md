# J-Pad

A custom 4x4 handwired macropad with rotary encoder, a configuration GUI and a
layout that survives a reboot.

<div style="display: flex; justify-content: center; gap: 20px;">
  <img src="images/top_view.jpg" alt="Macropad Top View" style="width: 100%; max-width: 400px;">
  <img src="images/side_view.jpg" alt="Macropad Side View" style="width: 100%; max-width: 400px;">
</div>

## Overview

This project is a handwired 4x4 macropad built around a Raspberry Pi Pico and QMK firmware. The firmware maps each physical key to a macro slot, and the host application in `host/` reads and writes those macros at runtime over raw HID.

The encoder has layer-aware behavior, and the firmware supports 16 logical layers with 16 macros per layer.

Layouts live in RAM while you edit them and are written to the flash of the Pico by an explicit save, so the board keeps its layout over a reboot. The host application talks to the board with a small request/response protocol over raw HID and ships a GUI that edits the layout the board is currently running.

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
- [Rust](https://www.rust-lang.org/tools/install) (stable) for the host application
- Computer with USB port

### Tools (for assembly)

- Soldering iron and solder
- Wire strippers
- 3D printer or printing service
- Basic hand tools

## Quick Start

### Flash Firmware

1. Hold the top-left key while plugging in USB.
2. Copy `jpad_4x4_macropad_default.uf2` to the `RPI-RP2` drive.
3. The board will reboot with the loaded firmware.

### Default Behavior

The firmware maps the 16 keys to macro indices `0..15` on every layer. The actual key behavior is defined by macros, so the board is configured from the host application in `host/`.

The first boot has no stored layout, so the firmware falls back to a built-in default layout: media keys, a mute key, function keys, a repeating left mouse button, and four keys that switch to the other layers. The encoder changes the volume on every layer, except for a few layers where it scrolls, zooms, scrolls again or skips tracks.

## Host Application

The host application is a single Rust binary called `jpad`, with a GUI for the
layout and a small CLI for scripting. It lives in `host/`.

```bash
cd host
cargo run --release
```

Press **Connect** and the layout the board is running appears in the pad. Click a
key to edit its steps, delay and auto repeat, click the knob in the top right
cell for the rotary encoder, and use the layer selector to switch layers. Keys you
changed get an outline, and the panel on the right shows the device, where its
layout came from and the keys it holds.

Switching layers keeps the key you are editing open, on the new layer.

The buttons along the bottom:

- **Read** re-reads the layout from the board, your edits are discarded after you confirm.
- **Write** sends your edited layout to the board, it is live right away but lost on a reboot.
- **Save** sends it and writes it into the flash of the Pico, this is what makes it survive a reboot.
- **Reset** puts the built-in defaults back into RAM, press **Save** to keep them.

While you have changes that are not on the board yet, the pad header says
`unsaved changes` and the bottom bar repeats it next to the buttons. The pill in
the title bar shows `saved` or `not saved`, which is the state of the board's own
flash.

### Command line

```bash
cargo run --release -- show    # print the layout the board is running
cargo run --release -- verify  # read, write back, read again and save to flash
cargo run --release -- demo    # send an example layout and save it
cargo run --release            # open the GUI, the default with no arguments
```

`show` and `demo` are meant as examples. `verify` leaves
the layout as it is and only proves that the whole path works, use it after
flashing new firmware.

### Recording keys

Instead of typing keycode names, arm **Record keys** in the key editor and just
press keys: every press becomes a step. Holding Ctrl, Shift or Alt while you
press a key records the combination, so Ctrl+C becomes `LCTL(KC_C)` and
Ctrl+Shift+C becomes `LCS(KC_C)`. Press Esc or the button again to stop.

The encoder editor has the same button. Click the direction you want to fill
first, then press keys.

Switching layers has its own shortcut: pick a number next to **Switch to layer**
and press **Add layer switch**, which appends `SWITCH_LAYER_0` to
`SWITCH_LAYER_15` as a step.

### Keycode reference

The **Keycodes** button in the title bar opens a searchable list of every keycode
the board understands, grouped by purpose, with the value each one has. Click an
entry to copy its name, then paste it into a step.

### Keycode expressions

Keycodes are written the way they are written in a keymap, and the host
converts them to raw QMK keycodes:

| Expression                                      | Meaning                                                              |
| ----------------------------------------------- | -------------------------------------------------------------------- |
| `KC_A`, `KC_F13`, `KC_KB_VOLUME_UP`, `MS_BTN1`  | plain keycodes                                                       |
| `LCTL(KC_C)`, `LSFT(KC_ENT)`, `RALT(KC_DELETE)` | one modifier, `C()`, `S()`, `A()` and `G()` are short hands for them |
| `LCSG(KC_TAB)`, `HYPR(KC_ENT)`, `MEH(KC_4)`     | modifier combinations                                                |
| `SWITCH_LAYER_0` .. `SWITCH_LAYER_15`           | switch to that layer                                                 |
| `0x0104`                                        | a raw keycode                                                        |

A step may hold up to 4 keycodes that are pressed together, and a macro may hold
up to 8 steps. Inside one step you can put both a layer switch and a key, for
example `SWITCH_LAYER_2, KC_TAB` goes to layer 2 and presses the tab key.

## How it works

### Talking to the board

The board exposes a raw HID interface (`FEED:0000`, usage page `FF60`, usage
`0061`). Every report is 32 bytes and carries one command, the device answers
request/response commands with a single report.

| Command        | Value | Direction      | Purpose                                                       |
| -------------- | ----- | -------------- | ------------------------------------------------------------- |
| `SET_ENCODER`  | 0     | host to device | counterclockwise and clockwise keycode of one layer           |
| `SET_MACRO`    | 1     | host to device | steps and delay of one macro slot, fragmented                 |
| `SET_REPEAT`   | 2     | host to device | auto repeat interval of one macro slot                        |
| `GET_LAYOUT`   | 3     | both           | ask for one fragment of the whole layout                      |
| `SET_LAYOUT`   | 4     | host to device | replace the whole layout, fragmented                          |
| `SAVE_LAYOUT`  | 5     | both           | write the layout into flash, answered when the write finished |
| `RESET_LAYOUT` | 6     | both           | load the built-in defaults into RAM                           |
| `PING`         | 7     | both           | device status, used to confirm a transfer finished            |

Every value in a report is little endian.

Layouts are transferred as fragments. A request carries the total number of
fragments, the fragment index and up to 25 payload bytes, a response carries the
same for the layout the device serves. The host asks for one fragment at a time
and the device serializes the layout again for every request, so a lost fragment
is simply asked for again and neither side has to keep transfer state. A
fragmented write ends with a `PING` whose response proves the board consumed all
fragments, since raw HID reports are handled in order.

### The layout format

The layout is serialized to one little endian blob, the same format that is
stored in flash. It starts with a magic value, a version and its own length,
followed by the counts the layout was built with, then every macro slot of every
layer (step count, repeat interval, delay, and the steps with their keycodes) and
finally the encoder actions per layer. A blank layout is 1354 bytes, the largest layout
the limits allow is 19786 bytes.

The version in the header is checked on load, so a layout written by a future
firmware with a different format falls back to the defaults instead of being
applied as garbage.

### Storage

The layout is kept in the emulated EEPROM of the Pico, which is a wear leveled
region of 32 kB in the last 64 kB of the chip's flash. Only the bytes that
actually changed are written, and a save of an unchanged layout writes nothing.
The firmware uses about 104 kB of the 264 kB of RAM statically, the rest is left
for other things.

The layout is written only when the host sends `SAVE_LAYOUT`, never while you are
editing, so flash wear stays out of the way. The device answers the command after
the write finished, and the host retries if the answer is lost.

### Limits

- 16 layers, 16 macro slots per layer
- 8 steps per macro, 4 keycodes per step
- 5000 ms maximum delay and repeat interval, 0 turns auto repeat off

## Project Files

- `qmk-config/` - QMK keyboard definition and keymap sources
  - `qmk-config/jpad_4x4_macropad/keyboard.json` declares keyboard features
  - `qmk-config/jpad_4x4_macropad/keymaps/default/keymap.c` maps the keys to macro slots and runs the macros
  - `qmk-config/jpad_4x4_macropad/keymaps/default/jpad_config.c` holds the layout, serializes it and stores it in flash
  - `qmk-config/jpad_4x4_macropad/keymaps/default/jpad_protocol.c` implements the raw HID protocol
  - `qmk-config/jpad_4x4_macropad/keymaps/default/rules.mk` enables the wear leveled EEPROM
- `hardware/` - Handwiring guide, BOM, and assembly instructions
- `case/` - 3D printable case files
  - `case.3mf` - main case design
  - `case.f3d` - Fusion 360 design file
  - `stl/` - STL files for printing
- `images/` - Assembly and wiring reference photos
- `host/` - Rust host application, the `jpad` binary
  - `host/src/main.rs` - entry point and the `show`, `verify` and `demo` commands
  - `host/src/app.rs` - the GUI
  - `host/src/device.rs` - raw HID client
  - `host/src/layout.rs` - layout model and blob format
  - `host/src/keycode_converter.rs` - keycode expression parser and printer
  - `host/src/keycodes.rs` - mapping of pressed keys to keycodes, and the keycode reference

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

Copy `qmk-config/jpad_4x4_macropad/` into your local QMK firmware tree under `keyboards/`.

Example:

```bash
cp -r qmk-config/jpad_4x4_macropad ~/qmk_firmware/keyboards/
```

Then build and flash:

```bash
qmk compile -kb jpad_4x4_macropad -km default
qmk flash -kb jpad_4x4_macropad -km default
```

The size limits of the layout live in `jpad_config.h`. Raising them means raising
`WEAR_LEVELING_LOGICAL_SIZE` and `WEAR_LEVELING_BACKING_SIZE` in `rules.mk` as
well, and the blob has to stay smaller than the logical EEPROM size.

## Releases

`.github/workflows/release.yml` runs on every push to `main` and on every pull
request. It builds the QMK firmware and the `jpad` application for Linux, macOS
(Intel and Apple Silicon) and Windows, so a broken firmware or a broken build
shows up before you tag anything.

- `jpad_4x4_macropad_default.uf2`, the firmware to copy to the `RPI-RP2` drive
- `jpad-<target>`, the configuration application for your OS and architecture
  `

## Notes

- The stored layout lives in the flash of the Pico, so refashing the firmware with QMK keeps it, while the layout format version guards against incompatible changes.
- Layer switching is handled by macro keycodes such as `SWITCH_LAYER_0..SWITCH_LAYER_15`, they are plain keycodes starting at `QK_USER`, so keep the layer switch keycodes at the start of `SAFE_RANGE` in `jpad_config.h`.
- A macro runs its steps in order, the keycodes of a step are pressed and released together, and the delay of the macro is waited out after the last step. A step is over before the next one starts, so a held modifier has to be part of the same keycode, `LCTL(KC_C)` instead of `KC_LCTL` and `KC_C` in two steps.

## License

Open source hardware and software. See individual files for specific licenses.
