use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;

mod canvas;
use canvas::{Canvas, TextStyles};

pub enum Response<'a> {
    Nothing,
    Contained,
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
    Alert(&'a [ElemHandle]),
}

pub const UP: char = 'k';
pub const DOWN: char = 'j';
pub const RIGHT: char = 'l';
pub const LEFT: char = 'h';

pub trait Element<'a> {
    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, selected: bool);

    fn advance(&mut self) { }

    fn draw_advance(&mut self, canvas: &mut Canvas, x: usize, y: usize, selected: bool) {
        self.draw(canvas, x, y, selected);
        self.advance()
    }

    fn respond<'b>(&'b mut self, input: char) -> Response<'b> {
        match input {
            UP    => Response::MoveUp,
            DOWN  => Response::MoveDown,
            RIGHT => Response::MoveRight,
            LEFT  => Response::MoveLeft,
            _     => Response::Nothing,
        }
    }

    fn enter_top(&mut self) { }

    fn enter_bottom(&mut self) { }

    fn enter_right(&mut self) { }

    fn enter_left(&mut self) { }

    fn alert(&mut self) { }
}

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

pub struct Text<'a> {
    inner: &'a str,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Text<'a> {
        Text {
            inner: text,
        }
    }
}

impl<'a> Element<'a> for Text<'a> {
    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, selected: bool) {
        canvas.text(self.inner, x, y, TextStyles::new().inverse(selected))
    }
}

pub struct Grid<'a> {
    pub elems: Vec<ElemHolder<'a>>,
    pub focus: usize,
}

const TL_IDX: usize = 0; // Index of top left element
const BR_IDX: usize = 1; // Index of bottom right element

impl<'a> Grid<'a> {
    pub fn with_capacity(tl: Box<Element<'a> + 'a>, tl_x: usize, tl_y: usize,
                         br: Box<Element<'a> + 'a>, br_x: usize, br_y: usize,
                         cap: usize)
    -> Grid<'a> {
        let mut grid = Grid {
            elems: {
                let mut elems = Vec::with_capacity(cap + 2);
                elems.push(ElemHolder::new(tl, tl_x, tl_y));
                elems.push(ElemHolder::new(br, br_x, br_y));
                elems
            },
            focus: TL_IDX,
        };
        grid
    }

    fn focus(&self) -> &ElemHolder<'a> {
        unsafe {
            self.elems.get_unchecked(self.focus)
        }
    }

    fn focus_mut(&mut self) -> &mut ElemHolder<'a> {
        unsafe {
            self.elems.get_unchecked_mut(self.focus)
        }
    }

    pub fn top_left(&self) -> ElemHandle {
        ElemHandle(TL_IDX)
    }

    pub fn bottom_right(&self) -> ElemHandle {
        ElemHandle(BR_IDX)
    }

    pub fn add_elem(&mut self, elem: Box<Element<'a> + 'a>, x: usize, y: usize) -> ElemHandle {
        self.elems.push(ElemHolder::new(elem, x, y));
        ElemHandle(self.elems.len() - 1)
    }

    pub fn connect_up_down(&mut self, up: ElemHandle, down: ElemHandle) -> Result<(), HandleOutOfBounds> {
        let upper: *mut ElemHolder<'a> = self.elems.get(up.0).ok_or(HandleOutOfBounds(up))? as *const _ as *mut _;
        let lower: *mut ElemHolder<'a> = self.elems.get(down.0).ok_or(HandleOutOfBounds(down))? as *const _ as *mut _;
        unsafe {
            (&mut *upper).down = down.0 as isize;
            (&mut *lower).up = up.0 as isize;
        }

        Ok(())
    }

    pub fn connect_left_right(&mut self, left: ElemHandle, right: ElemHandle) -> Result<(), HandleOutOfBounds> {
        let lefter: *mut ElemHolder<'a> = self.elems.get(left.0).ok_or(HandleOutOfBounds(left))? as *const _ as *mut _;
        let righter: *mut ElemHolder<'a> = self.elems.get(right.0).ok_or(HandleOutOfBounds(right))? as *const _ as *mut _;
        unsafe {
            (&mut *lefter).right = right.0 as isize;
            (&mut *righter).left = left.0 as isize;
        }

        Ok(())
    }

    fn move_up<'b>(&'b mut self) -> Response<'b> {
        if self.focus().up >= 0 {
            self.focus = self.focus().up as usize;
            self.focus_mut().elem.enter_top();
            Response::Contained
        } else {
            Response::MoveUp
        }
    }

    fn move_down<'b>(&'b mut self) -> Response<'b> {
        if self.focus().down >= 0 {
            self.focus = self.focus().down as usize;
            self.focus_mut().elem.enter_bottom();
            Response::Contained
        } else {
            Response::MoveDown
        }
    }

    fn move_right<'b>(&'b mut self) -> Response<'b> {
        if self.focus().right >= 0 {
            self.focus = self.focus().right as usize;
            self.focus_mut().elem.enter_right();
            Response::Contained
        } else {
            Response::MoveRight
        }
    }

    fn move_left<'b>(&'b mut self) -> Response<'b> {
        if self.focus().left >= 0 {
            self.focus = self.focus().left as usize;
            self.focus_mut().elem.enter_left();
            Response::Contained
        } else {
            Response::MoveLeft
        }
    }

    fn alert_all(&mut self, targets: &[ElemHandle]) {
        for t in targets {
            if let Some(e) = self.elems.get_mut(t.0) {
                e.elem.alert()
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct HandleOutOfBounds(ElemHandle);

impl HandleOutOfBounds {
    pub fn handle(self) -> ElemHandle {
        self.0
    }
}

impl Error for HandleOutOfBounds {
    fn description(&self) -> &str {
        "An element handle was invalid for the element grid on which it was used"
    }
}

impl Debug for HandleOutOfBounds {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "HandleOutOfBounds")
    }
}

impl Display for HandleOutOfBounds {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "HandleOutOfBounds")
    }
}

