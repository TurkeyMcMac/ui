extern crate ui;
use ui::{Element, Grid, UP, DOWN, LEFT, RIGHT};
use ui::canvas::Canvas;
use ui::util::{Updater, TextScroller};

fn main() {
    let mut grid = Grid::with_capacity(Box::new(Updater::new(TextScroller::new("a\nbb\nc\ndd\ng\nh\ni", 3, 5))), 0, 0,
                                       Box::new(Updater::new(TextScroller::new("abcdefg\nhijk\nlmnop\nqrstuv\nwxyz", 8, 3))), 11, 1,
                                       2);
    let left = grid.top_left();
    let right = grid.bottom_right();
    grid.connect_left_right(left, right);

    let mut canvas = Canvas::new(20, 20, ' ');

    grid.draw_advance(&mut canvas);
    print!("{}", canvas);
    grid.respond(DOWN);
    grid.respond(RIGHT);
    grid.respond(DOWN);
    grid.draw(&mut canvas);
    print!("{}", canvas);
}
