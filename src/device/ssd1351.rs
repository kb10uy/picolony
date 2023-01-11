use embedded_hal::{blocking::spi::Write as SpiWrite, digital::v2::OutputPin};
use ssd1351::{
    display::Display,
    interface::SpiInterface,
    mode::{displaymode::DisplayModeTrait, GraphicsMode},
    properties::{DisplayRotation, DisplaySize},
};

/// Initializes SSD1351 display as graphics mode.
pub fn create_ssd1351_graphics<S: SpiWrite<u8>, D: OutputPin>(
    spi: S,
    dc: D,
) -> GraphicsMode<SpiInterface<S, D>> {
    let interface = SpiInterface::new(spi, dc);
    let mut display = Display::new(
        interface,
        DisplaySize::Display128x96,
        DisplayRotation::Rotate0,
    );
    display
        .set_rotation(DisplayRotation::Rotate0)
        .expect("Failed to set rotation");
    DisplayModeTrait::new(display)
}
