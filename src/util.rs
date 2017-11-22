use ::{Element, Response};
use canvas::Canvas;

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

