use crate::{
    cache::SimpleCacheMap,
    string::{Uni2JisTableError, Unicode2JisTable, JIS_KUTEN_WIDTH},
};

use core::cell::RefCell;

use embedded_graphics_core::prelude::*;

/// Conversion table binary of Unicode codepoint to JIS kuten code.
const UNI2JIS_DATA: &[u8] = include_bytes!("../../assets/uni2jis.bin");

pub trait JisFontInterface {
    /// Cached type of glyph.
    type Cached: Default + Copy;

    /// Glyph width.
    const WIDTH: usize;

    /// Glyph height.
    const HEIGHT: usize;

    /// Validates input bitmap.
    /// Returns whether the bitmap is valid for this interface.
    fn validate_bitmap(bitmap: &[u8]) -> bool;

    /// Fetches glyph from bitmap into cached form.
    fn fetch(bitmap: &[u8], kuten: (u8, u8)) -> Self::Cached;

    /// Draw a character.
    fn draw<C: PixelColor, D: DrawTarget<Color = C>>(
        target: &mut D,
        offset: Point,
        fore_color: C,
        back_color: Option<C>,
        glyph: &Self::Cached,
    ) -> Result<(), D::Error>;
}

/// Represents a drawable font data based on JIS encoding.
pub struct JisFont<'a, I, const CACHE_SIZE: usize>
where
    I: JisFontInterface,
{
    uni2jis_table: Unicode2JisTable<'a>,
    font_cache: SimpleCacheMap<u16, I::Cached, CACHE_SIZE>,
    font_bitmap: &'a [u8],
}

impl<'a, I, const CACHE_SIZE: usize> JisFont<'a, I, CACHE_SIZE>
where
    I: JisFontInterface,
{
    /// Creates new font with font bitmap data.
    pub fn new(font_bitmap: &'a [u8]) -> Result<RefCell<Self>, JisFontError> {
        if !I::validate_bitmap(font_bitmap) {
            return Err(JisFontError::InvalidFontBitmap);
        }

        let uni2jis_table =
            Unicode2JisTable::new(UNI2JIS_DATA).map_err(|e| JisFontError::InvalidUni2Jis(e))?;

        Ok(RefCell::new(JisFont {
            uni2jis_table,
            font_bitmap,
            font_cache: SimpleCacheMap::new(),
        }))
    }

    /// Queries font cache.
    pub(crate) fn query(&mut self, draw_char: char) -> Option<&I::Cached> {
        self.font_cache.get_or_else(draw_char as u16, |_| {
            let kuten = self.uni2jis_table.query(draw_char)?;
            Some(I::fetch(self.font_bitmap, kuten))
        })
    }
}

/// `JisFont` errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JisFontError {
    InvalidUni2Jis(Uni2JisTableError),
    InvalidFontBitmap,
}

/// 8x12 JIS font interface.
pub enum JisFont8x12 {}

impl JisFontInterface for JisFont8x12 {
    type Cached = [u8; 12];
    const WIDTH: usize = 8;
    const HEIGHT: usize = 12;

    fn validate_bitmap(bitmap: &[u8]) -> bool {
        let expected = JIS_KUTEN_WIDTH * JIS_KUTEN_WIDTH * 12;
        bitmap.len() == expected
    }

    fn fetch(bitmap: &[u8], (ku, ten): (u8, u8)) -> Self::Cached {
        let mut b = [0; 12];
        let base_index = (ku as usize - 1) * JIS_KUTEN_WIDTH + (ten as usize - 1);
        b.copy_from_slice(&bitmap[(base_index * 12)..((base_index + 1) * 12)]);
        b
    }

    fn draw<C: PixelColor, D: DrawTarget<Color = C>>(
        target: &mut D,
        offset: Point,
        fore_color: C,
        back_color: Option<C>,
        glyph: &Self::Cached,
    ) -> Result<(), D::Error> {
        let pixels = glyph
            .into_iter()
            .enumerate()
            .map(|(byte, &x)| {
                let glyph_y = byte as i32;

                // Per-byte, part of column
                (0..8).map(move |glyph_x| {
                    let point = Point::new(offset.x + glyph_x, offset.y + glyph_y);
                    let shifted_bit = 1 << (7 - glyph_x);

                    if x & shifted_bit != 0 {
                        Some(Pixel(point, fore_color))
                    } else {
                        back_color.map(|c| Pixel(point, c))
                    }
                })
            })
            .flatten()
            .flatten();

        target.draw_iter(pixels)?;
        Ok(())
    }
}
