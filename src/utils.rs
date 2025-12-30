use directories::{ProjectDirs, BaseDirs};
use std::path::PathBuf;

/// Profile mode for the application (dev or prod)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Dev,
    Prod,
}

impl Profile {
    // Profile is now determined solely by the --dev CLI flag
    // No auto-detection is performed
}


/// Get the configuration directory path for TNJ
/// If profile is Dev, uses "tnj-dev" instead of "tnj"
pub fn get_config_dir(profile: Profile) -> Option<PathBuf> {
    let app_name = match profile {
        Profile::Dev => "tnj-dev",
        Profile::Prod => "tnj",
    };
    // Use "com" as qualifier for better cross-platform compatibility
    // On macOS, this will use ~/Library/Application Support/tnj/ or ~/Library/Preferences/tnj/
    ProjectDirs::from("com", "tnj", app_name)
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Get the data directory path for TNJ
/// If profile is Dev, uses "tnj-dev" instead of "tnj"
pub fn get_data_dir(profile: Profile) -> Option<PathBuf> {
    let app_name = match profile {
        Profile::Dev => "tnj-dev",
        Profile::Prod => "tnj",
    };
    // Use "com" as qualifier for better cross-platform compatibility
    // On macOS, this will use ~/Library/Application Support/tnj/ or ~/Library/Preferences/tnj/
    ProjectDirs::from("com", "tnj", app_name)
        .map(|dirs| dirs.data_dir().to_path_buf())
}

/// Expand `~` in a path string to the user's home directory
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Parse a date string in ISO 8601 format (YYYY-MM-DD)
pub fn parse_date(date_str: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

/// Get the current date as an ISO 8601 string (YYYY-MM-DD)
pub fn get_current_date_string() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// Parsed key binding information
#[derive(Debug, Clone)]
pub struct ParsedKeyBinding {
    pub key_code: crossterm::event::KeyCode,
    pub requires_ctrl: bool,
}

/// Check if a key event has the primary modifier (Ctrl on Windows/Linux, Option/Alt on macOS)
/// This follows the standard cross-platform TUI pattern where Ctrl and Option/Alt are treated as equivalent
pub fn has_primary_modifier(modifiers: crossterm::event::KeyModifiers) -> bool {
    #[cfg(target_os = "macos")]
    {
        modifiers.contains(crossterm::event::KeyModifiers::CONTROL) 
            || modifiers.contains(crossterm::event::KeyModifiers::ALT)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
    }
}

/// Format a key binding string for display, showing the platform-appropriate modifier
/// On macOS, "Ctrl+" is replaced with "Opt+" for better UX (Option key)
/// On other platforms, the string is returned as-is
pub fn format_key_binding_for_display(key_binding: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        key_binding.replace("Ctrl+", "Opt+")
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        key_binding.to_string()
    }
}

/// Parse a key binding string from config into a ParsedKeyBinding
/// Supports: single keys ("q", "n", "j", "k"), special keys ("Enter", "Left", "Right"), 
/// and modifiers ("Ctrl+b")
pub fn parse_key_binding(key_str: &str) -> Result<ParsedKeyBinding, String> {
    let key_str = key_str.trim();
    
    // Handle modifier keys (Ctrl+)
    if key_str.starts_with("Ctrl+") {
        let key_part = key_str.strip_prefix("Ctrl+")
            .expect("strip_prefix should succeed after starts_with check");
        let key_code = parse_key_code(key_part)?;
        return Ok(ParsedKeyBinding {
            key_code,
            requires_ctrl: true,
        });
    }
    
    // Handle regular keys (no modifiers)
    let key_code = parse_key_code(key_str)?;
    Ok(ParsedKeyBinding {
        key_code,
        requires_ctrl: false,
    })
}

/// Parse a key code from a string (without modifiers)
fn parse_key_code(key_str: &str) -> Result<crossterm::event::KeyCode, String> {
    // Handle special keys
    match key_str {
        "Enter" => Ok(crossterm::event::KeyCode::Enter),
        "Esc" | "Escape" => Ok(crossterm::event::KeyCode::Esc),
        "Backspace" => Ok(crossterm::event::KeyCode::Backspace),
        "Tab" => Ok(crossterm::event::KeyCode::Tab),
        "Space" | " " => Ok(crossterm::event::KeyCode::Char(' ')),
        "Left" => Ok(crossterm::event::KeyCode::Left),
        "Right" => Ok(crossterm::event::KeyCode::Right),
        "Up" => Ok(crossterm::event::KeyCode::Up),
        "Down" => Ok(crossterm::event::KeyCode::Down),
        "Home" => Ok(crossterm::event::KeyCode::Home),
        "End" => Ok(crossterm::event::KeyCode::End),
        "PageUp" => Ok(crossterm::event::KeyCode::PageUp),
        "PageDown" => Ok(crossterm::event::KeyCode::PageDown),
        "Delete" => Ok(crossterm::event::KeyCode::Delete),
        "Insert" => Ok(crossterm::event::KeyCode::Insert),
        "F1" => Ok(crossterm::event::KeyCode::F(1)),
        "F2" => Ok(crossterm::event::KeyCode::F(2)),
        "F3" => Ok(crossterm::event::KeyCode::F(3)),
        "F4" => Ok(crossterm::event::KeyCode::F(4)),
        "F5" => Ok(crossterm::event::KeyCode::F(5)),
        "F6" => Ok(crossterm::event::KeyCode::F(6)),
        "F7" => Ok(crossterm::event::KeyCode::F(7)),
        "F8" => Ok(crossterm::event::KeyCode::F(8)),
        "F9" => Ok(crossterm::event::KeyCode::F(9)),
        "F10" => Ok(crossterm::event::KeyCode::F(10)),
        "F11" => Ok(crossterm::event::KeyCode::F(11)),
        "F12" => Ok(crossterm::event::KeyCode::F(12)),
        _ => {
            // Try to parse as a single character
            if key_str.len() == 1 {
                match key_str.chars().next() {
                    Some(c) => Ok(crossterm::event::KeyCode::Char(c)),
                    None => Err(format!("Empty key string after length check (this should not happen)")),
                }
            } else {
                Err(format!("Unknown key binding: {}", key_str))
            }
        }
    }
}

