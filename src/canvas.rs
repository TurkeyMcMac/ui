use std::fmt::{self, Display, Formatter};

pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,
}

impl Canvas {
     pub fn new(width: usize, height: usize, filler: char) -> Canvas {
        Canvas {
            width, height,
            pixels: vec![Pixel { ch: filler, flags: 0, }; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Pixel> {
        if x < self.width && y < self.height {
            Some(unsafe {
                self.pixels.get_unchecked(y * self.width + x)
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Pixel> {
        if x < self.width && y < self.height {
            Some(unsafe {
                self.pixels.get_unchecked_mut(y * self.width + x)
            })
        } else {
            None
        }
    }
    
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> &Pixel {
        self.pixels.get_unchecked(y * self.width + x)
    }
    
    pub unsafe fn get_unchecked_mut(&mut self, x: usize, y: usize) -> &mut Pixel {
        self.pixels.get_unchecked_mut(y * self.width + x)
    }

    pub fn text(&mut self, text: &str, x: usize, y: usize, styles: TextStyles) {
        let mut current_x = x; let mut current_y = y;
        if x >= self.width || y >= self.height {
            return;
        }
        let mut in_bounds = true;
        let mut last_x = x; let mut last_y = y;

        unsafe {
            self.get_unchecked_mut(current_x, current_y).set_styles_on(styles);
        }

        for letter in text.chars() {
            match letter {
                '\n' => {
                    unsafe {
                        self.get_unchecked_mut(last_x, last_y).set_styles_off(styles);
                    }
                    current_x = x;
                    current_y += 1;
                    in_bounds = current_x < self.width && current_y < self.height;
                    if in_bounds {
                        unsafe {
                            self.get_unchecked_mut(current_x, current_y).set_styles_on(styles);
                        }
                        last_x = current_x;
                        last_y = current_y;
                    }
                },
                letter => {
                    if in_bounds {
                        unsafe {
                            self.get_unchecked_mut(current_x, current_y).ch = letter;
                        }
                        last_x = current_x;
                        last_y = current_y;
                        current_x += 1;
                        in_bounds = self.width > current_x;
                    }
                }
            }
        }

        unsafe {
            self.get_unchecked_mut(last_x, last_y).set_styles_off(styles);
        }
    }

    pub fn line(&mut self, fill: char, x: usize, y: usize, len: usize, styles: TextStyles) {
        if x >= self.width || y >= self.height || len == 0 {
            return;
        }
        let len = if x + len >= self.width {
            self.width - x
        } else {
            len
        };

        for p in &mut self.pixels[y * self.width + x .. y * self.width + x + len] {
            p.ch = fill
        }

        unsafe {
            self.get_unchecked_mut(x, y).set_styles_on(styles);
            self.get_unchecked_mut(x + len - 1, y).set_styles_off(styles)
        }
    }
}

impl Display for Canvas {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for ps in self.pixels.chunks(self.width) {
            for p in ps {
                write!(f, "{}", p)?;
            }
            write!(f, "\x1B[0m\n")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextStyles {
    inner: u8,
}

const BOLD_POS: u8 = 0;
const ITALICS_POS: u8 = 1;
const UNDERLINE_POS: u8 = 2;
const INVERSE_POS: u8 = 3;

const BOLD_ON: u8 = 1 << BOLD_POS;
const ITALICS_ON: u8 = 1 << ITALICS_POS;
const UNDERLINE_ON: u8 = 1 << UNDERLINE_POS;
const INVERSE_ON: u8 = 1 << INVERSE_POS;
const BOLD_OFF: u8 = 1 << (BOLD_POS + 4);
const ITALICS_OFF: u8 = 1 << (ITALICS_POS + 4);
const UNDERLINE_OFF: u8 = 1 << (UNDERLINE_POS + 4);
const INVERSE_OFF: u8 = 1 << (INVERSE_POS + 4);

#[derive(Clone, Copy)]
pub struct Pixel {
    pub ch: char,
    pub flags: u8,
}

impl Pixel {
    pub fn set_styles_on(&mut self, styles: TextStyles) {
        self.flags &= !0 << 4;
        self.flags |= styles.inner;
    }

    pub fn set_styles_off(&mut self, styles: TextStyles) {
        self.flags &= !0 >> 4;
        self.flags |= styles.inner << 4;
    }
}

impl Display for Pixel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.flags == 0 {
            write!(f, "{}", self.ch)
        } else {
            if self.flags & BOLD_ON != 0 {
                write!(f, "\x1B[1m")?;
            }
            if self.flags & ITALICS_ON != 0 {
                write!(f, "\x1B[3m")?;
            }
            if self.flags & UNDERLINE_ON != 0 {
                write!(f, "\x1B[4m")?;
            }
            if self.flags & INVERSE_ON != 0 {
                write!(f, "\x1B[7m")?;
            }
            write!(f, "{}", self.ch)?;
            if self.flags & BOLD_OFF != 0 {
                write!(f, "\x1B[22m")?;
            }
            if self.flags & ITALICS_OFF != 0 {
                write!(f, "\x1B[23m")?;
            }
            if self.flags & UNDERLINE_OFF != 0 {
                write!(f, "\x1B[24m")?;
            }
            if self.flags & INVERSE_OFF != 0 {
                write!(f, "\x1B[27m")?;
            }

            Ok(())
        }
    }
}

impl TextStyles {
    pub fn new() -> TextStyles {
        TextStyles { inner: 0 }
    }

    pub fn bold(mut self, yes: bool) -> TextStyles {
        self.inner |= (yes as u8) << BOLD_POS;
        self
    }

    pub fn italics(mut self, yes: bool) -> TextStyles {
        self.inner |= (yes as u8) << ITALICS_POS;
        self
    }

    pub fn underline(mut self, yes: bool) -> TextStyles {
        self.inner |= (yes as u8) << UNDERLINE_POS;
        self
    }

    pub fn inverse(mut self, yes: bool) -> TextStyles {
        self.inner |= (yes as u8) << INVERSE_POS;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_bounds_works() {
        let mut c = Canvas::new(10, 10, '#');
        c.text("foo", 12, 12, TextStyles::new().underline(true));
        println!("{}", c);
    }

    #[test]
    fn overflowing_text_works() {
        let mut c = Canvas::new(10, 10, '#');
        c.text("foo", 8, 1, TextStyles::new().underline(true));
        println!("{}", c);
    }

    #[test]
    fn newlines_work() {
        let mut c = Canvas::new(10, 10, '#');
        c.text("\nfoo\n\nbar\n", 1, 1, TextStyles::new().underline(true));
        println!("{}", c);
    }

    #[test]
    fn lines_work() {
        let mut c = Canvas::new(10, 10, '#');
        c.line('-', 1, 2, 11, TextStyles::new().inverse(true));
        println!("{}", c);
    }
}
