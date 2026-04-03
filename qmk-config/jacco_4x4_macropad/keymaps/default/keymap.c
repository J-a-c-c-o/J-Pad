
#include QMK_KEYBOARD_H
#include "timer.h"

#define NUM_MACROS 16
#define MAX_MACRO_LEN 64
#define NUM_LAYERS 16
#define MAX_MACRO_KEYS 5

typedef struct
{
    uint16_t keys[MAX_MACRO_KEYS];
    uint8_t key_count;
} macro_t;

macro_t macros[NUM_LAYERS][NUM_MACROS][MAX_MACRO_LEN];
uint8_t macro_len[NUM_LAYERS][NUM_MACROS];
int16_t macro_repeating[NUM_LAYERS][NUM_MACROS];

uint16_t encoder_actions[NUM_LAYERS][2];

enum custom_keycodes
{
    SWITCH_LAYER_0 = SAFE_RANGE,
    SWITCH_LAYER_1,
    SWITCH_LAYER_2,
    SWITCH_LAYER_3,
    SWITCH_LAYER_4,
    SWITCH_LAYER_5,
    SWITCH_LAYER_6,
    SWITCH_LAYER_7,
    SWITCH_LAYER_8,
    SWITCH_LAYER_9,
    SWITCH_LAYER_10,
    SWITCH_LAYER_11,
    SWITCH_LAYER_12,
    SWITCH_LAYER_13,
    SWITCH_LAYER_14,
    SWITCH_LAYER_15
};

void load_default_macros(void)
{
    for (uint8_t l = 0; l < NUM_LAYERS; l++)
    {
        for (uint8_t m = 0; m < NUM_MACROS; m++)
        {
            macros[l][m][0].keys[0] = KC_NO;
            macros[l][m][0].key_count = 0;
            macro_len[l][m] = 0;
            macro_repeating[l][m] = -1;
        }

        macros[l][0][0].keys[0] = KC_MPRV;
        macros[l][0][0].key_count = 1;
        macro_len[l][0] = 1;
        macros[l][1][0].keys[0] = KC_MPLY;
        macros[l][1][0].key_count = 1;
        macro_len[l][1] = 1;
        macros[l][2][0].keys[0] = KC_MNXT;
        macros[l][2][0].key_count = 1;
        macro_len[l][2] = 1;
        macros[l][3][0].keys[0] = SWITCH_LAYER_0;
        macros[l][3][0].key_count = 1;
        macro_len[l][3] = 1;

        macros[l][6][0].keys[0] = KC_LCTL;
        macros[l][6][0].keys[1] = KC_C;
        macros[l][6][0].key_count = 2;
        macro_len[l][6] = 1;
        macros[l][7][0].keys[0] = KC_MUTE;
        macros[l][7][0].key_count = 1;
        macro_len[l][7] = 1;
        macros[l][11][0].keys[0] = KC_MS_BTN1;
        macros[l][11][0].key_count = 1;
        macro_len[l][11] = 1;
        macro_repeating[l][11] = 100;
        macros[l][12][0].keys[0] = SWITCH_LAYER_1;
        macros[l][12][0].key_count = 1;
        macro_len[l][12] = 1;
        macros[l][13][0].keys[0] = SWITCH_LAYER_2;
        macros[l][13][0].key_count = 1;
        macro_len[l][13] = 1;
        macros[l][14][0].keys[0] = SWITCH_LAYER_3;
        macros[l][14][0].key_count = 1;
        macro_len[l][14] = 1;
        macros[l][15][0].keys[0] = SWITCH_LAYER_4;
        macros[l][15][0].key_count = 1;
        macro_len[l][15] = 1;
    }
    encoder_actions[0][0] = KC_VOLD;
    encoder_actions[0][1] = KC_VOLU;
    encoder_actions[1][0] = KC_LEFT;
    encoder_actions[1][1] = KC_RIGHT;
    encoder_actions[2][0] = C(KC_MINS);
    encoder_actions[2][1] = C(KC_EQL);
    encoder_actions[3][0] = KC_WH_D;
    encoder_actions[3][1] = KC_WH_U;
    encoder_actions[4][0] = KC_MPRV;
    encoder_actions[4][1] = KC_MNXT;
    for (uint8_t l = 5; l < NUM_LAYERS; l++)
    {
        encoder_actions[l][0] = KC_VOLD;
        encoder_actions[l][1] = KC_VOLU;
    }
}

void keyboard_post_init_user(void)
{
    load_default_macros();
}

bool encoder_update_user(uint8_t index, bool clockwise)
{
    if (index != 0)
        return false;
    uint8_t layer = get_highest_layer(layer_state);
    if (layer >= NUM_LAYERS)
        layer = 0;
    uint16_t action = encoder_actions[layer][clockwise ? 1 : 0];
    tap_code16(action);
    return false;
}

