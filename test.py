import sys
import hid
from time import sleep

vendor_id     = 0xfeed
product_id    = 0x0000

usage_page    = 0xFF60
usage         = 0x61
report_length = 32
fragment_payload_size = 27

COMMAND = 1

def get_raw_hid_interface():
    device_interfaces = hid.enumerate(vendor_id, product_id)
    raw_hid_interfaces = [i for i in device_interfaces if i['usage_page'] == usage_page and i['usage'] == usage]

    if len(raw_hid_interfaces) == 0:
        return None

    interface = hid.Device(path=raw_hid_interfaces[0]['path'])

    print(f"Manufacturer: {interface.manufacturer}")
    print(f"Product: {interface.product}")

    return interface

def send_raw_report(data):
    interface = get_raw_hid_interface()

    if interface is None:
        print("No device found")
        sys.exit(1)

    request_data = [0x00] * (report_length + 1)
    request_data[1:len(data) + 1] = data
    request_report = bytes(request_data)

    print("Request:")
    print(request_report)

    try:
        interface.write(request_report)
    finally:
        interface.close()

def build_macro_update(layer, macro_index, macro_steps):
    payload = [layer, macro_index, len(macro_steps)]

    for step in macro_steps:
        payload.append(len(step))
        payload.extend(step)

    return payload

def send_fragmented_report(transaction_id, payload):
    chunks = [payload[i:i + fragment_payload_size] for i in range(0, len(payload), fragment_payload_size)]
    total_fragments = len(chunks)

    for fragment_index, chunk in enumerate(chunks):
        send_raw_report([COMMAND, transaction_id, total_fragments, fragment_index, len(chunk)] + chunk)

def send_fragmented_macro_update(layer, macro_index, macro_steps):
    payload = build_macro_update(layer, macro_index, macro_steps)
    transaction_id = 1
    send_fragmented_report(transaction_id, payload)

if __name__ == '__main__':
    send_fragmented_macro_update(
        layer=0,
        macro_index=4,
        macro_steps=[
            [0x04, 0x05],
            [0x06, 0x07],
            [0x08, 0x09],
            [0x0A, 0x0B],
            [0x0C, 0x0D],
            [0x0E, 0x0F],
            [0x10, 0x11],
            [0x12, 0x13],
            [0x14, 0x15],
            [0x04, 0x05],
            [0x06, 0x07],
            [0x08, 0x09],
            [0x0A, 0x0B],
            [0x0C, 0x0D],
            [0x0E, 0x0F],
            [0x10, 0x11],
            [0x12, 0x13],
            [0x14, 0x15],
            [0x04, 0x05],
            [0x06, 0x07],
            [0x08, 0x09],
            [0x0A, 0x0B],
            [0x0C, 0x0D],
            [0x0E, 0x0F],
            [0x10, 0x11],
            [0x12, 0x13],
            [0x14, 0x15],
            [0x04, 0x05],
            [0x06, 0x07],
            [0x08, 0x09],
            [0x0A, 0x0B],
            [0x0C, 0x0D],
            [0x04, 0x05],
            [0x06, 0x07],
            [0x08, 0x09],
            [0x0A, 0x0B],
            [0x0C, 0x0D],
            [0x0E, 0x0F],
            [0x10, 0x11],
            [0x12, 0x13],
            [0x14, 0x15],
            [0x04, 0x05],
            [0x06, 0x07],
            [0x08, 0x09],
            [0x0A, 0x0B],
            [0x0C, 0x0D],
            [0x0E, 0x0F],
            [0x10, 0x11],
            [0x12, 0x13],
            [0x14, 0x15],
            [0x08, 0x09],
            [0x04, 0x04,0x04, 0x04,0x04, 0x04,0x04, 0x04],
        ],
    )
