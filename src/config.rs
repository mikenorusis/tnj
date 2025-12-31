use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::utils;

/// Current configuration version
pub const CURRENT_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width_percent: u16,
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default)]
    pub key_bindings: KeyBindings,
    #[serde(default = "default_current_theme")]
    pub current_theme: String,
    #[serde(default)]
    pub themes: HashMap<String, Theme>,
    #[serde(default = "default_list_view_mode")]
    pub list_view_mode: String,
    #[serde(default = "default_config_version")]
    pub config_version: Option<u32>,
    #[serde(default)]
    pub color_overrides: Option<Theme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_quit")]
    pub quit: String,
    #[serde(default = "default_toggle_sidebar")]
    pub toggle_sidebar: String,
    #[serde(default = "default_new")]
    pub new: String,
    #[serde(default = "default_edit")]
    pub edit: String,
    #[serde(default = "default_save")]
    pub save: String,
    #[serde(default = "default_delete")]
    pub delete: String,
    #[serde(default = "default_search")]
    pub search: String,
    #[serde(default = "default_select")]
    pub select: String,
    #[serde(default = "default_list_up")]
    pub list_up: String,
    #[serde(default = "default_list_down")]
    pub list_down: String,
    #[serde(default = "default_tab_left")]
    pub tab_left: String,
    #[serde(default = "default_tab_right")]
    pub tab_right: String,
    #[serde(default = "default_tab_1")]
    pub tab_1: String,
    #[serde(default = "default_tab_2")]
    pub tab_2: String,
    #[serde(default = "default_tab_3")]
    pub tab_3: String,
    #[serde(default = "default_tab_4")]
    pub tab_4: String,
    #[serde(default = "default_help")]
    pub help: String,
    #[serde(default = "default_undo")]
    pub undo: String,
    #[serde(default = "default_word_left")]
    pub word_left: String,
    #[serde(default = "default_word_right")]
    pub word_right: String,
    #[serde(default = "default_settings")]
    pub settings: String,
    #[serde(default = "default_toggle_task_status")]
    pub toggle_task_status: String,
    #[serde(default = "default_toggle_list_view")]
    pub toggle_list_view: String,
    #[serde(default = "default_filter")]
    pub filter: String,
    #[serde(default = "default_notebook_modal")]
    pub notebook_modal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default = "default_fg")]
    pub fg: String,
    #[serde(default = "default_bg")]
    pub bg: String,
    #[serde(default = "default_highlight_bg")]
    pub highlight_bg: String,
    #[serde(default = "default_highlight_fg")]
    pub highlight_fg: String,
    #[serde(default = "default_tab_bg")]
    pub tab_bg: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut themes = HashMap::new();
        
        // Add example custom theme for users to see how to define themes
        themes.insert("lightblue".to_string(), Theme {
            fg: "cyan".to_string(),
            bg: "black".to_string(),
            highlight_bg: "blue".to_string(),
            highlight_fg: "white".to_string(),
            tab_bg: "gray".to_string(),
        });
        
        Self {
            sidebar_width_percent: default_sidebar_width(),
            database_path: default_database_path(),
            key_bindings: KeyBindings::default(),
            current_theme: default_current_theme(),
            themes,
            list_view_mode: default_list_view_mode(),
            config_version: Some(CURRENT_CONFIG_VERSION),
            color_overrides: None,
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: default_quit(),
            toggle_sidebar: default_toggle_sidebar(),
            new: default_new(),
            edit: default_edit(),
            save: default_save(),
            delete: default_delete(),
            search: default_search(),
            select: default_select(),
            list_up: default_list_up(),
            list_down: default_list_down(),
            tab_left: default_tab_left(),
            tab_right: default_tab_right(),
            tab_1: default_tab_1(),
            tab_2: default_tab_2(),
            tab_3: default_tab_3(),
            tab_4: default_tab_4(),
            help: default_help(),
            undo: default_undo(),
            word_left: default_word_left(),
            word_right: default_word_right(),
            settings: default_settings(),
            toggle_task_status: default_toggle_task_status(),
            toggle_list_view: default_toggle_list_view(),
            filter: default_filter(),
            notebook_modal: default_notebook_modal(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: default_fg(),
            bg: default_bg(),
            highlight_bg: default_highlight_bg(),
            highlight_fg: default_highlight_fg(),
            tab_bg: default_tab_bg(),
        }
    }
}

