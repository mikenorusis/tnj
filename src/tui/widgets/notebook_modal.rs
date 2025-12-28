use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem, Clear};
use ratatui::style::{Style, Modifier};
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment, Constraint, Layout, Direction, Flex};
use ratatui::text::{Line, Span};
use crate::tui::App;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};
use crate::tui::app::{NotebookModalMode, NotebookModalField};

/// Calculate popup area (centered, with specified width and height percentages)
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Render notebook modal as a popup overlay
pub fn render_notebook_modal(f: &mut Frame, area: Rect, app: &App) {
    let active_theme = app.config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = get_contrast_text_color(highlight_bg);
    
    // Calculate popup area (70% width, 60% height, centered)
    let popup_area = popup_area(area, 70, 60);
    
    // Clear the background first
    f.render_widget(Clear, popup_area);
    
    // Render outer "Notebooks" box
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Notebooks")
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
    
    if let Some(ref state) = app.notebooks.modal_state {
        // Split into notebook list (left) and actions (right)
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner_area);
        
        let list_area = horizontal[0];
        let actions_area = horizontal[1];
        
        // Build notebook list with "[None]" first
        let mut notebook_items: Vec<ListItem> = vec![];
        notebook_items.push(ListItem::new("[None]"));
        for notebook in &app.notebooks.notebooks {
            notebook_items.push(ListItem::new(notebook.name.clone()));
        }
        
        // Highlight current selection
        let list = List::new(notebook_items)
            .block(Block::default().borders(Borders::ALL).title("Notebooks"))
            .style(Style::default().fg(fg_color).bg(bg_color))
            .highlight_style(
                Style::default()
                    .fg(highlight_fg)
                    .bg(highlight_bg)
                    .add_modifier(Modifier::BOLD)
            );
        
        let mut list_state = state.list_state.clone();
        f.render_stateful_widget(list, list_area, &mut list_state);
        
        // Render actions panel
        let mut action_lines: Vec<Line> = vec![];
        
        // Add action
        let add_style = if matches!(state.current_field, NotebookModalField::Add) {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Add", add_style),
        ]));
        
        // Rename action
        let rename_style = if matches!(state.current_field, NotebookModalField::Rename) {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Rename", rename_style),
        ]));
        
        // Delete action
        let delete_style = if matches!(state.current_field, NotebookModalField::Delete) {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Delete", delete_style),
        ]));
        
        // Switch action
        let switch_style = if matches!(state.current_field, NotebookModalField::Switch) {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Switch", switch_style),
        ]));
        
        // Render name editor if in Add or Rename mode
        if matches!(state.mode, NotebookModalMode::Add | NotebookModalMode::Rename) {
            action_lines.push(Line::from(""));
            let name_text = if state.name_editor.lines.is_empty() {
                "".to_string()
            } else {
                state.name_editor.lines[0].clone()
            };
            action_lines.push(Line::from(vec![
                Span::styled("Name: ", Style::default().fg(fg_color)),
                Span::styled(name_text, Style::default().fg(highlight_fg).bg(highlight_bg)),
            ]));
        }
        
        let actions_paragraph = Paragraph::new(action_lines)
            .block(Block::default().borders(Borders::ALL).title("Actions"))
            .style(Style::default().fg(fg_color).bg(bg_color));
        f.render_widget(actions_paragraph, actions_area);
    }
}

