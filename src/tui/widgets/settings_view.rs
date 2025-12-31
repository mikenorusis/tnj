use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget, Clear, Paragraph};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Layout, Direction, Constraint, Alignment, Flex};
use ratatui::widgets::ListState;
use ratatui::text::{Line, Span};
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
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    
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
    let mut categories_list_state = app.settings.list_state.clone();
    categories_list_state.select(Some(app.settings.category_index));
    let is_category_list_active = app.settings.current_field == crate::tui::app::SettingsField::CategoryList;
    render_settings_list(f, sidebar_area, &categories, &mut categories_list_state, &app.config, is_category_list_active);
    
    // Render main content based on selected category
    let selected_category = categories.get(app.settings.category_index);
    match selected_category {
        Some(category) if category == "Theme Settings" => {
            render_theme_settings(f, main_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
        }
        Some(category) if category == "Appearance Settings" => {
            render_appearance_settings(f, main_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
        }
        Some(category) if category == "Display Settings" => {
            render_display_settings(f, main_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
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
    
    // Calculate height needed for color editor (5 fields + preview + actions + borders)
    // Ensure we don't exceed available space after theme box
    let available_height = main_area.height.saturating_sub(theme_box_height);
    let color_editor_height = (5 + 4 + 2 + 2).min(available_height as usize).max(0) as u16;
    
    // Split main area vertically - Theme box and color editor
    let theme_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(theme_box_height), // Theme box
            Constraint::Length(color_editor_height), // Color editor
            Constraint::Min(0), // Remaining space
        ])
        .split(main_area);
    
    let theme_area = theme_areas[0];
    let color_editor_area = theme_areas[1];
    
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
    
    let mut list_state = app.settings.theme_list_state.clone();
    list_state.select(Some(app.settings.theme_index));
    StatefulWidget::render(list, theme_area, f.buffer_mut(), &mut list_state);
    
    // Render color editor
    render_color_editor(f, color_editor_area, app, fg_color, bg_color, highlight_fg, highlight_bg);
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
    list_state.select(Some(app.settings.sidebar_width_index));
    StatefulWidget::render(list, width_area, f.buffer_mut(), &mut list_state);
}

/// Render display settings content
fn render_display_settings(
    f: &mut Frame,
    main_area: Rect,
    app: &App,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    use crate::tui::app::ListViewMode;
    
    let mode_options = vec!["Simple", "TwoLine", "GroupedByTags"];
    let current_mode_str = match app.ui.list_view_mode {
        ListViewMode::Simple => "Simple",
        ListViewMode::TwoLine => "TwoLine",
        ListViewMode::GroupedByTags => "GroupedByTags",
    };
    
    // Calculate height needed for Display Mode box
    let mode_box_height = (mode_options.len() + 2).max(5).min(main_area.height as usize) as u16;
    
    // Split main area vertically
    let mode_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(mode_box_height), // Display mode box
            Constraint::Min(0), // Remaining space
        ])
        .split(main_area);
    
    let mode_area = mode_areas[0];
    
    // Create list items with radio button style
    let items: Vec<ListItem> = mode_options.iter().map(|&mode| {
        let is_selected = mode == current_mode_str;
        let radio = if is_selected { "●" } else { "○" };
        let text = format!("{} {}", radio, mode);
        ListItem::new(text)
    }).collect();
    
    // Render Display Mode sub-box with list
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Display Mode"))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
        );
    
    // Create a temporary list state for rendering
    let mut list_state = ListState::default();
    list_state.select(Some(app.settings.display_mode_index));
    StatefulWidget::render(list, mode_area, f.buffer_mut(), &mut list_state);
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

/// Render color editor
fn render_color_editor(
    f: &mut Frame,
    area: Rect,
    app: &App,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    // Wrap color editor in a bordered box with title
    let color_editor_block = Block::default()
        .borders(Borders::ALL)
        .title("Color Options");
    
    // Calculate inner area (accounting for borders)
    let inner_area = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    
    // Render the block border
    f.render_widget(color_editor_block, area);

    // Split inner area: color fields (left) and preview + actions (right)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30), // Color fields
            Constraint::Length(20), // Preview + actions
        ])
        .split(inner_area);

    let fields_area = horizontal[0];
    let preview_area = horizontal[1];

    // Render color fields
    let fields_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Foreground
            Constraint::Length(1), // Background
            Constraint::Length(1), // Highlight
            Constraint::Length(1), // Highlight FG
            Constraint::Length(1), // Tab BG
            Constraint::Min(0), // Error message space
        ])
        .split(fields_area);

    for (idx, field_area) in fields_layout.iter().take(5).enumerate() {
        render_color_field(f, *field_area, app, idx, fg_color, bg_color, highlight_fg, highlight_bg);
    }

    // Render error message if present
    if let Some(ref error) = app.settings.color_input_error {
        if fields_layout.len() > 5 {
            let error_area = fields_layout[5];
            let error_para = Paragraph::new(error.as_str())
                .style(Style::default().fg(ratatui::style::Color::Red));
            f.render_widget(error_para, error_area);
        }
    }

    // Render preview and actions
    let preview_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Preview
            Constraint::Min(0), // Actions
        ])
        .split(preview_area);

    render_color_preview(f, preview_layout[0], app, fg_color, bg_color, highlight_fg, highlight_bg);
    
    // Render save theme name input if active
    if app.settings.color_save_theme_name_editor.is_some() {
        // Show input field for theme name
        let name_text = if let Some(ref editor) = app.settings.color_save_theme_name_editor {
            if editor.lines.is_empty() {
                "".to_string()
            } else {
                editor.lines[0].clone()
            }
        } else {
            "".to_string()
        };
        
        let name_line = Line::from(vec![
            Span::styled("Theme name: ", Style::default().fg(fg_color)),
            Span::styled(name_text.as_str(), Style::default().fg(highlight_fg).bg(highlight_bg)),
        ]);
        
        let name_para = Paragraph::new(name_line);
        f.render_widget(name_para, preview_layout[1]);
        
        // Render cursor
        if let Some(ref editor) = app.settings.color_save_theme_name_editor {
            let prefix = "Theme name: ";
            let cursor_col = editor.cursor_col.min(name_text.chars().count());
            let prefix_len = prefix.chars().count();
            let total_cursor_col = prefix_len + cursor_col;
            let max_col = (preview_layout[1].width.saturating_sub(1)) as usize;
            let visible_cursor_col = total_cursor_col.min(max_col);
            
            let x = preview_layout[1].x + (visible_cursor_col as u16);
            let y = preview_layout[1].y;
            
            if x < preview_layout[1].x + preview_layout[1].width && y < preview_layout[1].y + preview_layout[1].height {
                f.set_cursor_position((x, y));
            }
        }
    } else {
        render_color_actions(f, preview_layout[1], app, fg_color, bg_color, highlight_fg, highlight_bg);
    }
}

