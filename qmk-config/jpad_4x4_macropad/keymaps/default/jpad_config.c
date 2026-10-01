// SPDX-License-Identifier: GPL-2.0-or-later
#include "jpad_config.h"

#include <string.h>

#include "debug.h"
#include "util.h"
#include "wear_leveling.h"

static jpad_macro_t macros[JPAD_NUM_LAYERS][JPAD_NUM_MACROS];
static jpad_encoder_t encoders[JPAD_NUM_LAYERS];
static bool layout_dirty = false;
static jpad_layout_source_t layout_source = JPAD_LAYOUT_SOURCE_DEFAULTS;
static uint16_t stored_length = 0;

static uint8_t persist_buffer[JPAD_BLOB_MAX];

static bool is_layer_keycode(uint16_t keycode) {
  return keycode >= SWITCH_LAYER_0 && keycode <= SWITCH_LAYER_15;
}

typedef struct {
  uint8_t *buffer;
  uint16_t length;
  uint16_t capacity;
  bool ok;
} blob_writer_t;

typedef struct {
  const uint8_t *buffer;
  uint16_t length;
  uint16_t offset;
  bool ok;
} blob_reader_t;

static void writer_put_u8(blob_writer_t *writer, uint8_t value) {
  if (!writer->ok || writer->length + 1 > writer->capacity) {
    writer->ok = false;
    return;
  }
  writer->buffer[writer->length++] = value;
}

static void writer_put_u16(blob_writer_t *writer, uint16_t value) {
  writer_put_u8(writer, (uint8_t)(value & 0xFF));
  writer_put_u8(writer, (uint8_t)(value >> 8));
}

static bool reader_take_u8(blob_reader_t *reader, uint8_t *out) {
  if (!reader->ok || reader->offset + 1 > reader->length) {
    reader->ok = false;
    return false;
  }
  *out = reader->buffer[reader->offset++];
  return true;
}

static bool reader_take_u16(blob_reader_t *reader, uint16_t *out) {
  uint8_t low, high;
  if (!reader_take_u8(reader, &low) || !reader_take_u8(reader, &high)) {
    return false;
  }
  *out = (uint16_t)((uint16_t)high << 8 | low);
  return true;
}

static void set_default_macro(jpad_macro_t *macro, uint16_t keycode,
                              uint16_t repeat_interval) {
  memset(macro, 0, sizeof(*macro));
  macro->repeat_interval = repeat_interval;
  if (keycode != KC_NO) {
    macro->steps[0].key_count = 1;
    macro->steps[0].keys[0] = keycode;
    macro->step_count = 1;
  }
}

void jpad_config_load_defaults(void) {
  memset(macros, 0, sizeof(macros));

  for (uint8_t layer = 0; layer < JPAD_NUM_LAYERS; layer++) {
    set_default_macro(&macros[layer][0], KC_MPRV, 0);
    set_default_macro(&macros[layer][1], KC_MPLY, 0);
    set_default_macro(&macros[layer][2], KC_MNXT, 0);
    set_default_macro(&macros[layer][3], SWITCH_LAYER_0, 0);
    set_default_macro(&macros[layer][4], KC_F13, 0);
    set_default_macro(&macros[layer][5], KC_F14, 0);
    set_default_macro(&macros[layer][6], KC_F15, 0);
    set_default_macro(&macros[layer][7], KC_MUTE, 0);
    set_default_macro(&macros[layer][8], KC_F16, 0);
    set_default_macro(&macros[layer][9], KC_F17, 0);
    set_default_macro(&macros[layer][10], KC_F18, 0);
    set_default_macro(&macros[layer][11], MS_BTN1, 100);
    set_default_macro(&macros[layer][12], SWITCH_LAYER_1, 0);
    set_default_macro(&macros[layer][13], SWITCH_LAYER_2, 0);
    set_default_macro(&macros[layer][14], SWITCH_LAYER_3, 0);
    set_default_macro(&macros[layer][15], SWITCH_LAYER_4, 0);

    encoders[layer].counterclockwise = KC_VOLD;
    encoders[layer].clockwise = KC_VOLU;
  }

  encoders[1].counterclockwise = KC_LEFT;
  encoders[1].clockwise = KC_RIGHT;
  encoders[2].counterclockwise = C(KC_MINS);
  encoders[2].clockwise = C(KC_EQL);
  encoders[3].counterclockwise = MS_WHLD;
  encoders[3].clockwise = MS_WHLU;
  encoders[4].counterclockwise = KC_MPRV;
  encoders[4].clockwise = KC_MNXT;
}

