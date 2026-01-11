use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment, Constraint, Layout, Flex};
use ratatui::text::{Line, Span};
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};

pub fn render_unsaved_changes(f: &mut Frame, area: Rect, selection: usize, config: &Config) {
    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    
    // Calculate popup area (50% width, 35% height, centered)
    let popup_area = popup_area(area, 50, 35);
    
    // Clear the background first - this prevents content from showing through
    f.render_widget(Clear, popup_area);
    
    // Build all lines for the combined content
    let mut all_lines = Vec::new();
    
    // Add message lines
    all_lines.push(Line::from(Span::styled(
        "You have unsaved changes. What would you like to do?",
        Style::default().fg(fg_color).bg(bg_color)
    )));
    all_lines.push(Line::from(Span::styled("", Style::default()))); // Empty line
    
    // Build options with selection highlighting
    let options = vec!["Discard Changes", "Save Changes"];
    for (index, option) in options.iter().enumerate() {
        let is_selected = index == selection;
        let prefix = if is_selected { "> " } else { "  " };
        let text = format!("{}{}", prefix, option);
        
        let style = if is_selected {
            Style::default().fg(highlight_fg).bg(highlight_bg)
        } else {
            Style::default().fg(fg_color).bg(bg_color)
        };
        
        all_lines.push(Line::from(Span::styled(text, style)));
    }
    
    // Add instruction line
    all_lines.push(Line::from(Span::styled("", Style::default()))); // Empty line
    all_lines.push(Line::from(Span::styled(
        "Use ↑↓ to navigate, Enter to confirm, Esc to cancel",
        Style::default().fg(fg_color).bg(bg_color)
    )));
    
    let paragraph = Paragraph::new(all_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Unsaved Changes")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(fg_color).bg(bg_color)))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .alignment(Alignment::Center);
    
    f.render_widget(paragraph, popup_area);
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
