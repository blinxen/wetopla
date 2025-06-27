use crate::utils::Rect;
use crossterm::{
    cursor::MoveTo,
    style::{ContentStyle, PrintStyledContent, StyledContent},
    QueueableCommand,
};
use std::io::{stdout, Stdout, Write};

pub struct Buffer {
    current: Vec<Vec<StyledContent<char>>>,
    previous: Vec<Vec<StyledContent<char>>>,
    stdout: Stdout,
    area: Rect,
}

impl Buffer {
    // Create a buffer that has only empty cells
    pub fn new(area: Rect) -> Self {
        Buffer {
            current: Buffer::init_buffer(area.height, area.width),
            previous: Buffer::init_buffer(area.height, area.width),
            stdout: stdout(),
            area,
        }
    }

    pub fn resize(&mut self, area: &Rect) {
        if self.area == *area {
            return;
        }
        self.area = area.clone();
        self.previous = Buffer::init_buffer(self.area.height, self.area.width);
        self.current = Buffer::init_buffer(self.area.height, self.area.width);
    }

    fn init_buffer(rows: u16, columns: u16) -> Vec<Vec<StyledContent<char>>> {
        vec![
            vec![StyledContent::new(ContentStyle::default(), ' '); columns as usize];
            rows as usize
        ]
    }

    fn relative_cell_position(&self, x: u16, y: u16) -> (usize, usize) {
        if x < self.area.x {
            panic!("x ({}) cannot be smaller than {}", x, self.area.x);
        }

        if y < self.area.y {
            panic!("y ({}) cannot be smaller than {}", y, self.area.y);
        }

        let (x, y) = (x - self.area.x, y - self.area.y);

        if y >= self.area.height {
            panic!(
                "Y coordinate outside of useable area ({} > {})",
                y, self.area.height
            )
        }

        if x >= self.area.width {
            panic!(
                "X coordinate outside of useable area ({} > {})",
                x, self.area.width
            )
        }

        (x as usize, y as usize)
    }

    pub fn write_string(&mut self, x: u16, y: u16, content: StyledContent<String>) {
        let (x, y) = self.relative_cell_position(x, y);
        for (i, char) in content.content().chars().enumerate() {
            if self.current.len() > y && self.current[y].len() > x + i {
                self.current[y][x + i] = StyledContent::new(*content.style(), char);
            } else {
                break;
            }
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        'outer: for (row, rows) in self.current.iter().enumerate() {
            for (column, cell) in rows.iter().enumerate() {
                if &self.previous[row][column] != cell {
                    let abs_column = column as u16 + self.area.x;
                    let abs_row = row as u16 + self.area.y;

                    // Don't show lines that don't fit vertically
                    if self.area.height < abs_row {
                        continue 'outer;
                    }

                    self.stdout.queue(MoveTo(abs_column, abs_row))?;

                    // Truncate lines that don't fit horizontally
                    if self.area.width < abs_column {
                        self.stdout.queue(MoveTo(abs_column - 4, abs_row))?;
                        self.stdout.queue(PrintStyledContent(StyledContent::new(
                            *cell.style(),
                            "<<<<",
                        )))?;
                        continue 'outer;
                    }

                    self.stdout.queue(PrintStyledContent(*cell))?;
                    self.previous[row][column] = *cell;
                }
            }
        }

        self.current = Buffer::init_buffer(self.area.height, self.area.width);
        self.stdout.flush()?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.previous = Buffer::init_buffer(self.area.height, self.area.width);
        self.current = Buffer::init_buffer(self.area.height, self.area.width);
    }
}
