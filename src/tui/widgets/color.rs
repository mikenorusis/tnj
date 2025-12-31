use ratatui::style::Color;

/// Named colors supported for cycling
/// Note: "lightgray" is not included because ratatui doesn't support it as a distinct color
/// (it maps to Color::Gray), which would cause color mutation issues when cycling
pub const NAMED_COLORS: &[&str] = &[
    "black", "red", "green", "yellow", "blue", 
    "magenta", "cyan", "white", "gray",
    "darkgray", "lightred", "lightgreen", "lightyellow",
    "lightblue", "lightmagenta", "lightcyan",
];

/// Parse a color string into a ratatui Color
/// Supports:
/// - Named colors: black, red, green, yellow, blue, magenta, cyan, white, gray/grey
/// - Extended named colors: darkgray, lightred, lightgreen, lightyellow, lightblue, lightmagenta, lightcyan
/// - Note: "lightgray" is accepted but maps to Color::Gray (not included in NAMED_COLORS for cycling)
/// - Hex format: #RRGGBB or #RGB (short form)
/// - RGB format: rgb(255,0,0) or rgb(255, 0, 0) (with spaces)
/// Returns Color::White as default for unrecognized colors
pub fn parse_color(color_str: &str) -> Color {
    let s = color_str.trim().to_lowercase();
    
    // 1. Try named colors (basic + extended)
    match s.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "lightgray" | "lightgrey" => Color::Gray, // LightGray not available, use Gray
        _ => {
            // 2. Try hex format: #RRGGBB or #RGB
            if s.starts_with('#') {
                if let Some(color) = parse_hex_color(&s) {
                    return color;
                }
            }
            // 3. Try rgb() format
            else if s.starts_with("rgb(") {
                if let Some(color) = parse_rgb_color(&s) {
                    return color;
                }
            }
            // 4. Fallback
            Color::White
        }
    }
}

/// Parse hex color format (#RRGGBB or #RGB)
fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.trim_start_matches('#');
    
    if hex.len() == 6 {
        // Full format: #RRGGBB
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Some(Color::Rgb(r, g, b));
        }
    } else if hex.len() == 3 {
        // Short format: #RGB -> #RRGGBB
        let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
        // Expand: 0x0 -> 0x00, 0xF -> 0xFF
        let r = (r << 4) | r;
        let g = (g << 4) | g;
        let b = (b << 4) | b;
        return Some(Color::Rgb(r, g, b));
    }
    
    None
}

/// Parse RGB color format (rgb(r,g,b) or rgb(r, g, b))
fn parse_rgb_color(s: &str) -> Option<Color> {
    // Remove "rgb(" prefix and ")" suffix
    let content = s
        .strip_prefix("rgb(")?
        .strip_suffix(')')?;
    
    // Split by comma and parse each component
    let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return None;
    }
    
    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    
    Some(Color::Rgb(r, g, b))
}

/// Format a Color back to string for display
pub fn format_color_for_display(color: &Color) -> String {
    match color {
        Color::Black => "black".to_string(),
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::White => "white".to_string(),
        Color::Gray => "gray".to_string(),
        Color::DarkGray => "darkgray".to_string(),
        Color::LightRed => "lightred".to_string(),
        Color::LightGreen => "lightgreen".to_string(),
        Color::LightYellow => "lightyellow".to_string(),
        Color::LightBlue => "lightblue".to_string(),
        Color::LightMagenta => "lightmagenta".to_string(),
        Color::LightCyan => "lightcyan".to_string(),
        // LightGray not available in ratatui, use Gray
        // Color::LightGray => "lightgray".to_string(),
        Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        Color::Indexed(_) => "indexed".to_string(),
        Color::Reset => "reset".to_string(),
    }
}

/// Calculate relative luminance for a color (WCAG formula)
/// Returns a value between 0.0 (dark) and 1.0 (light)
fn calculate_luminance(color: Color) -> f64 {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => {
            // Normalize to 0-1 range
            (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
        }
        // For named colors, use approximate RGB values
        Color::Black => (0.0, 0.0, 0.0),
        Color::Red => (1.0, 0.0, 0.0),
        Color::Green => (0.0, 1.0, 0.0),
        Color::Yellow => (1.0, 1.0, 0.0),
        Color::Blue => (0.0, 0.0, 1.0),
        Color::Magenta => (1.0, 0.0, 1.0),
        Color::Cyan => (0.0, 1.0, 1.0),
        Color::White => (1.0, 1.0, 1.0),
        Color::Gray => (0.5, 0.5, 0.5),
        Color::DarkGray => (0.25, 0.25, 0.25),
        Color::LightRed => (1.0, 0.5, 0.5),
        Color::LightGreen => (0.5, 1.0, 0.5),
        Color::LightYellow => (1.0, 1.0, 0.5),
        Color::LightBlue => (0.5, 0.5, 1.0),
        Color::LightMagenta => (1.0, 0.5, 1.0),
        Color::LightCyan => (0.5, 1.0, 1.0),
        // LightGray not available in ratatui, use Gray
        // Color::LightGray => (0.75, 0.75, 0.75),
        Color::Indexed(_) => (0.5, 0.5, 0.5), // Default to medium gray
        Color::Reset => (0.5, 0.5, 0.5), // Default to medium gray
    };
    
    // Apply gamma correction
    let r_linear = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g_linear = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b_linear = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };
    
    // Calculate relative luminance
    0.2126 * r_linear + 0.7152 * g_linear + 0.0722 * b_linear
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
/// Uses luminance calculation for RGB colors, heuristic for named colors
pub fn get_contrast_text_color(background: Color) -> Color {
    // For RGB colors, use luminance calculation
    if matches!(background, Color::Rgb(_, _, _)) {
        let luminance = calculate_luminance(background);
        if luminance < 0.5 {
            Color::White
        } else {
            Color::Black
        }
    } else {
        // For named colors, use simple heuristic
        if is_dark_color(background) {
            Color::White
        } else {
            Color::Black
        }
    }
}

/// Get an appropriate foreground color for text on a given background color string
/// Parses the background color first, then returns a contrasting text color
pub fn get_contrast_text_color_from_str(background_str: &str) -> Color {
    let bg_color = parse_color(background_str);
    get_contrast_text_color(bg_color)
}

