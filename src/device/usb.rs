use bsp::{
    hal::{clocks::UsbClock, usb::UsbBus},
    pac::{RESETS, USBCTRL_DPRAM, USBCTRL_REGS},
};
use rp_pico as bsp;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

/// Initializes USB bus.
pub fn initialize_bus(
    ctrl_reg: USBCTRL_REGS,
    ctrl_dpram: USBCTRL_DPRAM,
    clock: UsbClock,
    resets: &mut RESETS,
) -> UsbBusAllocator<UsbBus> {
    let raw_bus = UsbBus::new(ctrl_reg, ctrl_dpram, clock, true, resets);
    let usb_bus = UsbBusAllocator::new(raw_bus);
    usb_bus
}

/// Initializes USB device and sets it up for serial port.
pub fn initialize_device(
    bus: &UsbBusAllocator<UsbBus>,
    (vid, pid): (u16, u16),
) -> (UsbDevice<UsbBus>, SerialPort<UsbBus>) {
    // SerialPort instance must be created first, or descriptor requests will fail.
    let serial_port = SerialPort::new(bus);
    let usb_device = UsbDeviceBuilder::new(bus, UsbVidPid(vid, pid))
        .manufacturer("Kusaka Factory")
        .product("USB Serial Port")
        .device_class(USB_CLASS_CDC)
        .build();

    (usb_device, serial_port)
}
