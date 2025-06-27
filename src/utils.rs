use crossterm::cursor::{MoveTo, MoveToColumn, MoveToNextLine};
use crossterm::style::{
    Color, Colors, PrintStyledContent, SetBackgroundColor, SetForegroundColor, StyledContent,
    Stylize,
};
use crossterm::{cursor, QueueableCommand};
use std::io::Stdout;
use std::io::Write;

const HIGHLIGHTED_BUTTON_COLORS: Colors = Colors {
    background: Some(Color::White),
    foreground: Some(Color::Black),
};
const NORMAL_BUTTON_COLORS: Colors = Colors {
    background: Some(Color::Reset),
    foreground: Some(Color::White),
};

// A Rect is a description of an area where:
// * x and y are the coordinates for the top left corner
// * width is the max value that x can get to
// * height is the max value that y can get to
#[derive(Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub height: u16,
    pub width: u16,
}

// Draw border on a specific area
// Widget can use this function to draw borders
pub fn border(
    stdout: &mut Stdout,
    area: &Rect,
    title: &str,
    focused: bool,
) -> Result<(), std::io::Error> {
    let first_line = build_line('┌', '┐', '─', title, area.width);
    let last_line = build_line('└', '┘', '─', "", area.width);

    // Draw first line with nice curves
    stdout.queue(border_background(focused))?;
    reset_cursor_in_area(stdout, area)?;
    stdout.write_all(first_line.as_bytes())?;
    stdout.queue(cursor::MoveToColumn(area.x + 1))?;
    stdout.queue(PrintStyledContent(title.reset().bold()))?;
    // We need to re-apply the background color since "PrintStyledContent" resets it
    stdout.queue(border_background(focused))?;
    // Draw vertical lines only on the left most and right most column
    for _ in 1..area.height {
        go_to_next_line_in_area(stdout, area, 0)?;
        stdout.write_all("│".as_bytes())?;
        stdout.queue(cursor::MoveRight(area.width - 2))?;
        stdout.write_all("│".as_bytes())?;
    }
    // Draw last line with nice curves
    go_to_next_line_in_area(stdout, area, 0)?;
    stdout.write_all(last_line.as_bytes())?;
    stdout.queue(SetForegroundColor(Color::Reset))?;

    Ok(())
}

fn border_background(focused: bool) -> SetForegroundColor {
    if focused {
        SetForegroundColor(Color::Yellow)
    } else {
        SetForegroundColor(Color::White)
    }
}

// Helper method to help building borders easier
fn build_line(first: char, last: char, middle: char, title: &str, length: u16) -> String {
    let mut line = String::new();
    line.push(first);
    line.push_str(&" ".repeat(title.len()));
    for _ in 0..length - 2 - title.len() as u16 {
        line.push(middle);
    }
    line.push(last);

    line
}

pub fn highlight_button_text(
    stdout: &mut Stdout,
    text: &str,
    highlight: bool,
) -> Result<(), std::io::Error> {
    // Set highlight colors
    if highlight {
        // Unwrap is safe because we build the colors ourselfs
        stdout.queue(SetForegroundColor(
            HIGHLIGHTED_BUTTON_COLORS.foreground.unwrap(),
        ))?;
        stdout.queue(SetBackgroundColor(
            HIGHLIGHTED_BUTTON_COLORS.background.unwrap(),
        ))?;
    }
    stdout.write_all(text.as_bytes())?;
    // Reset background or foreground
    stdout.queue(SetForegroundColor(NORMAL_BUTTON_COLORS.foreground.unwrap()))?;
    stdout.queue(SetBackgroundColor(NORMAL_BUTTON_COLORS.background.unwrap()))?;

    Ok(())
}

pub fn split_rect_by_height(rect: &Rect) -> (Rect, Rect) {
    let height = rect.height / 2;
    let mut one = rect.clone();
    let mut two = rect.clone();

    one.height = height;
    two.y += height + 1;
    two.height = height - 1;

    (one, two)
}

// Go to the next line in a specific area
// x_offset can be set to allow moving on the X axis after the new line has been inserted
pub fn go_to_next_line_in_area(
    stdout: &mut Stdout,
    area: &Rect,
    x_offset: u16,
) -> Result<(), std::io::Error> {
    stdout.queue(MoveToNextLine(1))?;
    stdout.queue(MoveToColumn(area.x + x_offset))?;

    Ok(())
}

// Place cursor at the top left corner of an area
pub fn reset_cursor_in_area(stdout: &mut Stdout, area: &Rect) -> Result<(), std::io::Error> {
    stdout.queue(MoveTo(area.x, area.y))?;
    Ok(())
}

// Build a row that will be displayed in a container
pub fn build_row(contents: Vec<(&str, usize)>) -> StyledContent<String> {
    let mut row = String::new();

    for (content, space) in contents {
        row.push_str(content);
        row.push_str(&" ".repeat(space - content.len()));
    }

    row.stylize()
}