void consume_macro_keycodes(uint8_t layer, uint8_t macro_index, uint8_t macro_length, uint8_t **data_ptr)
{
    for (uint8_t i = 0; i < macro_length && i < MAX_MACRO_LEN; i++)
    {
        uint8_t actions_length = (*data_ptr)[0];
        macros[layer][macro_index][i].key_count = actions_length;
        printf("Macro has %d actions\n", actions_length);
        (*data_ptr)++;

        for (uint8_t j = 0; j < actions_length && j < MAX_MACRO_KEYS; j++)
        {
            printf("Reading keycode for layer %d, macro %d, action %d: 0x%04x\n", layer, macro_index, i, (*data_ptr)[0]);
            macros[layer][macro_index][i].keys[j] = (*data_ptr)[0];
            (*data_ptr)++;
        }
    }
}

void raw_hid_receive(uint8_t *data, uint8_t length)
{
    uint8_t response[length];
    memset(response, 0, length);
    response[0] = 0xFF;

    if (length < 3)
    {
        printf("Received invalid data over raw HID (length %d)\n", length);
        return;
    }
    uint8_t command = data[0];
    if (command == 0)
    {
        uint8_t layer = data[1];
        uint8_t macro_index = data[2];
        uint8_t macro_length = data[3];
        printf("Received macro update for layer %d, macro %d, length %d\n", layer, macro_index, macro_length);
        if (layer < NUM_LAYERS && macro_index < NUM_MACROS && macro_length <= MAX_MACRO_LEN)
        {
            macro_len[layer][macro_index] = macro_length;
            uint8_t *data_ptr = &data[4];
            consume_macro_keycodes(layer, macro_index, macro_length, &data_ptr);
        }
    }
    else if (command == 1)
    {
        uint8_t layer = data[1];
        uint8_t clockwise_action = data[2];
        uint8_t counterclockwise_action = data[3];
        printf("Received encoder update for layer %d, clockwise %d, counterclockwise %d\n", layer, clockwise_action, counterclockwise_action);
        if (layer < NUM_LAYERS)
        {
            encoder_actions[layer][0] = counterclockwise_action;
            encoder_actions[layer][1] = clockwise_action;
        }
    }
}

const uint16_t PROGMEM keymaps[NUM_LAYERS][MATRIX_ROWS][MATRIX_COLS] = {
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

bool handle_macro(macro_t *macro, keyrecord_t *record)
{
    if (record == NULL || record->event.pressed)
    {
        for (uint8_t i = 0; i < macro->key_count; i++)
        {
            if (macro->keys[i] >= SWITCH_LAYER_0 && macro->keys[i] <= SWITCH_LAYER_15)
            {
                uint8_t target_layer = macro->keys[i] - SWITCH_LAYER_0;
                layer_move(target_layer);
            }
            else
            {
                register_code16(macro->keys[i]);
                printf("Registered keycode 0x%04x\n", macro->keys[i]);
            }
        }

        for (uint8_t i = 0; i < macro->key_count; i++)
        {
            if (macro->keys[i] >= SWITCH_LAYER_0 && macro->keys[i] <= SWITCH_LAYER_15)
            {
                continue;
            }
            unregister_code16(macro->keys[i]);
            printf("Unregistered keycode 0x%04x\n", macro->keys[i]);
        }
    }
    return true;
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

    if (keycode < NUM_MACROS && record->event.pressed)
    {
        uint8_t layer = get_highest_layer(layer_state);
        uint8_t len = macro_len[layer][keycode];
        int16_t repeating = macro_repeating[layer][keycode];

        for (uint8_t i = 0; i < len; i++)
        {
            handle_macro(&macros[layer][keycode][i], record);
        }

        if (repeating >= 0)
        {
            macro_repeat_active = true;
            macro_repeat_key = keycode;
            macro_repeat_timer = timer_read();
        }
        return false;
    }

    return true;
}

void matrix_scan_user(void)
{
    if (macro_repeat_active && macro_repeat_key < NUM_MACROS)
    {
        uint8_t layer = get_highest_layer(layer_state);
        int16_t repeating_interval = macro_repeating[layer][macro_repeat_key];
        if (timer_elapsed(macro_repeat_timer) >= repeating_interval)
        {
            uint8_t len = macro_len[layer][macro_repeat_key];

            if (len > 0)
            {
                for (uint8_t i = 0; i < len; i++)
                {
                    handle_macro(&macros[layer][macro_repeat_key][i], NULL);
                }
            }
            macro_repeat_timer = timer_read();
        }
    }
}