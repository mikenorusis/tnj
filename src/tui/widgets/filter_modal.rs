use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem, Clear};
use ratatui::style::{Style, Modifier};
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment, Constraint, Layout, Direction, Flex};
use ratatui::text::{Line, Span};
use crate::tui::App;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};
use crate::tui::app::FilterFormField;

/// Render filter modal as a popup overlay
pub fn render_filter_modal(f: &mut Frame, area: Rect, app: &App) {
    let active_theme = app.config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = get_contrast_text_color(highlight_bg);
    
    // Calculate popup area (70% width, 60% height, centered)
    let popup_area = popup_area(area, 70, 60);
    
    // Clear the background first
    f.render_widget(Clear, popup_area);
    
    // Render outer "Filters" box
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Filters")
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
    
    if let Some(ref state) = app.filter_mode_state {
        // Split inner area vertically: fields, buttons
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1), // Fields area
                Constraint::Length(6), // Buttons area (borders + 3 buttons)
            ])
            .split(inner_area);
        
        let fields_area = vertical[0];
        let buttons_area = vertical[1];
        
        // Render fields
        render_filter_fields(f, fields_area, app, state, fg_color, bg_color, highlight_fg, highlight_bg);
        
        // Render buttons
        render_filter_buttons(f, buttons_area, app, state, fg_color, bg_color, highlight_fg, highlight_bg);
    }
}

