// Copyright 2023 QMK
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H
#include "timer.h"

/* ---------------- Encoder Modes ---------------- */

enum encoder_modes
{
    ENC_MODE_DEFAULT,
    ENC_MODE_CLICKER,
    ENC_MODE_SCRUB,
    ENC_MODE_ZOOM,
    ENC_MODE_SCROLL
};

static uint8_t encoder_mode = ENC_MODE_DEFAULT;

/* ---------------- Custom Keycodes ---------------- */

enum custom_keycodes
{
    VOL_MUTE,
    MEDIA_PLAY_PAUSE,
    MEDIA_NEXT,
    MEDIA_PREV,

    ENC_DEFAULT,
    ENC_CLICKER,
    ENC_SCRUB,
    ENC_ZOOM,
    ENC_SCROLL
};

/* ---------------- Layers ---------------- */

enum layers
{
    _BASE,
};

/* ---------------- Autoclicker ---------------- */

static bool autoclick_enabled = false;
static uint16_t autoclick_timer = 0;
static uint16_t click_interval_ms = 100;

static const uint16_t speeds[] = {50, 100, 200, 400, 800};
static uint8_t speed_index = 1;

/* ---------------- Keymaps ---------------- */

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    [_BASE] = LAYOUT(
        MEDIA_PREV, MEDIA_PLAY_PAUSE, MEDIA_NEXT, ENC_DEFAULT,
        KC_F13, KC_F14, KC_F15, VOL_MUTE,
        KC_F16, KC_F17, KC_F18, KC_F19,
        ENC_CLICKER, ENC_SCRUB, ENC_ZOOM, ENC_SCROLL),
};

/* ---------------- Encoder ---------------- */

bool encoder_update_user(uint8_t index, bool clockwise)
{

    if (index != 0)
        return false;

    switch (encoder_mode)
    {

    case ENC_MODE_DEFAULT:
        if (clockwise)
            tap_code(KC_VOLU);
        else
            tap_code(KC_VOLD);

        break;

    case ENC_MODE_CLICKER:
        if (!clockwise && speed_index < (sizeof(speeds) / sizeof(speeds[0])) - 1)
            speed_index++;

        else if (clockwise && speed_index > 0)
            speed_index--;

        click_interval_ms = speeds[speed_index];

        break;

    case ENC_MODE_SCRUB:
        if (clockwise)
            tap_code(KC_RIGHT);
        else
            tap_code(KC_LEFT);

        break;

    case ENC_MODE_ZOOM:
        if (clockwise)
            tap_code16(C(KC_EQL));
        else
            tap_code16(C(KC_MINS));

        break;

    case ENC_MODE_SCROLL:
        if (clockwise)
            tap_code(KC_WH_U);
        else
            tap_code(KC_WH_D);

        break;
    }

    return false;
}

/* ---------------- Key Processing ---------------- */

bool process_record_user(uint16_t keycode, keyrecord_t *record)
{

    switch (keycode)
    {

        /* ----- Media Keys ----- */

    case VOL_MUTE:

        if (record->event.pressed)
            tap_code(KC_MUTE);

        return false;

    case MEDIA_PLAY_PAUSE:

        if (record->event.pressed)
            tap_code(KC_MPLY);

        return false;

    case MEDIA_NEXT:

        if (record->event.pressed)
            tap_code(KC_MNXT);

        return false;

    case MEDIA_PREV:

        if (record->event.pressed)
            tap_code(KC_MPRV);

        return false;

        /* ----- Encoder Mode Toggles ----- */

    case ENC_DEFAULT:

        if (record->event.pressed)
        {
            encoder_mode = ENC_MODE_DEFAULT;
            autoclick_enabled = false;
        }

        return false;

    case ENC_CLICKER:

        if (record->event.pressed)
        {
            if (encoder_mode == ENC_MODE_CLICKER)
            {
                autoclick_enabled = false;
                encoder_mode = ENC_MODE_DEFAULT;
            }
            else
            {
                encoder_mode = ENC_MODE_CLICKER;
                autoclick_enabled = true;
                autoclick_timer = timer_read();
            }
        }

        return false;

    case ENC_SCRUB:

        if (record->event.pressed)
        {

            autoclick_enabled = false;
            encoder_mode =
                (encoder_mode == ENC_MODE_SCRUB)
                    ? ENC_MODE_DEFAULT
                    : ENC_MODE_SCRUB;
        }

        return false;

    case ENC_ZOOM:

        if (record->event.pressed)
        {
            autoclick_enabled = false;
            encoder_mode =
                (encoder_mode == ENC_MODE_ZOOM)
                    ? ENC_MODE_DEFAULT
                    : ENC_MODE_ZOOM;
        }

        return false;

    case ENC_SCROLL:

        if (record->event.pressed)
        {
            autoclick_enabled = false;
            encoder_mode =
                (encoder_mode == ENC_MODE_SCROLL)
                    ? ENC_MODE_DEFAULT
                    : ENC_MODE_SCROLL;
        }

        return false;
    }

    return true;
}

/* ---------------- Autoclick Logic ---------------- */

void matrix_scan_user(void)
{

    if (!autoclick_enabled)
        return;

    if (timer_elapsed(autoclick_timer) >= click_interval_ms)
    {
        tap_code(KC_MS_BTN1);

        autoclick_timer = timer_read();
    }
}