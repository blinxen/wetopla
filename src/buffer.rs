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

    fn relative_cell_position(&self, x: u16, y: u16) -> (u16, u16) {
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

        (x, y)
    }

    pub fn write_string(&mut self, x: u16, y: u16, content: StyledContent<String>) {
        // TODO: truncate if string does not fit in line
        let (x, y) = self.relative_cell_position(x, y);
        for (i, char) in content.content().chars().enumerate() {
            self.current[y as usize][x as usize + i] = StyledContent::new(*content.style(), char);
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        for (row, rows) in self.current.iter().enumerate() {
            for (column, cell) in rows.iter().enumerate() {
                if &self.previous[row][column] != cell {
                    self.stdout.queue(MoveTo(
                        column as u16 + self.area.x,
                        row as u16 + self.area.y,
                    ))?;
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
