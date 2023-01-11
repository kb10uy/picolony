use crate::graphics::font::{JisFont, JisFontInterface};

use core::{
    cell::RefCell,
    fmt::{Error as FmtError, Result as FmtResult, Write},
    num::NonZeroUsize,
};

use embedded_graphics_core::prelude::*;

/// Pair of font and color.
pub struct JisTextStyle<'a, 'f, I, C, const CACHE_SIZE: usize>
where
    'f: 'a,
    I: JisFontInterface,
{
    font: &'a RefCell<JisFont<'f, I, CACHE_SIZE>>,
    fore_color: C,
    back_color: Option<C>,
}

impl<'a, 'f, I, C, const CACHE_SIZE: usize> JisTextStyle<'a, 'f, I, C, CACHE_SIZE>
where
    'f: 'a,
    I: JisFontInterface,
{
    /// Creates new style.
    pub fn new(font: &'a RefCell<JisFont<'f, I, CACHE_SIZE>>, fore_color: C) -> Self {
        JisTextStyle {
            font,
            fore_color,
            back_color: None,
        }
    }

    /// Sets background color.
    pub fn with_background(mut self, back_color: C) -> Self {
        self.back_color = Some(back_color);
        self
    }
}

/// Text to draw with `JisTextStyle`.
pub struct JisText<'a, 'f, I, C, const CACHE_SIZE: usize>
where
    'f: 'a,
    I: JisFontInterface,
{
    text: &'a str,
    style: &'a JisTextStyle<'a, 'f, I, C, CACHE_SIZE>,
    offset: Point,
    wrapping_width: Option<NonZeroUsize>,
}

impl<'a, 'f, I, C, const CACHE_SIZE: usize> JisText<'a, 'f, I, C, CACHE_SIZE>
where
    'f: 'a,
    I: JisFontInterface,
{
    /// Constructs new text to draw.
    pub fn new(
        text: &'a str,
        offset: Point,
        style: &'a JisTextStyle<'a, 'f, I, C, CACHE_SIZE>,
    ) -> Self {
        JisText {
            text,
            offset,
            style,
            wrapping_width: None,
        }
    }

    /// Sets wrapping width.
    pub fn with_wrapping(mut self, width: usize) -> Self {
        self.wrapping_width = NonZeroUsize::new(width);
        self
    }
}

impl<'a, 'f, I, C, const CACHE_SIZE: usize> Drawable for JisText<'a, 'f, I, C, CACHE_SIZE>
where
    'f: 'a,
    I: JisFontInterface,
    C: PixelColor,
{
    type Color = C;
    type Output = (usize, usize);

    fn draw<D>(&self, target: &mut D) -> Result<(usize, usize), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let mut font_cache = self.style.font.borrow_mut();
        let mut max_relx = 0;
        let (mut relx, mut rely) = (0, 0);
        let mut chars_in_line = 0;
        for line in self.text.lines() {
            for draw_char in line.chars() {
                let glyph_source = match font_cache.query(draw_char) {
                    Some(g) => g,
                    None => continue,
                };
                let char_offset = Point::new(self.offset.x + relx, self.offset.y + rely);
                I::draw(
                    target,
                    char_offset,
                    self.style.fore_color,
                    self.style.back_color,
                    glyph_source,
                )?;
                relx += I::WIDTH as i32;

                // Line wrapping.
                if let Some(wrap) = self.wrapping_width {
                    chars_in_line += 1;
                    if chars_in_line >= wrap.get() {
                        max_relx = max_relx.max(relx);
                        relx = 0;
                        rely += I::HEIGHT as i32;
                        chars_in_line = 0;
                    }
                }
            }
            max_relx = max_relx.max(relx);
            relx = 0;
            rely += I::HEIGHT as i32;
            chars_in_line = 0;
        }

        Ok((max_relx as usize, rely as usize))
    }
}

/// Text to draw with `JisTextStyle`.
pub struct JisTextDirect<'a, 'f, I, C, D, const CACHE_SIZE: usize>
where
    'f: 'a,
    I: JisFontInterface,
{
    draw_target: &'a mut D,
    style: &'a JisTextStyle<'a, 'f, I, C, CACHE_SIZE>,
    offset: Point,
    wrapping_width: Option<NonZeroUsize>,
    chars_in_line: usize,
    relx: i32,
    rely: i32,
}

impl<'a, 'f, I, C, D, const CACHE_SIZE: usize> JisTextDirect<'a, 'f, I, C, D, CACHE_SIZE>
where
    'f: 'a,
    I: JisFontInterface,
    D: DrawTarget<Color = C>,
{
    /// Constructs new text to draw.
    pub fn new(
        draw_target: &'a mut D,
        offset: Point,
        style: &'a JisTextStyle<'a, 'f, I, C, CACHE_SIZE>,
    ) -> Self {
        JisTextDirect {
            draw_target,
            offset,
            style,
            wrapping_width: None,
            chars_in_line: 0,
            relx: 0,
            rely: 0,
        }
    }

    /// Sets wrapping width.
    pub fn with_wrapping(mut self, width: usize) -> Self {
        self.wrapping_width = NonZeroUsize::new(width);
        self
    }
}

impl<'a, 'f, I, C, D, const CACHE_SIZE: usize> Write for JisTextDirect<'a, 'f, I, C, D, CACHE_SIZE>
where
    'f: 'a,
    I: JisFontInterface,
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    fn write_str(&mut self, s: &str) -> FmtResult {
        let mut font_cache = self.style.font.borrow_mut();
        for line in s.lines() {
            for draw_char in line.chars() {
                let glyph_source = match font_cache.query(draw_char) {
                    Some(g) => g,
                    None => continue,
                };
                let char_offset = Point::new(self.offset.x + self.relx, self.offset.y + self.rely);
                I::draw(
                    self.draw_target,
                    char_offset,
                    self.style.fore_color,
                    self.style.back_color,
                    glyph_source,
                )
                .map_err(|_| FmtError)?;
                self.relx += I::WIDTH as i32;

                // Line wrapping.
                if let Some(wrap) = self.wrapping_width {
                    self.chars_in_line += 1;
                    if self.chars_in_line >= wrap.get() {
                        self.relx = 0;
                        self.rely += I::HEIGHT as i32;
                        self.chars_in_line = 0;
                    }
                }
            }
            self.relx = 0;
            self.rely += I::HEIGHT as i32;
            self.chars_in_line = 0;
        }
        Ok(())
    }
}
