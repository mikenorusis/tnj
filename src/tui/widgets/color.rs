use ratatui::style::Color;

/// Parse a color string into a ratatui Color
/// Supports: black, red, green, yellow, blue, magenta, cyan, white, gray/grey
/// Returns Color::White as default for unrecognized colors
pub fn parse_color(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        _ => Color::White,
    }
}

/// Determine if a color is considered "dark" (needs light text)
/// This is a simple heuristic based on common terminal color brightness
/// Note: Gray is typically rendered as light in most terminals, so it's treated as light
fn is_dark_color(color: Color) -> bool {
    matches!(
        color,
        Color::Black | Color::Blue | Color::Magenta | Color::Red
    )
}

/// Get an appropriate foreground color for text on a given background color
/// Returns black for light backgrounds, white for dark backgrounds
/// This ensures good contrast for readability
pub fn get_contrast_text_color(background: Color) -> Color {
    if is_dark_color(background) {
        Color::White
    } else {
        Color::Black
    }
}

/// Get an appropriate foreground color for text on a given background color string
/// Parses the background color first, then returns a contrasting text color
pub fn get_contrast_text_color_from_str(background_str: &str) -> Color {
    let bg_color = parse_color(background_str);
    get_contrast_text_color(bg_color)
}