/// Render a single color field
fn render_color_field(
    f: &mut Frame,
    area: Rect,
    app: &App,
    field_index: usize,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    let theme = app.get_color_preview_theme();
    let field_name = app.get_color_field_name(field_index);
    let is_active = app.settings.color_field_index == field_index;
    let is_input_mode = app.settings.color_input_mode && 
                        app.settings.color_input_field_index == Some(field_index);
    
    let color_value = match field_index {
        0 => &theme.fg,
        1 => &theme.bg,
        2 => &theme.highlight_bg,
        3 => {
            // For highlight_fg, use the value or calculate if empty
            if theme.highlight_fg.is_empty() {
                // Calculate from highlight_bg
                use crate::tui::widgets::color::{parse_color, get_contrast_text_color, format_color_for_display};
                let highlight_bg_color = parse_color(&theme.highlight_bg);
                let calculated = get_contrast_text_color(highlight_bg_color);
                return render_color_field_value(f, area, field_name, &format_color_for_display(&calculated), 
                    is_active, is_input_mode, fg_color, bg_color, highlight_fg, highlight_bg);
            } else {
                &theme.highlight_fg
            }
        },
        4 => &theme.tab_bg,
        _ => return,
    };
    
    render_color_field_value(f, area, field_name, color_value, is_active, is_input_mode, 
        fg_color, bg_color, highlight_fg, highlight_bg);
}

/// Helper function to render a color field value
fn render_color_field_value(
    f: &mut Frame,
    area: Rect,
    field_name: &str,
    color_value: &str,
    is_active: bool,
    is_input_mode: bool,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    let color = parse_color(color_value);

    // Build display text
    let mut display_value = format!("[{}]", color_value);
    if is_input_mode {
        display_value.push_str(" <input>");
    }

    // Create text with color swatch
    let swatch = "█";
    let text = if is_active {
        Line::from(vec![
            Span::styled(
                format!("{}: {} {}", field_name, display_value, swatch),
                Style::default()
                    .fg(highlight_fg)
                    .bg(highlight_bg)
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!("{}: {} ", field_name, display_value),
                Style::default().fg(fg_color).bg(bg_color)
            ),
            Span::styled(
                swatch,
                Style::default().fg(color).bg(bg_color)
            ),
        ])
    };

    let para = Paragraph::new(text);
    f.render_widget(para, area);
}

/// Render color preview box
fn render_color_preview(
    f: &mut Frame,
    area: Rect,
    app: &App,
    _fg_color: ratatui::style::Color,
    _bg_color: ratatui::style::Color,
    _highlight_fg: ratatui::style::Color,
    _highlight_bg: ratatui::style::Color,
) {
    let theme = app.get_color_preview_theme();
    let fg = parse_color(&theme.fg);
    let bg = parse_color(&theme.bg);
    let highlight = parse_color(&theme.highlight_bg);
    let tab_bg = parse_color(&theme.tab_bg);

    // Get highlight_fg from theme or calculate
    let highlight_fg_color = if theme.highlight_fg.is_empty() {
        use crate::tui::widgets::color::{get_contrast_text_color};
        get_contrast_text_color(highlight)
    } else {
        parse_color(&theme.highlight_fg)
    };
    
    let preview_text = vec![
        Line::from(Span::styled("Sample Text", Style::default().fg(fg).bg(bg))),
        Line::from(Span::styled("Highlighted", Style::default().fg(highlight_fg_color).bg(highlight))),
        Line::from(Span::styled("Tab", Style::default().fg(ratatui::style::Color::White).bg(tab_bg))),
    ];

    let para = Paragraph::new(preview_text)
        .block(Block::default().borders(Borders::ALL).title("Preview"));
    f.render_widget(para, area);
}

/// Render color action buttons
fn render_color_actions(
    f: &mut Frame,
    area: Rect,
    _app: &App,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    _highlight_fg: ratatui::style::Color,
    _highlight_bg: ratatui::style::Color,
) {
    let actions = vec![
        Line::from(Span::styled(
            "[Reset to Theme]",
            Style::default().fg(fg_color).bg(bg_color)
        )),
        Line::from(Span::styled(
            "[Save as Theme]",
            Style::default().fg(fg_color).bg(bg_color)
        )),
    ];

    let para = Paragraph::new(actions);
    f.render_widget(para, area);
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

