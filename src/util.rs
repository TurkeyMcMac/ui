use ::{Element, Response, UP, DOWN, RIGHT, LEFT};
use canvas::{Canvas, TextStyles};

use std::marker::PhantomData;

pub struct Updater<'a, E>
    where E: Element<'a>
{
    inner: E,
    updated: bool,
    _a: PhantomData<&'a ()>,
}

impl<'a, E> Updater<'a, E>
    where E: Element<'a>
{
    pub fn new(elem: E) -> Updater<'a, E> {
        Updater {
            inner: elem,
            updated: true,
            _a: PhantomData,
        }
    }
}

impl<'a, E> Element<'a> for Updater<'a, E>
    where E: Element<'a>
{
    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, selected: bool) {
        if self.updated {
            self.inner.draw(canvas, x, y, selected)
        }
    }

    fn advance(&mut self) {
        self.inner.advance();
        self.updated = false
    }

    fn respond<'b>(&'b mut self, input: char) -> Response<'b> {
        match self.inner.respond(input) {
            Response::Nothing => Response::Nothing,
            r => {
                self.updated = true;
                r
            }
        }
    }

    fn enter_top(&mut self) {
        self.updated = true;
        self.inner.enter_top()
    }

    fn enter_bottom(&mut self) {
        self.updated = true;
        self.inner.enter_bottom()
    }

    fn enter_right(&mut self) {
        self.updated = true;
        self.inner.enter_right()
    }

    fn enter_left(&mut self) {
        self.updated = true;
        self.inner.enter_left()
    }

    fn alert(&mut self) {
        self.updated = true;
        self.inner.alert()
    }
}

pub struct TextScroller<'a> {
    lines: Vec<&'a str>,
    width: usize,
    height: usize,
    window: usize,
}

impl<'a> TextScroller<'a> {
    pub fn new(text: &'a str, width: usize, height: usize) -> TextScroller<'a> {
        TextScroller {
            lines: text.lines().collect(),
            width, height,
            window: 0,
        }
    }

    pub fn scroll_up(&mut self) -> Response {
        if self.window > 0 {
            self.window -= 1;
            Response::Contained
        } else {
            Response::MoveUp
        }
    }

    pub fn scroll_down(&mut self) -> Response {
        if self.window + self.height < self.lines.len() {
            self.window += 1;
            Response::Contained
        } else {
            Response::MoveDown
        }
    }
}

impl<'a> Element<'a> for TextScroller<'a> {
    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, _selected: bool) {
        if self.lines.len() < self.height {
            for (i, l) in self.lines.iter().enumerate() {
                canvas.text(l, x, y + i, TextStyles::new())
            }
        } else {
            for (i, l) in self.lines[self.window..self.window + self.height].iter().enumerate() {
                padded_line(canvas, l, x, y + i, self.width, ' ', TextStyles::new())
            }
        }
    }

    fn respond(&mut self, input: char) -> Response {
        match input {
            UP    => self.scroll_up(),
            DOWN  => self.scroll_down(),
            RIGHT => Response::MoveRight,
            LEFT  => Response::MoveLeft,
            _ => Response::Nothing,
        }
    }
}

fn padded_line<'a>(canvas: &mut Canvas, text: &'a str, x: usize, y: usize, length: usize, pad: char, styles: TextStyles) {
        if x >= canvas.width() || y >= canvas.height() {
            return;
        }
        let length = if x + length > canvas.width() {
            canvas.width() - x
        } else {
            length
        };
        let mut current_x = x;

        unsafe {
            canvas.get_unchecked_mut(current_x, y).set_styles_on(styles);
        }

        let mut space_left = length;

        for letter in text.chars().take(length) {
            unsafe {
                canvas.get_unchecked_mut(current_x, y).ch = letter;
            }
            current_x += 1;
            space_left -= 1;
        }

        unsafe {
            canvas.get_unchecked_mut(current_x, y).set_styles_off(styles);
        }

        canvas.line(pad, current_x, y, space_left, styles)
}
