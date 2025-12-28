use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget, Clear};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Layout, Direction, Constraint, Alignment, Flex};
use ratatui::widgets::ListState;
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};
use crate::tui::App;
use crate::tui::widgets::settings_list::render_settings_list;

pub fn render_settings_view(
    f: &mut Frame,
    area: Rect,
    themes: &[String],
    current_theme: &str,
    _selected_index: usize,
    list_state: &mut ListState,
    config: &Config,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = get_contrast_text_color(highlight_bg);

    // Render outer "Settings" box
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Settings")
        .style(Style::default().fg(fg_color).bg(bg_color));
    f.render_widget(outer_block, area);

    // Calculate inner area (accounting for borders)
    let inner_area = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );

    // Calculate height needed for Theme box
    // Each theme item takes 1 line, plus 2 for borders (top and bottom)
    // Add a bit of padding - minimum 5 lines, or themes count + 2 for borders
    let theme_box_height = (themes.len() + 2).max(5).min(inner_area.height as usize) as u16;
    
    // Split inner area vertically - Theme box takes only what it needs
    let theme_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(theme_box_height), // Theme box
            Constraint::Min(0), // Remaining space
        ])
        .split(inner_area);
    
    let theme_area = theme_areas[0];

    // Create list items with radio button style
    let items: Vec<ListItem> = themes.iter().map(|theme_name| {
        let is_selected = theme_name == current_theme;
        
        // Radio button: ● for selected, ○ for unselected
        let radio = if is_selected { "●" } else { "○" };
        let text = format!("{} {}", radio, theme_name);
        
        ListItem::new(text)
    }).collect();

    // Render Theme sub-box with list
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Theme"))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
        );

    StatefulWidget::render(list, theme_area, f.buffer_mut(), list_state);
}

/// Render settings as a modal popup overlay (similar to help)
pub fn render_settings_view_modal(f: &mut Frame, area: Rect, app: &App) {
    let active_theme = app.config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = get_contrast_text_color(highlight_bg);
    
    // Calculate popup area (70% width, 60% height, centered)
    let popup_area = popup_area(area, 70, 60);
    
    // Clear the background first - this prevents content from showing through
    f.render_widget(Clear, popup_area);
    
    // Render outer "Settings" box
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Settings")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(fg_color).bg(bg_color));
    f.render_widget(outer_block, popup_area);
    
    // Calculate inner area (accounting for borders)
    let inner_area = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width.saturating_sub(2),
        popup_area.height.saturating_sub(2),
    );
    
    // Split inner area horizontally: sidebar (categories) and main (theme selector)
    // Sidebar takes about 30% of width, minimum 20 chars
    let sidebar_width = (inner_area.width * 30 / 100).max(20).min(inner_area.width.saturating_sub(10));
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(sidebar_width), // Categories sidebar
            Constraint::Min(1), // Main area (theme selector)
        ])
        .split(inner_area);
    
    let sidebar_area = horizontal[0];
    let main_area = horizontal[1];
    
    // Render categories list in sidebar
    let categories = app.get_settings_categories();
    let mut categories_list_state = app.settings_list_state.clone();
    categories_list_state.select(Some(app.settings_category_index));
    render_settings_list(f, sidebar_area, &categories, &mut categories_list_state, &app.config);
    
    // Render main content based on selected category
    let selected_category = categories.get(app.settings_category_index);
    match selected_category {
        Some(category) if category == "Theme Settings" => {
            render_theme_settings(f, main_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
        }
        Some(category) if category == "Appearance Settings" => {
            render_appearance_settings(f, main_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
        }
        Some(category) if category == "System Settings" => {
            render_system_settings(f, main_area, app, fg_color, bg_color);
        }
        _ => {
            // Default to theme settings if category not recognized
            render_theme_settings(f, main_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
        }
    }
}

/// Render theme settings content
fn render_theme_settings(
    f: &mut Frame,
    main_area: Rect,
    app: &App,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    // Get themes
    let themes = app.get_available_themes();
    let current_theme = &app.config.current_theme;
    
    // Calculate height needed for Theme box
    let theme_box_height = (themes.len() + 2).max(5).min(main_area.height as usize) as u16;
    
    // Split main area vertically - Theme box takes only what it needs
    let theme_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(theme_box_height), // Theme box
            Constraint::Min(0), // Remaining space
        ])
        .split(main_area);
    
    let theme_area = theme_areas[0];
    
    // Create list items with radio button style
    let items: Vec<ListItem> = themes.iter().map(|theme_name| {
        let is_selected = theme_name == current_theme;
        let radio = if is_selected { "●" } else { "○" };
        let text = format!("{} {}", radio, theme_name);
        ListItem::new(text)
    }).collect();
    
    // Render Theme sub-box with list
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Theme"))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
        );
    
    let mut list_state = app.settings_theme_list_state.clone();
    list_state.select(Some(app.settings_theme_index));
    StatefulWidget::render(list, theme_area, f.buffer_mut(), &mut list_state);
}

/// Render appearance settings content
fn render_appearance_settings(
    f: &mut Frame,
    main_area: Rect,
    app: &App,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    let width_options = app.get_sidebar_width_options();
    let current_width = app.config.sidebar_width_percent;
    
    // Calculate height needed for Sidebar Width box
    let width_box_height = (width_options.len() + 2).max(5).min(main_area.height as usize) as u16;
    
    // Split main area vertically
    let width_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(width_box_height), // Sidebar width box
            Constraint::Min(0), // Remaining space
        ])
        .split(main_area);
    
    let width_area = width_areas[0];
    
    // Create list items with radio button style
    let items: Vec<ListItem> = width_options.iter().map(|&width| {
        let is_selected = width == current_width;
        let radio = if is_selected { "●" } else { "○" };
        let text = format!("{} {}%", radio, width);
        ListItem::new(text)
    }).collect();
    
    // Render Sidebar Width sub-box with list
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Sidebar Width"))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
        );
    
    // Create a temporary list state for rendering
    let mut list_state = ListState::default();
    list_state.select(Some(app.settings_sidebar_width_index));
    StatefulWidget::render(list, width_area, f.buffer_mut(), &mut list_state);
}

/// Render system settings content
fn render_system_settings(
    f: &mut Frame,
    main_area: Rect,
    app: &App,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
) {
    use ratatui::widgets::Paragraph;
    
    let config_path = app.get_config_file_path();
    let db_path = app.get_database_file_path();
    
    // Create text content showing the paths
    let content = format!(
        "Config File:\n{}\n\nDatabase File:\n{}",
        config_path,
        db_path
    );
    
    // Render as a paragraph with word wrapping
    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("File Locations"))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(paragraph, main_area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
/// Based on ratatui popup example: https://ratatui.rs/examples/apps/popup/
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

