// SPDX-License-Identifier: GPL-2.0-or-later
#include "jpad_protocol.h"

#include <string.h>

#include "debug.h"
#include "raw_hid.h"

_Static_assert(JPAD_FRAGMENT_PAYLOAD_SIZE == JPAD_RESPONSE_PAYLOAD_SIZE, "Fragments and responses must fit the same report");

static uint8_t blob[JPAD_BLOB_MAX];
static uint16_t blob_length = 0;

static bool fragment_active = false;
static uint8_t fragment_transaction = 0;
static uint8_t fragment_command = 0;
static uint16_t fragment_total = 0;
static uint16_t fragment_next = 0;

static bool save_pending = false;
static uint8_t save_transaction = 0;

static void send_response(uint8_t command, uint8_t transaction, uint16_t index, uint16_t total, const uint8_t *payload, uint8_t length)
{
    uint8_t report[RAW_EPSIZE];
    memset(report, 0, sizeof(report));

    if (length > JPAD_RESPONSE_PAYLOAD_SIZE)
    {
        length = JPAD_RESPONSE_PAYLOAD_SIZE;
    }

    report[0] = command;
    report[1] = transaction;
    report[2] = (uint8_t)(index & 0xFF);
    report[3] = (uint8_t)(index >> 8);
    report[4] = (uint8_t)(total & 0xFF);
    report[5] = (uint8_t)(total >> 8);
    report[6] = length;
    if (length > 0 && payload != NULL)
    {
        memcpy(&report[JPAD_RESPONSE_HEADER_SIZE], payload, length);
    }

    raw_hid_send(report, RAW_EPSIZE);
}

static void send_error(uint8_t error)
{
    uint8_t report[RAW_EPSIZE];
    memset(report, 0, sizeof(report));
    report[0] = JPAD_RSP_ERROR;
    report[1] = error;
    raw_hid_send(report, RAW_EPSIZE);
}

static void reset_fragment_transaction(void)
{
    fragment_active = false;
    fragment_transaction = 0;
    fragment_total = 0;
    fragment_next = 0;
    fragment_command = 0;
    blob_length = 0;
}

static bool apply_macro_payload(const uint8_t *data, uint16_t length)
{
    if (length < 5)
    {
        return false;
    }

    uint8_t layer = data[0];
    uint8_t index = data[1];
    uint8_t step_count = data[2];
    uint16_t delay_ms = (uint16_t)((uint16_t)data[3] | ((uint16_t)data[4] << 8));

    jpad_macro_t *macro = jpad_config_macro(layer, index);
    if (macro == NULL || step_count > JPAD_MAX_STEPS)
    {
        return false;
    }

    uint16_t repeat_interval = macro->repeat_interval;
    memset(macro, 0, sizeof(*macro));
    macro->step_count = step_count;
    macro->delay_ms = delay_ms;
    macro->repeat_interval = repeat_interval;

    uint16_t offset = 5;
    for (uint8_t step_index = 0; step_index < step_count; step_index++)
    {
        if (offset + 1 > length)
        {
            return false;
        }

        uint8_t key_count = data[offset++];
        if (key_count > JPAD_MAX_STEP_KEYS)
        {
            return false;
        }

        macro->steps[step_index].key_count = key_count;
        for (uint8_t key_index = 0; key_index < key_count; key_index++)
        {
            if (offset + 2 > length)
            {
                return false;
            }
            macro->steps[step_index].keys[key_index] = (uint16_t)((uint16_t)data[offset] | ((uint16_t)data[offset + 1] << 8));
            offset += 2;
        }
    }

    if (offset != length)
    {
        return false;
    }

    dprintf("jpad: macro update layer %d slot %d steps %d delay %d\n", layer, index, step_count, delay_ms);
    return true;
}

