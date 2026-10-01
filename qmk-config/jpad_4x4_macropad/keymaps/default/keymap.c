// SPDX-License-Identifier: GPL-2.0-or-later
#include QMK_KEYBOARD_H

#include "debug.h"
#include "jpad_config.h"
#include "jpad_protocol.h"
#include "timer.h"

void keyboard_post_init_user(void)
{
    jpad_protocol_init();
    jpad_config_init();
}

bool encoder_update_user(uint8_t index, bool clockwise)
{
    if (index != 0)
    {
        return false;
    }

    uint8_t layer = get_highest_layer(layer_state | default_layer_state);
    jpad_encoder_t *encoder = jpad_config_encoder(layer);
    if (encoder != NULL)
    {
        tap_code16(clockwise ? encoder->clockwise : encoder->counterclockwise);
    }
    return false;
}

void raw_hid_receive(uint8_t *data, uint8_t length)
{
    jpad_protocol_receive(data, length);
}

static bool macro_repeat_active = false;
static uint16_t macro_repeat_timer = 0;
static uint8_t macro_repeat_key = 0;

bool process_record_user(uint16_t keycode, keyrecord_t *record)
{

    if (macro_repeat_active && keycode == macro_repeat_key && record->event.pressed)
    {
        macro_repeat_active = false;
        return false;
    }

    if (keycode < JPAD_NUM_MACROS && record->event.pressed)
    {
        uint8_t layer = get_highest_layer(layer_state | default_layer_state);
        jpad_macro_t *macro = jpad_config_macro(layer, keycode);
        if (macro != NULL)
        {
            jpad_config_run_macro(macro);

            if (macro->repeat_interval > 0)
            {
                macro_repeat_active = true;
                macro_repeat_key = keycode;
                macro_repeat_timer = timer_read();
            }
        }
        return false;
    }

    return true;
}

void matrix_scan_user(void)
{
    jpad_protocol_task();

    if (macro_repeat_active && macro_repeat_key < JPAD_NUM_MACROS)
    {
        uint8_t layer = get_highest_layer(layer_state | default_layer_state);
        jpad_macro_t *macro = jpad_config_macro(layer, macro_repeat_key);
        if (macro == NULL || macro->repeat_interval <= 0)
        {
            macro_repeat_active = false;
        }
        else if (timer_elapsed(macro_repeat_timer) >= macro->repeat_interval)
        {
            if (macro->step_count > 0)
            {
                jpad_config_run_macro(macro);
            }
            macro_repeat_timer = timer_read();
        }
    }
}

const uint16_t PROGMEM keymaps[JPAD_NUM_LAYERS][MATRIX_ROWS][MATRIX_COLS] = {
    [0] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [1] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [2] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [3] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [4] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [5] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [6] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [7] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [8] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [9] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [10] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [11] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [12] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [13] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [14] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15),
    [15] = LAYOUT(
        0, 1, 2, 3,
        4, 5, 6, 7,
        8, 9, 10, 11,
        12, 13, 14, 15)};