uint16_t jpad_config_serialize(uint8_t *out, uint16_t max) {
  blob_writer_t writer = {
      .buffer = out,
      .length = 0,
      .capacity = max,
      .ok = true,
  };

  writer_put_u16(&writer, JPAD_BLOB_MAGIC);
  writer_put_u8(&writer, JPAD_BLOB_VERSION);
  writer_put_u8(&writer, 0);
  writer_put_u16(&writer, 0);
  writer_put_u8(&writer, JPAD_NUM_LAYERS);
  writer_put_u8(&writer, JPAD_NUM_MACROS);
  writer_put_u8(&writer, JPAD_MAX_STEPS);
  writer_put_u8(&writer, JPAD_MAX_STEP_KEYS);

  for (uint8_t layer = 0; layer < JPAD_NUM_LAYERS; layer++) {
    for (uint8_t index = 0; index < JPAD_NUM_MACROS; index++) {
      const jpad_macro_t *macro = &macros[layer][index];
      writer_put_u8(&writer, macro->step_count);
      writer_put_u16(&writer, (uint16_t)macro->repeat_interval);
      writer_put_u16(&writer, macro->delay_ms);

      for (uint8_t step_index = 0; step_index < macro->step_count;
           step_index++) {
        const jpad_step_t *step = &macro->steps[step_index];
        writer_put_u8(&writer, step->key_count);
        for (uint8_t key_index = 0; key_index < step->key_count; key_index++) {
          writer_put_u16(&writer, step->keys[key_index]);
        }
      }
    }
  }

  for (uint8_t layer = 0; layer < JPAD_NUM_LAYERS; layer++) {
    writer_put_u16(&writer, (uint16_t)encoders[layer].counterclockwise);
    writer_put_u16(&writer, (uint16_t)encoders[layer].clockwise);
  }

  if (!writer.ok) {
    return 0;
  }

  out[4] = (uint8_t)(writer.length & 0xFF);
  out[5] = (uint8_t)(writer.length >> 8);
  return writer.length;
}

bool jpad_config_deserialize(const uint8_t *data, uint16_t length) {
  if (data == NULL || length < JPAD_BLOB_HEADER_SIZE ||
      length > JPAD_BLOB_MAX) {
    return false;
  }

  blob_reader_t reader = {
      .buffer = data,
      .length = length,
      .offset = 0,
      .ok = true,
  };

  uint16_t magic, total_length;
  uint8_t version, flags, layer_count, macro_count, max_steps, max_step_keys;

  if (!reader_take_u16(&reader, &magic) || !reader_take_u8(&reader, &version) ||
      !reader_take_u8(&reader, &flags) ||
      !reader_take_u16(&reader, &total_length) ||
      !reader_take_u8(&reader, &layer_count) ||
      !reader_take_u8(&reader, &macro_count) ||
      !reader_take_u8(&reader, &max_steps) ||
      !reader_take_u8(&reader, &max_step_keys)) {
    return false;
  }

  if (magic != JPAD_BLOB_MAGIC || version != JPAD_BLOB_VERSION || flags != 0 ||
      total_length != length) {
    return false;
  }

  if (layer_count > JPAD_NUM_LAYERS || macro_count > JPAD_NUM_MACROS ||
      max_steps > JPAD_MAX_STEPS || max_step_keys > JPAD_MAX_STEP_KEYS) {
    return false;
  }

  for (int pass = 0; pass < 2; pass++) {
    const bool apply = pass == 1;

    reader.offset = JPAD_BLOB_HEADER_SIZE;
    reader.ok = true;

    for (uint8_t layer = 0; layer < layer_count; layer++) {
      for (uint8_t index = 0; index < macro_count; index++) {
        jpad_macro_t *macro = apply ? &macros[layer][index] : NULL;
        jpad_step_t *steps = apply ? macro->steps : NULL;

        if (apply) {
          memset(macro, 0, sizeof(*macro));
        }

        uint8_t step_count, key_count;
        uint16_t repeat_interval, delay_ms;
        if (!reader_take_u8(&reader, &step_count) ||
            !reader_take_u16(&reader, &repeat_interval) ||
            !reader_take_u16(&reader, &delay_ms)) {
          return false;
        }

        if (step_count > max_steps) {
          return false;
        }

        if (apply) {
          macro->step_count = step_count;
          macro->repeat_interval = repeat_interval;
          macro->delay_ms = delay_ms;
        }

        for (uint8_t step_index = 0; step_index < step_count; step_index++) {
          if (!reader_take_u8(&reader, &key_count) ||
              key_count > max_step_keys) {
            return false;
          }

          if (apply) {
            steps[step_index].key_count = key_count;
          }

          for (uint8_t key_index = 0; key_index < key_count; key_index++) {
            uint16_t keycode;
            if (!reader_take_u16(&reader, &keycode)) {
              return false;
            }
            if (apply) {
              steps[step_index].keys[key_index] = keycode;
            }
          }
        }
      }
    }

    for (uint8_t layer = 0; layer < layer_count; layer++) {
      uint16_t counterclockwise, clockwise;
      if (!reader_take_u16(&reader, &counterclockwise) ||
          !reader_take_u16(&reader, &clockwise)) {
        return false;
      }
      if (apply) {
        encoders[layer].counterclockwise = (int16_t)counterclockwise;
        encoders[layer].clockwise = (int16_t)clockwise;
      }
    }
  }

  if (!reader.ok || reader.offset != length) {
    return false;
  }

  layout_dirty = true;
  return true;
}