static void handle_set_encoder(const uint8_t *data, uint8_t length)
{
    jpad_encoder_t *encoder = length >= 6 ? jpad_config_encoder(data[1]) : NULL;
    if (encoder == NULL)
    {
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    encoder->clockwise = (int16_t)((uint16_t)data[2] | ((uint16_t)data[3] << 8));
    encoder->counterclockwise = (int16_t)((uint16_t)data[4] | ((uint16_t)data[5] << 8));
    dprintf("jpad: encoder update layer %d\n", data[1]);
}

static void handle_set_repeat(const uint8_t *data, uint8_t length)
{
    jpad_macro_t *macro = length >= 5 ? jpad_config_macro(data[1], data[2]) : NULL;
    if (macro == NULL)
    {
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    macro->repeat_interval = (uint16_t)data[3] | ((uint16_t)data[4] << 8);
    dprintf("jpad: repeat update layer %d slot %d interval %d\n", data[1], data[2], macro->repeat_interval);
}

static void handle_fragment(const uint8_t *data, uint8_t length)
{
    uint8_t command = data[0];

    if (length < JPAD_FRAGMENT_HEADER_SIZE + 1)
    {
        reset_fragment_transaction();
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    uint8_t transaction = data[1];
    uint16_t total = (uint16_t)data[2] | ((uint16_t)data[3] << 8);
    uint16_t index = (uint16_t)data[4] | ((uint16_t)data[5] << 8);
    uint8_t payload_length = data[6];

    if (total == 0 || index >= total || payload_length > length - JPAD_FRAGMENT_HEADER_SIZE)
    {
        reset_fragment_transaction();
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    if (index == 0)
    {
        reset_fragment_transaction();
        fragment_active = true;
        fragment_transaction = transaction;
        fragment_total = total;
        fragment_next = 0;
        fragment_command = command;
    }

    if (!fragment_active || transaction != fragment_transaction || total != fragment_total || command != fragment_command)
    {
        dprintf("jpad: dropping out of order fragment\n");
        reset_fragment_transaction();
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    if (index != fragment_next)
    {
        dprintf("jpad: dropping fragment %d, expected %d\n", index, fragment_next);
        reset_fragment_transaction();
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    if (blob_length + payload_length > sizeof(blob))
    {
        dprintf("jpad: layout payload too large\n");
        reset_fragment_transaction();
        send_error(JPAD_ERROR_LAYOUT_REJECTED);
        return;
    }

    memcpy(&blob[blob_length], &data[JPAD_FRAGMENT_HEADER_SIZE], payload_length);
    blob_length += payload_length;
    fragment_next++;

    if (fragment_next < fragment_total)
    {
        return;
    }

    bool accepted = command == JPAD_CMD_SET_LAYOUT ? jpad_config_deserialize(blob, blob_length) : apply_macro_payload(blob, blob_length);
    if (accepted)
    {
        jpad_config_mark_dirty();
    }
    else
    {
        dprintf("jpad: rejected payload of %d bytes\n", blob_length);
        send_error(JPAD_ERROR_LAYOUT_REJECTED);
    }

    reset_fragment_transaction();
}

static void handle_get_layout(const uint8_t *data, uint8_t length)
{
    if (length < 4)
    {
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    uint8_t transaction = data[1];
    uint16_t index = (uint16_t)data[2] | ((uint16_t)data[3] << 8);

    uint16_t total_length = jpad_config_serialize(blob, sizeof(blob));
    if (total_length == 0)
    {
        send_error(JPAD_ERROR_LAYOUT_REJECTED);
        return;
    }

    uint16_t total_fragments = (uint16_t)((total_length + JPAD_FRAGMENT_PAYLOAD_SIZE - 1) / JPAD_FRAGMENT_PAYLOAD_SIZE);
    if (index >= total_fragments)
    {
        dprintf("jpad: requested layout fragment %d of %d\n", index, total_fragments);
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    uint16_t offset = (uint16_t)(index * JPAD_FRAGMENT_PAYLOAD_SIZE);
    uint16_t payload_length = (uint16_t)(total_length - offset);
    if (payload_length > JPAD_FRAGMENT_PAYLOAD_SIZE)
    {
        payload_length = JPAD_FRAGMENT_PAYLOAD_SIZE;
    }

    send_response(JPAD_RSP_GET_LAYOUT, transaction, index, total_fragments, &blob[offset], (uint8_t)payload_length);
}

static void handle_ping(const uint8_t *data, uint8_t length)
{
    if (length < 2)
    {
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    uint16_t stored = jpad_config_stored_length();
    uint8_t payload[6] = {
        JPAD_NUM_LAYERS,
        JPAD_NUM_MACROS,
        (uint8_t)(jpad_config_is_dirty() ? 1 : 0),
        (uint8_t)jpad_config_source(),
        (uint8_t)(stored & 0xFF),
        (uint8_t)(stored >> 8),
    };
    send_response(JPAD_RSP_PING, data[1], 0, 0, payload, sizeof(payload));
}

static void handle_save_layout(const uint8_t *data, uint8_t length)
{
    if (length < 2)
    {
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    if (save_pending)
    {
        dprintf("jpad: save already in progress\n");
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    save_pending = true;
    save_transaction = data[1];
}

static void handle_reset_layout(const uint8_t *data, uint8_t length)
{
    if (length < 2)
    {
        send_error(JPAD_ERROR_MALFORMED_PACKET);
        return;
    }

    jpad_config_load_defaults();
    jpad_config_mark_dirty();
    send_response(JPAD_RSP_RESET_LAYOUT, data[1], 0, 0, NULL, 0);
}

void jpad_protocol_receive(uint8_t *data, uint8_t length)
{
    if (data == NULL || length < 2)
    {
        return;
    }

    switch (data[0])
    {
        case JPAD_CMD_SET_ENCODER:
            handle_set_encoder(data, length);
            break;
        case JPAD_CMD_SET_MACRO:
        case JPAD_CMD_SET_LAYOUT:
            handle_fragment(data, length);
            break;
        case JPAD_CMD_SET_REPEAT:
            handle_set_repeat(data, length);
            break;
        case JPAD_CMD_GET_LAYOUT:
            handle_get_layout(data, length);
            break;
        case JPAD_CMD_SAVE_LAYOUT:
            handle_save_layout(data, length);
            break;
        case JPAD_CMD_RESET_LAYOUT:
            handle_reset_layout(data, length);
            break;
        case JPAD_CMD_PING:
            handle_ping(data, length);
            break;
        default:
            dprintf("jpad: unknown command %d\n", data[0]);
            send_error(JPAD_ERROR_UNSUPPORTED_COMMAND);
            break;
    }
}

void jpad_protocol_task(void)
{
    if (!save_pending)
    {
        return;
    }

    save_pending = false;
    jpad_config_save();
    send_response(JPAD_RSP_SAVE_LAYOUT, save_transaction, 0, 0, NULL, 0);
}

void jpad_protocol_init(void)
{
    reset_fragment_transaction();
    save_pending = false;
    save_transaction = 0;
}
