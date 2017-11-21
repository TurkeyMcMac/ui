use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

mod canvas;
use canvas::{Canvas, TextStyles};

pub enum Response<'a> {
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
    fn select(&mut self);

    fn unselect(&mut self);

    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize);

    fn advance(&mut self);

    fn draw_advance(&mut self, canvas: &mut Canvas, x: usize, y: usize) {
        self.draw(canvas, x, y);
        self.advance()
    }

    fn respond<'b>(&'b mut self, input: char) -> Response<'b> {
        match input {
            UP    => Response::MoveUp,
            DOWN  => Response::MoveDown,
            RIGHT => Response::MoveRight,
            LEFT  => Response::MoveLeft,
            _     => Response::Contained,
        }
    }

    fn enter_top(&mut self) {
        self.select()
    }

    fn enter_bottom(&mut self) {
        self.select()
    }

    fn enter_right(&mut self) {
        self.select()
    }

    fn enter_left(&mut self) {
        self.select()
    }

    fn alert(&mut self) -> Option<&[ElemHandle]> {
        None
    }
}

pub struct Text<'a> {
    inner: &'a str,
    selected: bool,
    updated: bool,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Text<'a> {
        Text {
            inner: text,
            selected: false,
            updated: true,
        }
    }
}

impl<'a> Element<'a> for Text<'a> {
    fn select(&mut self) {
        self.selected = true;
        self.updated = true
    }

    fn unselect(&mut self) {
        self.selected = false;
        self.updated = true
    }

    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize) {
        if self.updated {
            canvas.text(self.inner, x, y, TextStyles::new().inverse(self.selected))
        }
    }

    fn advance(&mut self) {
        self.updated = false
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
        grid.focus_mut().elem.select();
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
            self.focus_mut().elem.unselect();
            self.focus = self.focus().up as usize;
            self.focus_mut().elem.enter_bottom();
            Response::Contained
        } else {
            Response::MoveUp
        }
    }

    fn move_down<'b>(&'b mut self) -> Response<'b> {
        if self.focus().down >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().down as usize;
            self.focus_mut().elem.enter_top();
            Response::Contained
        } else {
            Response::MoveDown
        }
    }

    fn move_right<'b>(&'b mut self) -> Response<'b> {
        if self.focus().right >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().right as usize;
            self.focus_mut().elem.enter_left();
            Response::Contained
        } else {
            Response::MoveRight
        }
    }

    fn move_left<'b>(&'b mut self) -> Response<'b> {
        if self.focus().left >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().left as usize;
            self.focus_mut().elem.enter_right();
            Response::Contained
        } else {
            Response::MoveLeft
        }
    }

    fn alert_all(&mut self, targets: &[ElemHandle]) {
        for t in targets {
            if let Some(targets) = unsafe { // TODO: Do this a better way
                (&mut *(&self.elems as *const _ as *mut Vec<ElemHolder<'a>>))
            }.get_mut(t.0).and_then(|e| e.elem.alert())
            {
                self.alert_all(targets);
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
    fn select(&mut self) {
        self.focus = TL_IDX;
        self.focus_mut().elem.select()
    }

    fn unselect(&mut self) {
        self.focus_mut().elem.unselect()
    }

    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize) {
        for &ElemHolder { ref elem, x: elem_x, y: elem_y, .. } in &self.elems {
            elem.draw(canvas, x + elem_x, y + elem_y)
        }
    }

    fn advance(&mut self) {
        for &mut ElemHolder { ref mut elem, .. } in &mut self.elems {
            elem.advance()
        }
    }

    fn draw_advance(&mut self, canvas: &mut Canvas, x: usize, y: usize) {
        for &mut ElemHolder { ref mut elem, x: elem_x, y: elem_y, .. } in &mut self.elems {
            elem.draw_advance(canvas, x + elem_x, y + elem_y)
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
            Response::Contained      => Response::Contained,
            Response::Alert(targets) => {
                self.alert_all(targets);
                Response::Contained
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Counter<'a> {
        count: &'a mut u32,
        selected: bool,
        updated: bool,
    }

    impl<'a> Element<'a> for Counter<'a> {
        fn select(&mut self) {
            *self.count += 2;
            self.selected = true;
            self.updated = true
        }

        fn unselect(&mut self) {
            *self.count -= 1;
            self.selected = false;
            self.updated = true
        }

        fn draw(&self, canvas: &mut Canvas, x: usize, y: usize) {
            if self.updated {
                canvas.text(&self.count.to_string(), x, y, TextStyles::new().inverse(self.selected))
            }
        }

        fn advance(&mut self) {
            self.updated = false
        }
    }

    #[test]
    fn it_works() {
        let mut counter = 1000;
        let count: &mut u32 = &mut counter;
        let mut grid = Grid::with_capacity(Box::new(Text::new("foo")), 1, 1, Box::new(Text::new("baz")), 3, 3, 10);
        let top = grid.top_left();
        let bottom = grid.bottom_right();
        let middle = grid.add_elem(Box::new(Counter { count, selected: false, updated: true }), 2, 2);
        grid.connect_up_down(top, middle).unwrap();
        grid.connect_up_down(middle, bottom).unwrap();
        grid.connect_left_right(top, middle).unwrap();
        grid.connect_left_right(middle, bottom).unwrap();
        let mut canvas = Canvas::new(10, 10, ' ');
        grid.draw_advance(&mut canvas, 0, 0);
        print!("{}", canvas);
        grid.draw_advance(&mut canvas, 0, 0);
        print!("{}", canvas);
        grid.respond(DOWN);
        grid.draw_advance(&mut canvas, 0, 0);
        print!("{}", canvas);
        grid.respond(DOWN);
        grid.draw_advance(&mut canvas, 0, 0);
        print!("{}", canvas);
        grid.respond(LEFT);
        grid.draw_advance(&mut canvas, 0, 0);
        print!("{}", canvas);
        grid.respond(RIGHT);
        grid.draw_advance(&mut canvas, 0, 0);
        print!("{}", canvas);
    }
}