#[derive(Clone, Copy)]
pub struct ElemHandle(usize);

pub struct ElemHolder<'a> {
    elem: Box<Element<'a> + 'a>,
    x: usize,
    y: usize,
    up: isize,
    down: isize,
    right: isize,
    left: isize,
}

impl<'a> ElemHolder<'a> {
    pub fn new(elem: Box<Element<'a> + 'a>, x: usize, y: usize) -> ElemHolder<'a> {
        ElemHolder {
            elem, x, y,
            up: -1,
            down: -1,
            right: -1,
            left: -1,
        }
    }
}

impl<'a> Element<'a> for Grid<'a> {
    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, selected: bool) {
        for (i, &ElemHolder { ref elem, x: elem_x, y: elem_y, .. }) in self.elems.iter().enumerate() {
            elem.draw(canvas, x + elem_x, y + elem_y, i == self.focus && selected)
        }
    }

    fn advance(&mut self) {
        for &mut ElemHolder { ref mut elem, .. } in &mut self.elems {
            elem.advance()
        }
    }

    fn draw_advance(&mut self, canvas: &mut Canvas, x: usize, y: usize, selected: bool) {
        for (i, &mut ElemHolder { ref mut elem, x: elem_x, y: elem_y, .. }) in self.elems.iter_mut().enumerate() {
            elem.draw_advance(canvas, x + elem_x, y + elem_y, i == self.focus && selected)
        }
    }

    fn enter_top(&mut self) {
        self.focus = TL_IDX;
        self.focus_mut().elem.enter_top()
    }

    fn enter_bottom(&mut self) {
        self.focus = BR_IDX;
        self.focus_mut().elem.enter_bottom()
    }

    fn enter_right(&mut self) {
        self.focus = BR_IDX;
        self.focus_mut().elem.enter_right()
    }

    fn enter_left(&mut self) {
        self.focus = TL_IDX;
        self.focus_mut().elem.enter_left()
    }

    fn respond<'b>(&'b mut self, input: char) -> Response<'b> {
        let response = unsafe { // TODO: Find a better way to do this maybe
            (&mut *(self as *mut Grid<'a>)).focus_mut().elem.respond(input)
        };
        match response {
            Response::MoveUp         => self.move_up(),
            Response::MoveDown       => self.move_down(),
            Response::MoveRight      => self.move_right(),
            Response::MoveLeft       => self.move_left(),
            Response::Alert(targets) => {
                self.alert_all(targets);
                Response::Contained
            },
            r => r,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Counter<'a> {
        count: &'a mut u32,
    }

    impl<'a> Element<'a> for Counter<'a> {
        fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, selected: bool) {
            canvas.text(&self.count.to_string(), x, y, TextStyles::new().inverse(selected))
        }
    }

    #[test]
    fn it_works() {
        let mut counter = 1000;
        let count: &mut u32 = &mut counter;
        let mut grid = Grid::with_capacity(Box::new(Updater::new(Text::new("\nf\no\n\no"))), 0, 0, Box::new(Updater::new(Text::new("baz"))), 3, 3, 10);
        let top = grid.top_left();
        let bottom = grid.bottom_right();
        let middle = grid.add_elem(Box::new(Updater::new(Counter { count })), 2, 2);
        grid.connect_up_down(top, middle).unwrap();
        grid.connect_up_down(middle, bottom).unwrap();
        grid.connect_left_right(top, middle).unwrap();
        grid.connect_left_right(middle, bottom).unwrap();
        let mut canvas = Canvas::new(10, 10, ' ');
        grid.draw_advance(&mut canvas, 0, 0, true);
        print!("{}", canvas);
        grid.respond(DOWN);
        grid.draw_advance(&mut canvas, 0, 0, true);
        print!("{}", canvas);
        grid.respond(UP);
        grid.draw_advance(&mut canvas, 0, 0, true);
        print!("{}", canvas);
        grid.respond(DOWN);
        grid.draw_advance(&mut canvas, 0, 0, true);
        print!("{}", canvas);
        grid.respond(UP);
        grid.draw_advance(&mut canvas, 0, 0, true);
        print!("{}", canvas);
    }
}