bool jpad_config_load(void) {
  if (wear_leveling_read(JPAD_EEPROM_BASE, persist_buffer,
                         sizeof(persist_buffer)) != WEAR_LEVELING_SUCCESS) {
    dprintf("jpad: wear leveling read failed, using defaults\n");
    return false;
  }

  uint16_t total_length =
      (uint16_t)persist_buffer[4] | (uint16_t)(persist_buffer[5] << 8);
  if (total_length < JPAD_BLOB_HEADER_SIZE ||
      total_length > sizeof(persist_buffer)) {
    dprintf("jpad: stored layout has invalid length %d, using defaults\n",
            total_length);
    return false;
  }

  if (!jpad_config_deserialize(persist_buffer, total_length)) {
    dprintf("jpad: stored layout is invalid, using defaults\n");
    return false;
  }

  dprintf("jpad: loaded stored layout (%d bytes)\n", total_length);
  layout_dirty = false;
  layout_source = JPAD_LAYOUT_SOURCE_FLASH;
  stored_length = total_length;
  return true;
}

void jpad_config_save(void) {
  uint16_t length =
      jpad_config_serialize(persist_buffer, sizeof(persist_buffer));
  if (length == 0) {
    dprintf("jpad: layout serialization failed, not saving\n");
    return;
  }

  if (wear_leveling_write(JPAD_EEPROM_BASE, persist_buffer, length) ==
      WEAR_LEVELING_FAILED) {
    dprintf("jpad: failed to save layout\n");
    return;
  }

  layout_dirty = false;
  layout_source = JPAD_LAYOUT_SOURCE_FLASH;
  stored_length = length;
  dprintf("jpad: saved layout (%d bytes)\n", length);
}

void jpad_config_init(void) {
  jpad_config_load_defaults();
  if (!jpad_config_load()) {

    layout_dirty = true;
  }
}

void jpad_config_mark_dirty(void) { layout_dirty = true; }

bool jpad_config_is_dirty(void) { return layout_dirty; }

jpad_layout_source_t jpad_config_source(void) { return layout_source; }

uint16_t jpad_config_stored_length(void) { return stored_length; }

jpad_macro_t *jpad_config_macro(uint8_t layer, uint8_t macro_index) {
  if (layer >= JPAD_NUM_LAYERS || macro_index >= JPAD_NUM_MACROS) {
    return NULL;
  }
  return &macros[layer][macro_index];
}

jpad_encoder_t *jpad_config_encoder(uint8_t layer) {
  if (layer >= JPAD_NUM_LAYERS) {
    return NULL;
  }
  return &encoders[layer];
}

void jpad_config_run_macro(const jpad_macro_t *macro) {
  if (macro == NULL) {
    return;
  }

  for (uint8_t step_index = 0; step_index < macro->step_count; step_index++) {
    const jpad_step_t *step = &macro->steps[step_index];

    for (uint8_t key_index = 0; key_index < step->key_count; key_index++) {
      uint16_t keycode = step->keys[key_index];
      if (is_layer_keycode(keycode)) {
        layer_move(keycode - SWITCH_LAYER_0);
      } else if (keycode != KC_NO) {
        register_code16(keycode);
      }
    }

    for (uint8_t key_index = 0; key_index < step->key_count; key_index++) {
      uint16_t keycode = step->keys[key_index];
      if (is_layer_keycode(keycode) || keycode == KC_NO) {
        continue;
      }
      unregister_code16(keycode);
    }
  }

  if (macro->delay_ms > 0) {
    wait_ms(macro->delay_ms);
  }
}
