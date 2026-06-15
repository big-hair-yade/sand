use crossterm::{cursor, queue};
use std::io::{Write, stdout};

//Implement display features
pub trait ParseRow {
    fn parse(&self, frac: f32, inner: char, outer: char) -> String;
    fn writerow(&self, frac: f32, inner: char, outer: char);
}
struct Row {
    pub x_pos: u16,
    pub y_pos: u16,
    pub length: u16,
}
impl ParseRow for Row {
    fn parse(&self, frac: f32, inner: char, outer: char) -> String {
        let filled_cells = (frac * (self.length as f32)) as u16; //This will truncate the f32 to a u32, not round it.
        let unfilled_cells = self.length - filled_cells;
        let unfilled_cells_left = unfilled_cells / 2;
        let unfilled_cells_right = unfilled_cells - unfilled_cells_left;
        let left_row: String = std::iter::repeat_n(outer, unfilled_cells_left as usize).collect();
        let middle_row: String = std::iter::repeat_n(inner, filled_cells as usize).collect();
        let right_row: String = std::iter::repeat_n(outer, unfilled_cells_right as usize).collect();
        format!("{}{}{}", left_row, middle_row, right_row)
    }
    fn writerow(&self, frac: f32, inner: char, outer: char) {
        let _ = queue!(
            stdout(),
            cursor::MoveTo(self.x_pos as u16, self.y_pos as u16)
        );
        write!(stdout(), "{}", self.parse(frac, inner, outer)).expect("Failed to write row");
    }
}

//Circle below
fn find_first_cell_per_row_relative(y_relative: u16, radius: f32) -> Option<u16> {
    let mut x_left: u16 = 0;
    let mut x_right: u16 = radius as u16 + 1;
    assert!(radius > 0.0);
    assert!(((x_right * x_right + y_relative * y_relative) as f32).sqrt() >= radius);
    if ((y_relative * y_relative) as f32).sqrt() >= radius {
        return None;
    }
    // Now we know that x_left is within the circle and x_right is outside of the circle
    // This means we are ready to perform a binary search.
    let mut x_test;
    while x_right - x_left > 1 {
        x_test = (x_left + x_right) / 2;
        if ((x_test * x_test + y_relative * y_relative) as f32).sqrt() >= radius {
            x_right = x_test;
        } else {
            x_left = x_test;
        }
    }

    assert!((x_right - x_left) == 1);
    Some(x_left)
}

pub trait CalculateRows {
    fn calculate_rows(&mut self, x_pos: u16, y_pos: u16, radius: f32);
}
pub trait DrawCircle {
    fn draw_circle(&self, frac: f32, fill: char, unfill: char);
}
#[derive(Default)]
pub struct Circle {
    vec_rows: Vec<Row>,
    total_characters: u16,
}
impl CalculateRows for Circle {
    fn calculate_rows(&mut self, x_pos: u16, y_pos: u16, radius: f32) {
        self.vec_rows = Vec::new();
        self.total_characters = 0;

        assert!((y_pos as i32 - (radius as i32) - 1) > 0);
        let min_bound_y: u16 = y_pos - (radius as u16) - 1;
        let max_bound_y: u16 = y_pos + (radius as u16) + 1;
        for y_idx in min_bound_y..=max_bound_y {
            assert!(y_idx as i32 + y_pos as i32 > 0);
            let y_relative = (y_idx as i32 - y_pos as i32).abs() as u16;
            let this_row_length = find_first_cell_per_row_relative(y_relative, radius);
            if let Some(length) = this_row_length {
                assert!(x_pos as i32 - length as i32 > 0);
                self.vec_rows.push(Row {
                    x_pos: x_pos - length,
                    y_pos: y_idx,
                    length: length * 2 + 1, // + 1 to account for the middle character
                });
                self.total_characters += length * 2 + 1;
            }
        }
        self.vec_rows.reverse();
    }
}
impl DrawCircle for Circle {
    fn draw_circle(&self, frac: f32, fill: char, unfill: char) {
        let mut accumulated_frac: f32 = 0.0;
        for row_idx in 0..self.vec_rows.len() {
            let row_ref: Option<&Row> = self.vec_rows.get(row_idx);
            match row_ref {
                Some(row_ref) => {
                    if (accumulated_frac + row_ref.length as f32 / self.total_characters as f32)
                        <= frac
                    {
                        row_ref.writerow(1.0, fill, fill);
                    } else if accumulated_frac > frac {
                        row_ref.writerow(1.0, unfill, unfill);
                    } else {
                        let row_frac: f32 = (frac - accumulated_frac)
                            * self.total_characters as f32
                            / row_ref.length as f32;
                        row_ref.writerow(row_frac, fill, unfill);
                    }
                    accumulated_frac += row_ref.length as f32 / self.total_characters as f32;
                }
                None => println!("Failure printing rows"),
            }
        }
    }
}

pub fn resize_hourglass(
    top_circle: &mut Circle,
    bottom_circle: &mut Circle,
    size_x: u16,
    size_y: u16,
) {
    let x_middle = size_x / 2;
    let y_middle = size_y / 2;
    let y_quarter = size_y / 4;
    let radius = size_y as f32 / 5.0;
    top_circle.calculate_rows(x_middle, y_quarter, radius);
    bottom_circle.calculate_rows(x_middle, y_quarter + y_middle, radius);
}