impl Theme {
    /// Get preset themes that are always available
    pub fn get_preset_themes() -> HashMap<String, Theme> {
        let mut themes = HashMap::new();
        
        themes.insert("default".to_string(), Theme {
            fg: "white".to_string(),
            bg: "black".to_string(),
            highlight_bg: "blue".to_string(),
            highlight_fg: "white".to_string(),
            tab_bg: "gray".to_string(),
        });
        
        themes.insert("dark".to_string(), Theme {
            fg: "white".to_string(),
            bg: "black".to_string(),
            highlight_bg: "cyan".to_string(),
            highlight_fg: "black".to_string(),
            tab_bg: "gray".to_string(),
        });
        
        themes.insert("light".to_string(), Theme {
            fg: "black".to_string(),
            bg: "white".to_string(),
            highlight_bg: "blue".to_string(),
            highlight_fg: "white".to_string(),
            tab_bg: "gray".to_string(),
        });
        
        themes.insert("green".to_string(), Theme {
            fg: "green".to_string(),
            bg: "black".to_string(),
            highlight_bg: "yellow".to_string(),
            highlight_fg: "black".to_string(),
            tab_bg: "gray".to_string(),
        });
        
        themes.insert("monochrome".to_string(), Theme {
            fg: "white".to_string(),
            bg: "black".to_string(),
            highlight_bg: "white".to_string(),
            highlight_fg: "black".to_string(),
            tab_bg: "gray".to_string(),
        });
        
        themes
    }
}

// Default value functions
fn default_sidebar_width() -> u16 {
    30
}

fn default_database_path() -> String {
    // This is a fallback - actual profile will be determined at load time
    if let Some(data_dir) = utils::get_data_dir(utils::Profile::Prod) {
        data_dir.join("app.db").to_string_lossy().to_string()
    } else {
        "~/.local/share/tnj/app.db".to_string()
    }
}

fn default_quit() -> String {
    "q".to_string()
}

fn default_toggle_sidebar() -> String {
    "b".to_string()
}

fn default_new() -> String {
    "n".to_string()
}

fn default_edit() -> String {
    "e".to_string()
}

fn default_save() -> String {
    "Ctrl+s".to_string()
}

fn default_delete() -> String {
    "d".to_string()
}

fn default_search() -> String {
    "/".to_string()
}

fn default_select() -> String {
    "Enter".to_string()
}

fn default_list_up() -> String {
    "k".to_string()
}

fn default_list_down() -> String {
    "j".to_string()
}

fn default_tab_left() -> String {
    "Left".to_string()
}

fn default_tab_right() -> String {
    "Right".to_string()
}

fn default_tab_1() -> String {
    "1".to_string()
}

fn default_tab_2() -> String {
    "2".to_string()
}

fn default_tab_3() -> String {
    "3".to_string()
}

fn default_tab_4() -> String {
    "4".to_string()
}

fn default_help() -> String {
    "F1".to_string()
}

fn default_current_theme() -> String {
    "default".to_string()
}

fn default_undo() -> String {
    "Ctrl+z".to_string()
}

fn default_word_left() -> String {
    "Ctrl+Left".to_string()
}

fn default_word_right() -> String {
    "Ctrl+Right".to_string()
}

fn default_settings() -> String {
    "F2".to_string()
}

fn default_toggle_task_status() -> String {
    "Space".to_string()
}

fn default_toggle_list_view() -> String {
    "t".to_string()
}

fn default_filter() -> String {
    "f".to_string()
}

fn default_notebook_modal() -> String {
    "Ctrl+n".to_string()
}

fn default_fg() -> String {
    "white".to_string()
}

fn default_bg() -> String {
    "black".to_string()
}

fn default_highlight_bg() -> String {
    "blue".to_string()
}

fn default_highlight_fg() -> String {
    "white".to_string()
}

fn default_tab_bg() -> String {
    "gray".to_string()
}

fn default_list_view_mode() -> String {
    "Simple".to_string()
}

fn default_config_version() -> Option<u32> {
    Some(CURRENT_CONFIG_VERSION)
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config directory: {0}")]
    ConfigDirError(String),
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to write config file: {0}")]
    WriteError(String),
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    #[error("Theme name already exists: {0}")]
    ThemeNameExists(String),
}

