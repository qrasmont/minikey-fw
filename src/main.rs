#![no_std]
#![no_main]

mod keycode;
mod keypad;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;
use rp2040_hal::{
    clocks::init_clocks_and_plls, entry, gpio::Pins, pac, usb::UsbBus, Clock, Sio, Watchdog,
};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::{descriptor::KeyboardReport, hid_class::HIDClass};

use crate::keycode::KeyCode;

// Place this boot block at the start of the program image
// Needed for the ROM bootloader get our code up and running
#[link_section = ".boot2"]
#[used]
static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const CRYSTAL_FREQUENCY_HZ: u32 = 12_000_000u32;

const USB_POLLING_RATE_MS: u8 = 10;

const USB_KBD_VID: u16 = 0x16c0;
const USB_KBD_PID: u16 = 0x27db;

fn send_press(hid: &HIDClass<UsbBus>, key: KeyCode, delay: &mut cortex_m::delay::Delay) {
    let mut report = KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0; 6],
    };

    report.keycodes[0] = key as u8;
    hid.push_input(&report).unwrap();
    delay.delay_ms(USB_POLLING_RATE_MS.into());

    report.keycodes[0] = 0;
    hid.push_input(&report).unwrap();
    delay.delay_ms(USB_POLLING_RATE_MS.into());
}

#[entry]
fn main() -> ! {
    info!("Start");

    // Acquire our RP2040 peripherals
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // Init our clocks
    let clocks = init_clocks_and_plls(
        CRYSTAL_FREQUENCY_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Bring up the RP2040 USB bus
    let usb = UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );

    // Helper to manage resource allocation and initialization of the USB bus
    let usb_allocator = UsbBusAllocator::new(usb);

    // This interface allows:
    // - to declare the type of report we need (Keyboard)
    // - to read and write those reports
    let mut usb_hid = HIDClass::new(&usb_allocator, KeyboardReport::desc(), USB_POLLING_RATE_MS);

    // Build a USB device
    let mut usb_device = UsbDeviceBuilder::new(&usb_allocator, UsbVidPid(USB_KBD_VID, USB_KBD_PID))
        .manufacturer("Quentin")
        .product("Minikey")
        .serial_number("0")
        .device_class(0)
        .build();

    let mut led = pins.gpio25.into_push_pull_output();

    let key = pins.gpio0.into_pull_up_input();
    let mut state = key.is_low().unwrap();

    loop {
        usb_device.poll(&mut [&mut usb_hid]);

        let previous_state = state;
        state = key.is_low().unwrap();

        match (previous_state, state) {
            (false, true) => {
                led.set_high().unwrap();
                send_press(&usb_hid, KeyCode::A, &mut delay);
            }
            (true, false) => {
                led.set_low().unwrap();
            }
            _ => {}
        }
    }
}
