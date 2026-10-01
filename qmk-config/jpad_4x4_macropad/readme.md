# J-Pad 4x4

![J-Pad 4x4](../../images/top_view.jpg)

A 4x4 grid of rotary encoders with a push switch in each one, on a Raspberry Pi
Pico. Every key and every encoder can be mapped separately, and the encoder
pushes can each have their own tap, hold and double tap actions.

The layout lives in the flash of the Pico, so it survives a reboot and a
reflash of the firmware.

For wiring, the guide for building it and the host application that edits the
layout, see the [project readme](../../README.md).

## Flashing

Hold the top left encoder while plugging the board in, a `RPI-RP2` drive
appears, and `jpad_4x4_macropad_default.uf2` from the releases page is copied to
it.
