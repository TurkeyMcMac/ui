use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

mod canvas;
use canvas::{Canvas, TextStyles};

pub trait Element<'a> {
    fn select(&mut self);

    fn unselect(&mut self);

    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize);

    fn advance(&mut self);

    fn draw_advance(&mut self, canvas: &mut Canvas, x: usize, y: usize) {
        self.draw(canvas, x, y);
        self.advance()
    }

    fn respond(&mut self, _input: char) { }

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

    fn move_up(&mut self) -> bool {
        false
    }

    fn move_down(&mut self) -> bool {
        false
    }

    fn move_right(&mut self) -> bool {
        false
    }

    fn move_left(&mut self) -> bool {
        false
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
    pub fn with_capacity(tl: Box<Element<'a>>, tl_x: usize, tl_y: usize,
                         br: Box<Element<'a>>, br_x: usize, br_y: usize,
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
    elem: Box<Element<'a>>,
    x: usize,
    y: usize,
    up: isize,
    down: isize,
    right: isize,
    left: isize,
}

impl<'a> ElemHolder<'a> {
    pub fn new(elem: Box<Element<'a>>, x: usize, y: usize) -> ElemHolder<'a> {
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

    fn advance(&mut self) {
        for &mut ElemHolder { ref mut elem, .. } in &mut self.elems {
            elem.advance()
        }
    }

    fn draw(&self, canvas: &mut Canvas, x: usize, y: usize) {
        for &ElemHolder { ref elem, x: elem_x, y: elem_y, .. } in &self.elems {
            elem.draw(canvas, x + elem_x, y + elem_y)
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

    fn move_up(&mut self) -> bool {
        if self.focus_mut().elem.move_up() {
            true
        } else if self.focus().up >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().up as usize;
            self.focus_mut().elem.enter_bottom();
            true
        } else {
            false
        }
    }

    fn move_down(&mut self) -> bool {
        if self.focus_mut().elem.move_down() {
            true
        } else if self.focus().down >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().down as usize;
            self.focus_mut().elem.enter_top();
            true
        } else {
            false
        }
    }

    fn move_right(&mut self) -> bool {
        if self.focus_mut().elem.move_right() {
            true
        } else if self.focus().right >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().right as usize;
            self.focus_mut().elem.enter_left();
            true
        } else {
            false
        }
    }

    fn move_left(&mut self) -> bool {
        if self.focus_mut().elem.move_left() {
            true
        } else if self.focus().left >= 0 {
            self.focus_mut().elem.unselect();
            self.focus = self.focus().left as usize;
            self.focus_mut().elem.enter_right();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut grid = Grid::with_capacity(Box::new(Text::new("foo")), 0, 0, Box::new(Text::new("bar")), 5, 5, 10);
        let top = grid.top_left();
        let bottom = grid.bottom_right();
        grid.connect_up_down(top, bottom).unwrap();
        grid.connect_left_right(top, bottom).unwrap();
        println!("{}", grid.focus as usize);
        let mut canvas = Canvas::new(10, 10, ' ');
        grid.draw(&mut canvas, 0, 0);
        print!("{}", canvas);
        println!("{}", grid.move_down());
        grid.draw(&mut canvas, 0, 0);
        print!("{}", canvas);
        println!("{}", grid.move_up());
        grid.draw(&mut canvas, 0, 0);
        print!("{}", canvas);
        println!("{}", grid.move_right());
        grid.draw(&mut canvas, 0, 0);
        print!("{}", canvas);
    }
}
