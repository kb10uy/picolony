use fugit::RateExtU32;
use rp_pico as bsp;

use bsp::{
    hal::{
        clocks::ClocksManager,
        uart::{
            Enabled, Error as UartError, ReadErrorType, UartConfig, UartDevice,
            UartPeripheral as RawUart, ValidUartPinout,
        },
        Clock,
    },
    pac::RESETS,
};
use cortex_m::prelude::*;
use nb::Error as NbError;

/// Alias the type for our UART to make things clearer.
pub type UartPeripheral<D, P> = RawUart<Enabled, D, P>;

/// Initializes UART0.
pub fn initialize<D: UartDevice, P: ValidUartPinout<D>>(
    target_device: D,
    pins: P,
    resets: &mut RESETS,
    clocks: &ClocksManager,
    baud: u32,
) -> Result<UartPeripheral<D, P>, UartError> {
    let mut config = UartConfig::default();
    config.baudrate = baud.Hz();

    let uart =
        RawUart::new(target_device, pins, resets).enable(config, clocks.peripheral_clock.freq())?;

    Ok(uart)
}

/// Extension trait for convenience.
pub trait UartPeripheralIoExt {
    /// Blocks and reads a line from this UART device.
    fn read_line(&mut self, buffer: &mut [u8], enable_echo: bool) -> Result<usize, ReadErrorType>;
}

impl<D: UartDevice, P: ValidUartPinout<D>> UartPeripheralIoExt for UartPeripheral<D, P> {
    fn read_line(&mut self, buffer: &mut [u8], enable_echo: bool) -> Result<usize, ReadErrorType> {
        let mut read_bytes = 0;
        let mut fulfilled = false;

        loop {
            match self.read() {
                Ok(b) => {
                    if enable_echo {
                        self.write(b).unwrap();
                    }
                    if b == b'\n' {
                        break;
                    }

                    if !fulfilled {
                        buffer[read_bytes] = b;
                        read_bytes += 1;

                        if read_bytes >= buffer.len() {
                            fulfilled = true;
                        }
                    }
                }
                Err(NbError::WouldBlock) => continue,
                Err(NbError::Other(err)) => return Err(err),
            }
        }

        Ok(read_bytes)
    }
}
