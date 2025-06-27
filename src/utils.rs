use crate::buffer::Buffer;
use crossterm::style::{Color, StyledContent, Stylize};

// A Rect is a description of an area where:
// * x and y are the coordinates for the top left corner
// * width is the max value that x can get to
// * height is the max value that y can get to
#[derive(Clone, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub height: u16,
    pub width: u16,
}

// Draw border on a specific area
// Widget can use this function to draw borders
pub fn border(
    buffer: &mut Buffer,
    // TODO: Maybe we should pass a mutable reference here because the buffer will make the area
    // smaller. This function can update the area with the new boundries
    area: &Rect,
    focused: bool,
    title: String,
    header: Option<StyledContent<String>>,
    footer: Option<StyledContent<String>>,
) {
    let line_color = if focused { Color::Yellow } else { Color::Reset };
    let first_line = build_border_line('┌', '┐', '─', area.width);
    let last_line = build_border_line('└', '┘', '─', area.width);
    // Add offset if header or footer is defined to make everything fit
    let top_offset = if header.is_some() { 1 } else { 0 };
    let bottom_offset = if footer.is_some() { 1 } else { 0 };

    // Draw Header
    if let Some(header) = header {
        buffer.write_string(area.x, area.y, header);
    }

    // Draw first line with nice curves
    buffer.write_string(area.x, area.y + top_offset, first_line.with(line_color));
    buffer.write_string(
        area.x + ((area.width / 2) - title.len() as u16),
        area.y + top_offset,
        title.bold(),
    );
    // Draw vertical lines only on the left most and right most column
    for i in 1..area.height - top_offset - bottom_offset {
        buffer.write_string(
            area.x,
            area.y + top_offset + i,
            String::from("│").with(line_color),
        );
        buffer.write_string(
            area.x + area.width - 1,
            area.y + top_offset + i,
            String::from("│").with(line_color),
        );
    }
    // Draw last line with nice curves
    buffer.write_string(
        area.x,
        area.y + area.height - 1 - bottom_offset,
        last_line.with(line_color),
    );

    // Draw footer
    if let Some(footer) = footer {
        buffer.write_string(area.x, area.y + area.height - 1, footer);
    }
}

// Helper method to help building borders easier
fn build_border_line(first: char, last: char, middle: char, length: u16) -> String {
    let mut line = String::new();
    line.push(first);
    for _ in 0..length - 2 {
        line.push(middle);
    }
    line.push(last);

    line
}

pub fn split_rect_by_height(rect: &Rect) -> (Rect, Rect) {
    let height = rect.height / 2;
    let mut one = rect.clone();
    let mut two = rect.clone();

    one.height = height + 1;
    two.y += height + 1;
    two.height = height - 1;

    (one, two)
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