impl Config {
    /// Load configuration from file, or create default if missing
    /// Uses the provided profile to determine config and database paths
    pub fn load_with_profile(profile: utils::Profile) -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path(profile)?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::ReadError(e.to_string()))?;
            let mut config: Config = toml::from_str(&contents)?;
            
            // Ensure database path matches profile (in case config was manually edited)
            config.database_path = Self::default_database_path_for_profile(profile);
            
            Ok(config)
        } else {
            // Create default config and save it
            let mut config = Config::default();
            config.database_path = Self::default_database_path_for_profile(profile);
            let save_result = config.save_with_profile(profile);
            if let Err(ref e) = save_result {
                eprintln!("ERROR: Failed to save config file: {}", e);
                eprintln!("Config path: {:?}", config_path);
            }
            save_result?;
            Ok(config)
        }
    }

    /// Load configuration from file, using production profile
    /// Use load_with_profile() to specify a different profile
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with_profile(utils::Profile::Prod)
    }

    /// Save configuration to file
    pub fn save_with_profile(&mut self, profile: utils::Profile) -> Result<(), ConfigError> {
        // Ensure config version is set before saving
        self.config_version = Some(CURRENT_CONFIG_VERSION);
        
        let config_path = Self::get_config_path(profile)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::WriteError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, toml_string)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// Save configuration to file, using production profile
    /// Use save_with_profile() to specify a different profile
    pub fn save(&mut self) -> Result<(), ConfigError> {
        self.save_with_profile(utils::Profile::Prod)
    }

    /// Get the path to the config file
    pub fn get_config_path(profile: utils::Profile) -> Result<PathBuf, ConfigError> {
        let config_dir = utils::get_config_dir(profile)
            .ok_or_else(|| ConfigError::ConfigDirError("Could not determine config directory".to_string()))?;
        Ok(config_dir.join("config.toml"))
    }

    /// Get default database path for a specific profile
    fn default_database_path_for_profile(profile: utils::Profile) -> String {
        if let Some(data_dir) = utils::get_data_dir(profile) {
            data_dir.join("app.db").to_string_lossy().to_string()
        } else {
            // Fallback paths - platform-specific
            #[cfg(target_os = "macos")]
            {
                match profile {
                    utils::Profile::Dev => "~/Library/Application Support/tnj-dev/app.db".to_string(),
                    utils::Profile::Prod => "~/Library/Application Support/tnj/app.db".to_string(),
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                match profile {
                    utils::Profile::Dev => "~/.local/share/tnj-dev/app.db".to_string(),
                    utils::Profile::Prod => "~/.local/share/tnj/app.db".to_string(),
                }
            }
        }
    }

    /// Get the expanded database path (with ~ expansion)
    pub fn get_database_path(&self) -> PathBuf {
        utils::expand_path(&self.database_path)
    }

    /// Get the currently active theme
    /// If highlight_fg is not set (empty string), it will be calculated from highlight_bg
    pub fn get_active_theme(&self) -> Theme {
        use crate::tui::widgets::color::{parse_color, get_contrast_text_color, format_color_for_display};
        
        // First check color overrides (user customizations)
        let mut theme = if let Some(ref overrides) = self.color_overrides {
            overrides.clone()
        } else if let Some(theme) = self.themes.get(&self.current_theme) {
            theme.clone()
        } else if let Some(theme) = Theme::get_preset_themes().get(&self.current_theme) {
            theme.clone()
        } else {
            // Final fallback: default theme
            Theme::get_preset_themes().get("default")
                .cloned()
                .unwrap_or_else(|| Theme::default())
        };
        
        // If highlight_fg is empty or not set, calculate it from highlight_bg
        if theme.highlight_fg.is_empty() {
            let highlight_bg_color = parse_color(&theme.highlight_bg);
            let calculated_fg = get_contrast_text_color(highlight_bg_color);
            theme.highlight_fg = format_color_for_display(&calculated_fg);
        }
        
        theme
    }

    /// Set the active theme by name
    pub fn set_theme(&mut self, name: &str) -> Result<(), ConfigError> {
        // Verify theme exists (preset or user-defined)
        if !self.themes.contains_key(name) && 
           !Theme::get_preset_themes().contains_key(name) {
            return Err(ConfigError::ThemeNotFound(name.to_string()));
        }
        
        self.current_theme = name.to_string();
        Ok(())
    }

    /// Get all available theme names (presets + user-defined)
    pub fn get_available_themes(&self) -> Vec<String> {
        let mut themes: Vec<String> = Theme::get_preset_themes().keys().cloned().collect();
        
        // Add user-defined themes that aren't already in presets
        for theme_name in self.themes.keys() {
            if !Theme::get_preset_themes().contains_key(theme_name) {
                themes.push(theme_name.clone());
            }
        }
        
        // Sort for consistent display
        themes.sort();
        themes
    }

    /// Clear color overrides, reverting to base theme
    pub fn clear_color_overrides(&mut self) {
        self.color_overrides = None;
    }

    /// Set color overrides
    pub fn set_color_overrides(&mut self, theme: Theme) {
        self.color_overrides = Some(theme);
    }

    /// Get current color overrides
    pub fn get_color_overrides(&self) -> Option<&Theme> {
        self.color_overrides.as_ref()
    }

    /// Save current color overrides as a new theme, or update existing custom theme
    pub fn save_theme_from_overrides(&mut self, name: &str) -> Result<(), ConfigError> {
        // Check if theme name is a preset theme (cannot overwrite presets)
        if Theme::get_preset_themes().contains_key(name) {
            return Err(ConfigError::ThemeNameExists(name.to_string()));
        }

        // Get current overrides or base theme
        let theme = self.get_active_theme();
        
        // Save or update user-defined theme (overwrites if it exists)
        self.themes.insert(name.to_string(), theme);
        
        Ok(())
    }
}

