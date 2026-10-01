// SPDX-License-Identifier: GPL-2.0-or-later
#pragma once

#include "jpad_config.h"
#include "usb_descriptor.h"

#define JPAD_CMD_SET_ENCODER 0
#define JPAD_CMD_SET_MACRO 1
#define JPAD_CMD_SET_REPEAT 2
#define JPAD_CMD_GET_LAYOUT 3
#define JPAD_CMD_SET_LAYOUT 4
#define JPAD_CMD_SAVE_LAYOUT 5
#define JPAD_CMD_RESET_LAYOUT 6
#define JPAD_CMD_PING 7

#define JPAD_RSP_GET_LAYOUT 0x83
#define JPAD_RSP_SAVE_LAYOUT 0x85
#define JPAD_RSP_RESET_LAYOUT 0x86
#define JPAD_RSP_PING 0x87
#define JPAD_RSP_ERROR 0xFF

#define JPAD_ERROR_UNSUPPORTED_COMMAND 1
#define JPAD_ERROR_MALFORMED_PACKET 2
#define JPAD_ERROR_LAYOUT_REJECTED 3

#define JPAD_FRAGMENT_HEADER_SIZE 7
#define JPAD_FRAGMENT_PAYLOAD_SIZE (RAW_EPSIZE - JPAD_FRAGMENT_HEADER_SIZE)

#define JPAD_RESPONSE_HEADER_SIZE 7
#define JPAD_RESPONSE_PAYLOAD_SIZE (RAW_EPSIZE - JPAD_RESPONSE_HEADER_SIZE)

void jpad_protocol_init(void);
void jpad_protocol_receive(uint8_t *data, uint8_t length);
void jpad_protocol_task(void);
