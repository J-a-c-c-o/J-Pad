// SPDX-License-Identifier: GPL-2.0-or-later
#pragma once

#include QMK_KEYBOARD_H

#include <stdbool.h>
#include <stdint.h>

#define JPAD_NUM_LAYERS 16
#define JPAD_NUM_MACROS 16

#define JPAD_MAX_STEPS 8
#define JPAD_MAX_STEP_KEYS 4

#define JPAD_BLOB_MAGIC 0x504Au
#define JPAD_BLOB_VERSION 1
#define JPAD_BLOB_HEADER_SIZE 10

#define JPAD_BLOB_MAX (JPAD_BLOB_HEADER_SIZE + (JPAD_NUM_LAYERS * ((JPAD_NUM_MACROS * 5) + (JPAD_NUM_MACROS * JPAD_MAX_STEPS * (1 + 2 * JPAD_MAX_STEP_KEYS)) + 4)))
#define JPAD_BLOB_MIN (JPAD_BLOB_HEADER_SIZE + (JPAD_NUM_LAYERS * 4))

#define JPAD_EEPROM_BASE 0

enum jpad_custom_keycodes
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

_Static_assert(SWITCH_LAYER_15 - SWITCH_LAYER_0 == 15, "Layer switch keycodes must be contiguous");
_Static_assert(JPAD_BLOB_MAX <= 32768, "Blob must fit the logical EEPROM");
_Static_assert(JPAD_BLOB_MIN <= JPAD_BLOB_MAX, "Blob size limits are inconsistent");
typedef struct
{
    uint8_t key_count;
    uint16_t keys[JPAD_MAX_STEP_KEYS];
} jpad_step_t;

typedef struct
{
    jpad_step_t steps[JPAD_MAX_STEPS];
    uint8_t step_count;
    uint16_t delay_ms;
    uint16_t repeat_interval;
} jpad_macro_t;

typedef struct
{
    int16_t counterclockwise;
    int16_t clockwise;
} jpad_encoder_t;

typedef enum
{
    JPAD_LAYOUT_SOURCE_DEFAULTS = 0,
    JPAD_LAYOUT_SOURCE_FLASH = 1,
} jpad_layout_source_t;

void jpad_config_init(void);
jpad_layout_source_t jpad_config_source(void);
uint16_t jpad_config_stored_length(void);
void jpad_config_load_defaults(void);
bool jpad_config_load(void);
void jpad_config_save(void);
void jpad_config_mark_dirty(void);
bool jpad_config_is_dirty(void);

jpad_macro_t *jpad_config_macro(uint8_t layer, uint8_t macro_index);
jpad_encoder_t *jpad_config_encoder(uint8_t layer);
void jpad_config_run_macro(const jpad_macro_t *macro);
uint16_t jpad_config_serialize(uint8_t *out, uint16_t max);
bool jpad_config_deserialize(const uint8_t *data, uint16_t length);
