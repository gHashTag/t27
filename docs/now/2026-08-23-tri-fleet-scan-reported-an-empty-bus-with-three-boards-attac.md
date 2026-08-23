# NOW -- tri fleet scan reported an empty bus with three boards attached (2026-08-23)

## tri fleet scan reported an empty bus with three boards attached (Closes #2521)

- Two independent bugs, both failing towards 'no hardware'. It matched device names against ft232/ftdi/usb serial/jtag, and macOS calls these Digilent USB Device — a name is a label, idVendor 1027 is the identity. And ioreg draws its tree with pipes, so trim() plus starts_with('+-o') matched only TOP-LEVEL nodes; an FT232H on a hub port is always nested, so no board could match whatever it was called.
- Also counted one board three times, because a device's interfaces repeat its vendor and serial. Now asks ioreg -r -c IOUSBHostDevice and accepts only device-class nodes. Verified on the live bench: 3 bridges, --expect 3 exits 0 and --expect 4 exits 1.