fn render_filter_fields(
    f: &mut Frame,
    area: Rect,
    app: &App,
    state: &crate::tui::app::FilterFormState,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    // Build constraints based on whether we're on Tasks tab (show Status) or not
    let is_tasks_tab = app.current_tab == crate::tui::app::Tab::Tasks;
    let mut constraints = vec![
        Constraint::Length(5), // Tags field
        Constraint::Length(5), // Archived selector
    ];
    
    if is_tasks_tab {
        constraints.push(Constraint::Length(5)); // Status selector (only for Tasks)
    }
    
    constraints.extend(vec![
        Constraint::Length(5), // Tag logic selector
        Constraint::Min(0), // Remaining space
    ]);
    
    // Split into sections: Tags field, Archived selector, (Status selector if Tasks), Tag logic selector
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    // Render Tags field
    let tags_area = vertical[0];
    let tags_label = if matches!(state.current_field, FilterFormField::Tags) {
        "> Tags:"
    } else {
        "  Tags:"
    };
    let tags_block = Block::default()
        .borders(Borders::ALL)
        .title(tags_label)
        .style(if matches!(state.current_field, FilterFormField::Tags) {
            Style::default().fg(highlight_fg).bg(highlight_bg)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        });
    
    let tags_content = if state.tags.lines.is_empty() {
        String::new()
    } else {
        state.tags.lines[0].clone()
    };
    let tags_paragraph = Paragraph::new(tags_content)
        .block(tags_block)
        .style(Style::default().fg(fg_color));
    f.render_widget(tags_paragraph, tags_area);
    
    // Render cursor if tags field is active
    if matches!(state.current_field, FilterFormField::Tags) {
        // Calculate cursor position for single-line field
        let line_len = if state.tags.lines.is_empty() {
            0
        } else {
            state.tags.lines[0].chars().count()
        };
        let cursor_col = state.tags.cursor_col.min(line_len);
        let max_col = (tags_area.width.saturating_sub(2)) as usize;
        let x = tags_area.x + 1 + (cursor_col.min(max_col) as u16);
        let y = tags_area.y + 1;
        
        // Only set cursor if it's within the visible area
        if x < tags_area.x + tags_area.width && y < tags_area.y + tags_area.height {
            f.set_cursor_position((x, y));
        }
    }
    
    // Render Archived selector
    let archived_area = vertical[1];
    let archived_options = vec!["Active", "Archived", "All"];
    let archived_label = if matches!(state.current_field, FilterFormField::Archived) {
        "> Archived Status:"
    } else {
        "  Archived Status:"
    };
    
    let items: Vec<ListItem> = archived_options.iter().enumerate().map(|(idx, opt)| {
        let is_selected = idx == state.archived_index;
        let radio = if is_selected { "●" } else { "○" };
        let text = format!("{} {}", radio, opt);
        ListItem::new(text)
    }).collect();
    
    let archived_list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(archived_label)
            .style(if matches!(state.current_field, FilterFormField::Archived) {
                Style::default().fg(highlight_fg).bg(highlight_bg)
            } else {
                Style::default().fg(fg_color).bg(bg_color)
            }))
        .highlight_style(Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD));
    
    let mut archived_list_state = ratatui::widgets::ListState::default();
    archived_list_state.select(Some(state.archived_index));
    f.render_stateful_widget(archived_list, archived_area, &mut archived_list_state);
    
    // Render Status selector (only for Tasks tab)
    let mut logic_area_index = 2;
    if is_tasks_tab {
        let status_area = vertical[2];
        let status_options = vec!["Todo", "Done", "All"];
        let status_label = if matches!(state.current_field, FilterFormField::Status) {
            "> Task Status:"
        } else {
            "  Task Status:"
        };
        
        let status_items: Vec<ListItem> = status_options.iter().enumerate().map(|(idx, opt)| {
            let is_selected = idx == state.status_index;
            let radio = if is_selected { "●" } else { "○" };
            let text = format!("{} {}", radio, opt);
            ListItem::new(text)
        }).collect();
        
        let status_list = List::new(status_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(status_label)
                .style(if matches!(state.current_field, FilterFormField::Status) {
                    Style::default().fg(highlight_fg).bg(highlight_bg)
                } else {
                    Style::default().fg(fg_color).bg(bg_color)
                }))
            .highlight_style(Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD));
        
        let mut status_list_state = ratatui::widgets::ListState::default();
        status_list_state.select(Some(state.status_index));
        f.render_stateful_widget(status_list, status_area, &mut status_list_state);
        
        logic_area_index = 3;
    }
    
    // Render Tag Logic selector
    let logic_area = vertical[logic_area_index];
    let logic_options = vec!["AND", "OR"];
    let logic_label = if matches!(state.current_field, FilterFormField::TagLogic) {
        "> Tag Logic:"
    } else {
        "  Tag Logic:"
    };
    
    let logic_items: Vec<ListItem> = logic_options.iter().enumerate().map(|(idx, opt)| {
        let is_selected = idx == state.tag_logic_index;
        let radio = if is_selected { "●" } else { "○" };
        let text = format!("{} {}", radio, opt);
        ListItem::new(text)
    }).collect();
    
    let logic_list = List::new(logic_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(logic_label)
            .style(if matches!(state.current_field, FilterFormField::TagLogic) {
                Style::default().fg(highlight_fg).bg(highlight_bg)
            } else {
                Style::default().fg(fg_color).bg(bg_color)
            }))
        .highlight_style(Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD));
    
    let mut logic_list_state = ratatui::widgets::ListState::default();
    logic_list_state.select(Some(state.tag_logic_index));
    f.render_stateful_widget(logic_list, logic_area, &mut logic_list_state);
}

fn render_filter_buttons(
    f: &mut Frame,
    area: Rect,
    _app: &App,
    state: &crate::tui::app::FilterFormState,
    fg_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
    highlight_fg: ratatui::style::Color,
    highlight_bg: ratatui::style::Color,
) {
    let buttons = vec!["Apply", "Clear", "Cancel"];
    let mut lines = Vec::new();
    
    // Determine which button is selected
    let selected_index = match state.current_field {
        FilterFormField::Apply => Some(0),
        FilterFormField::Clear => Some(1),
        FilterFormField::Cancel => Some(2),
        _ => None,
    };
    
    // Build button lines with selection highlighting
    for (index, button_text) in buttons.iter().enumerate() {
        let is_selected = selected_index == Some(index);
        let prefix = if is_selected { "> " } else { "  " };
        let text = format!("{}{}", prefix, button_text);
        
        let style = if is_selected {
            Style::default().fg(highlight_fg).bg(highlight_bg)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        
        lines.push(Line::from(Span::styled(text, style)));
    }
    
    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(if selected_index.is_some() { "> Actions" } else { "  Actions" })
            .style(if selected_index.is_some() {
                Style::default().fg(highlight_fg).bg(highlight_bg)
            } else {
                Style::default().fg(fg_color).bg(bg_color)
            }))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .alignment(Alignment::Center);
    
    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

