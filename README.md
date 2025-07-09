# J-Pad 

A simple 4x4 handwired macropad with rotary encoder.

<div style="display: flex; justify-content: center; gap: 20px;">
  <img src="images/top_view.jpg" alt="Macropad Top View" style="width: 100%; max-width: 400px;">
  <img src="images/side_view.jpg" alt="Macropad Side View" style="width: 100%; max-width: 400px;">
</div>

## Requirements

### Hardware
- 1x Raspberry Pi Pico
- 15x Cherry MX compatible switches
- 1x Rotary encoder (EC11)
- 16x 1N4148 diodes
- 4x 3M 10mm metal screws
- Wire for handwiring
- 15x Keycaps (any profile)
- MX encoder adapter: [Thingiverse Thing:3770166](https://www.thingiverse.com/thing:3770166)

### Software
- [QMK Firmware](https://qmk.fm/) (for customization)
- Computer with USB port

### Tools (for assembly)
- Soldering iron and solder
- Wire strippers
- 3D printer or printing service
- Basic hand tools

## Quick Start

### Flash Firmware
1. Hold top-left key while plugging in USB
2. Copy `firmware/jacco_4x4_macropad_default.uf2` to the RPI-RP2 drive
3. Done!

### Default Layout
```
┌───┬───┬───┬───┐
│ 7 │ 8 │ 9 │ / │
├───┼───┼───┼───┤
│ 4 │ 5 │ 6 │ * │
├───┼───┼───┼───┤
│ 1 │ 2 │ 3 │ - │
├───┼───┼───┼───┤
│ 0 │ . │Ent│ + │
└───┴───┴───┴───┘
```
**Encoder**: Volume up/down

## Project Files

- `firmware/` - Ready-to-flash .uf2 files
- `qmk-config/` - QMK keyboard definition (copy to your QMK install)
- `hardware/` - Handwiring guide, BOM, and assembly instructions
- `case/` - 3D printable case files
  - `case.3mf` - 3mf file for main case
  - `case.3fd` - fusion design file
  - `stl/` - STL files for printing
    - `Top.stl` - Top case part
    - `Bottom.stl` - Bottom case part
- `images/` - Assembly photos and reference images
  - `top_view.jpg` - Completed macropad from above
  - `side_view.jpg` - Side profile view
  - `rows.jpg` - Initial row wiring
  - `rows+diodes.jpg` - Rows with diodes installed
  - `rows+columns.jpg` - Complete switch matrix
  - `full_wired_on_pi_pico.jpg` - Final assembly with controller

## Building Your Own

### 1. Print Required Parts
- Print `case.3mf` (main case)
- Print MX encoder adapter from [Thingiverse Thing:3770166](https://www.thingiverse.com/thing:3770166)
- Settings: 0.2mm layer height, 20% infill, PLA or PETG

### 2. Gather Components
See `hardware/README.md` for the full Bill of Materials.

### 3. Handwire Assembly
Follow the detailed guide in `hardware/assembly.md` to:
- Wire the switch matrix
- Connect the Raspberry Pi Pico
- Install the encoder with MX adapter

### 4. Flash Firmware
Use the Quick Start guide above.

## Customizing

Copy `qmk-config/jacco_4x4_macropad/` to `~/qmk_firmware/keyboards/` then:

```bash
# Edit keymap
nano ~/qmk_firmware/keyboards/jacco_4x4_macropad/keymaps/default/keymap.c

# Edit keyboard.json if needed
nano ~/qmk_firmware/keyboards/jacco_4x4_macropad/keyboard.json

# Compile and flash
qmk flash -kb jacco_4x4_macropad -km default
```

See [QMK docs](https://docs.qmk.fm/) for more info.

## License

Open source hardware and software. See individual files for specific licenses.