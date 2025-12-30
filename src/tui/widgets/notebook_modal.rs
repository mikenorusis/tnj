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
        
        // Determine if actions list is active
        let is_actions_active = matches!(state.current_field, NotebookModalField::ActionsList);
        
        // Add action (index 0)
        let add_style = if is_actions_active && state.actions_selected_index == 0 {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Add", add_style),
        ]));
        
        // Rename action (index 1)
        let rename_style = if is_actions_active && state.actions_selected_index == 1 {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Rename", rename_style),
        ]));
        
        // Delete action (index 2)
        let delete_style = if is_actions_active && state.actions_selected_index == 2 {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Delete", delete_style),
        ]));
        
        // Switch action (index 3)
        let switch_style = if is_actions_active && state.actions_selected_index == 3 {
            Style::default().fg(highlight_fg).bg(highlight_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        action_lines.push(Line::from(vec![
            Span::styled("Switch", switch_style),
        ]));
        
        // Render name editor if in Add or Rename mode
        let name_editor_line_index = if matches!(state.mode, NotebookModalMode::Add | NotebookModalMode::Rename) {
            action_lines.push(Line::from(""));
            let name_text = if state.name_editor.lines.is_empty() {
                "".to_string()
            } else {
                state.name_editor.lines[0].clone()
            };
            // Get line index for the name editor line (after pushing it)
            action_lines.push(Line::from(vec![
                Span::styled("Name: ", Style::default().fg(fg_color)),
                Span::styled(name_text, Style::default().fg(highlight_fg).bg(highlight_bg)),
            ]));
            // The name editor line is the last line we just pushed
            Some(action_lines.len() - 1)
        } else {
            None
        };
        
        let actions_paragraph = Paragraph::new(action_lines)
            .block(Block::default().borders(Borders::ALL).title("Actions"))
            .style(Style::default().fg(fg_color).bg(bg_color));
        f.render_widget(actions_paragraph, actions_area);
        
        // Render cursor for name editor if in Add or Rename mode
        if matches!(state.mode, NotebookModalMode::Add | NotebookModalMode::Rename) {
            if let Some(line_idx) = name_editor_line_index {
                // Calculate cursor position
                // The text starts after "Name: " (6 characters)
                let name_prefix = "Name: ";
                let name_text = if state.name_editor.lines.is_empty() {
                    ""
                } else {
                    &state.name_editor.lines[0]
                };
                
                // Calculate cursor column position within the name text
                let line_len = name_text.chars().count();
                let cursor_col = state.name_editor.cursor_col.min(line_len);
                
                // Account for the "Name: " prefix
                let prefix_len = name_prefix.chars().count();
                let total_cursor_col = prefix_len + cursor_col;
                
                // Calculate position within the actions area
                // Account for borders (1 char on each side) and ensure we don't exceed width
                // Content area width is width - 2 (left + right borders)
                // Maximum column should be width - 3 to keep cursor within content (not on right border)
                let max_col = (actions_area.width.saturating_sub(3)) as usize;
                let visible_cursor_col = total_cursor_col.min(max_col);
                
                // Calculate y position:
                // - actions_area.y + 1 (top border, title is on the border line)
                // - + line_idx (which line in the paragraph content, 0-indexed)
                // Note: line_idx is the index of the name editor line in action_lines
                let x = actions_area.x + 1 + (visible_cursor_col as u16);
                let y = actions_area.y + 1 + (line_idx as u16);
                
                // Only set cursor if it's within the visible area
                if x < actions_area.x + actions_area.width && y < actions_area.y + actions_area.height {
                    f.set_cursor_position((x, y));
                }
            }
        }
    }
}

