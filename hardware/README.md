# Hardware - Handwired Build

## Overview

This is a handwired 4x4 macropad using a Raspberry Pi Pico microcontroller. No custom PCB required!

### Key Features
- Raspberry Pi Pico microcontroller
- 15x Cherry MX switches + 1x rotary encoder
- Handwired matrix (4x4 grid)
- USB-C connectivity via Pico
- 3D printed case

## Bill of Materials

| Component | Quantity | Description | Example Source |
|-----------|----------|-------------|----------------|
| Raspberry Pi Pico | 1 | Microcontroller board | Any electronics supplier |
| Cherry MX switches | 15 | Mechanical switches | NovelKeys, Gateron, etc. |
| Rotary encoder | 1 | EC11 type recommended | Any electronics supplier |
| Diodes 1N4148 | 16 | Through-hole diodes | Any electronics supplier |
| Wire | ~2m | 30AWG wire wrap wire | Amazon, electronics stores |
| Screws | 4 | 3M 10mm DIN 965 screws (or similar) | Amazon, hardware stores |
| Keycaps | 15 | Any MX compatible | Amazon, mechanical keyboard stores |

**Special Part**: MX Switch Adapter for Encoder
- Download from [Thingiverse Thing:3770166](https://www.thingiverse.com/thing:3770166)
- 3D print this adapter to mount the encoder in an MX switch position
- Allows encoder to fit in the standard switch grid

## Wiring Diagram

### Matrix Layout
```
     Col0  Col1  Col2  Col3
     GP9   GP11  GP13  GP15
Row0 SW1   SW2   SW3   ENC   GP18
Row1 SW4   SW5   SW6   SW7   GP22
Row2 SW8   SW9   SW10  SW11  GP20
Row3 SW12  SW13  SW14  SW15  GP16
```

### Pin Assignments
```
Matrix pins:
- Rows: GP18, GP22, GP20, GP16
- Cols: GP9, GP11, GP13, GP15

Encoder pins:
- Pin A: GP26
- Pin B: GP28
```

### Diode Placement
- One 1N4148 diode per switch (including encoder position)
- Cathode (striped end) connects to column wire
- Anode connects to switch pin
- Prevents key ghosting in matrix

## Assembly Guide

See `assembly.md` for complete step-by-step handwiring instructions including:
- Matrix wiring techniques
- Pico connection details
- Testing procedures
- Troubleshooting tips

## Alternative Builds

### Without Encoder Switch
- Wire only encoder A/B pins (rotation)
- Leave switch pin unconnected
- Use regular switch in position [0,3] or leave empty

### Different Switch Count
- Can reduce to 3x4 (12 switches) if desired
- Update QMK configuration accordingly
- Adjust case design as needed

## Advantages of Handwiring
- **No PCB required** - cheaper and more accessible
- **Easy to modify** - change layout anytime
- **Learning experience** - understand how keyboards work
- **Repairable** - easy to fix broken connections

## Tools Required
- Soldering iron and solder
- Wire strippers
- Flush cutters
- Multimeter (for testing)
- 3D printer (for case and encoder adapter)

## Cost Estimate
- Raspberry Pi Pico: $4
- Switches: $15-30 (depending on type)
- Encoder: $2
- Diodes and wire: $5
- Keycaps + case: free (if printed)
- Screws: $2
- **Total: ~$20-40** (much cheaper than custom PCB)
