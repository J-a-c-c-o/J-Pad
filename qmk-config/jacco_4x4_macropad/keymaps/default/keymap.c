// Copyright 2023 QMK
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H

enum custom_keycodes {
    AC_TOG = SAFE_RANGE, // Auto-clicker toggle
    LAYER_SPEED
};

enum layers {
    _BASE,
    _SPEED
};

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    /*
     * ┌───┬───┬───┬───┐
     * │ 7 │ 8 │ 9 │Speed│
     * ├───┼───┼───┼───┤
     * │ 4 │ 5 │ 6 │ *  │
     * ├───┼───┼───┼───┤
     * │ 1 │ 2 │ 3 │ -  │
     * ├───┼───┼───┼───┤
     * │AC │ . │Ent│ +  │
     * └───┴───┴───┴───┘
     */
    [_BASE] = LAYOUT(
        KC_P7,   KC_P8,   KC_P9,   MO(_SPEED),
        KC_P4,   KC_P5,   KC_P6,   KC_PAST,
        KC_P1,   KC_P2,   KC_P3,   KC_PMNS,
        AC_TOG,  KC_PDOT, KC_PENT, KC_PPLS
    ),
    [_SPEED] = LAYOUT(
        _______, _______, _______, _______,
        _______, _______, _______, _______,
        _______, _______, _______, _______,
        _______, _______, _______, _______
    )
};


#include "timer.h"

static bool autoclick_enabled = false;
static uint16_t autoclick_timer = 0;
static uint16_t click_interval_ms = 100;
static const uint16_t speeds[] = {50, 100, 200, 400, 800};
static uint8_t speed_index = 1;

// Encoder: hold to change and rotate to adjust speed, just rotate to change volume
bool encoder_update_user(uint8_t index, bool clockwise) {
    if (index != 0) return false;
    if (layer_state_is(_SPEED)) {
        // On speed layer, rotate changes speed
        if (!clockwise && speed_index < (sizeof(speeds)/sizeof(speeds[0]))-1) {
            speed_index++;
        } else if (clockwise && speed_index > 0) {
            speed_index--;
        }
        click_interval_ms = speeds[speed_index];
    } else {
        // On base layer, rotate changes volume
        if (clockwise) {
            tap_code(KC_VOLU);
        } else {
            tap_code(KC_VOLD);
        }
    }
    return false;
}

// Process custom keycodes
bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    switch (keycode) {
        case AC_TOG:
            if (record->event.pressed) {
                autoclick_enabled = !autoclick_enabled;
                if (autoclick_enabled) {
                    autoclick_timer = timer_read();
                }
            }
            return false;
        default:
            return true;
    }
}

void matrix_scan_user(void) {
    if (!autoclick_enabled) {
        return;
    }
    if (timer_elapsed(autoclick_timer) > click_interval_ms) {
        tap_code(KC_MS_BTN1);
        autoclick_timer = timer_read();
    }
}