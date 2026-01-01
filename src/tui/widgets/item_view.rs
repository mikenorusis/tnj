use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Layout as RatLayout, Direction, Constraint};
use ratatui::text::{Text, Line, Span};
use crate::tui::app::SelectedItem;
use crate::Config;
use crate::tui::widgets::color::parse_color;
use ratskin::RatSkin;
use termimad::minimad::Text as MinimadText;
use std::cmp;

/// Get content as a markdown-formatted string for an item
pub fn get_content_string(item: &SelectedItem) -> String {
    match item {
        SelectedItem::Task(task) => {
            let mut content = format!("**Title:** {}\n", task.title);
            content.push_str(&format!("**Status:** {}\n", task.status));
            
            if let Some(ref due_date) = task.due_date {
                content.push_str(&format!("**Due Date:** {}\n", due_date));
            }
            
            if let Some(ref description) = task.description {
                content.push_str("\n**Description/Notes:**\n\n");
                content.push_str(description);
                content.push('\n');
            }
            
            if let Some(ref tags) = task.tags {
                content.push_str(&format!("\n**Tags:** {}\n", tags));
            }
            
            content
        }
        SelectedItem::Note(note) => {
            let mut content = format!("**Title:** {}\n", note.title);
            
            if let Some(ref tags) = note.tags {
                content.push_str(&format!("\n**Tags:** {}\n", tags));
            }
            
            if let Some(ref note_content) = note.content {
                content.push_str("\n**Content:**\n\n");
                content.push_str(note_content);
                content.push('\n');
            }
            
            content
        }
        SelectedItem::Journal(journal) => {
            let mut content = format!("**Date:** {}\n", journal.date);
            
            if let Some(ref title) = journal.title {
                content.push_str(&format!("**Title:** {}\n", title));
            }
            
            if let Some(ref tags) = journal.tags {
                content.push_str(&format!("\n**Tags:** {}\n", tags));
            }
            
            if let Some(ref journal_content) = journal.content {
                content.push_str("\n**Content:**\n\n");
                content.push_str(journal_content);
                content.push('\n');
            }
            
            content
        }
    }
}

pub fn render_item_view(f: &mut Frame, area: Rect, item: &SelectedItem, config: &Config, scroll_offset: usize) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    // Split area into content and scrollbar first (needed to calculate width for parsing)
    let horizontal = RatLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1), // Scrollbar
        ])
        .split(area);
    
    let content_area = horizontal[0];
    let scrollbar_area = horizontal[1];

    let viewport_height = (area.height - 2) as usize; // Account for borders
    
    // Get content as markdown string
    let content_string = get_content_string(item);
    
    // Calculate text width (content area width minus borders)
    let text_width = (content_area.width.saturating_sub(2)) as usize;
    
    // Parse markdown with ratskin (requires width for wrapping)
    // Convert String to minimad::Text and usize to u16 for parse()
    let content_text_input = MinimadText::from(content_string.as_str());
    let text_width_u16: u16 = text_width.try_into().unwrap_or(u16::MAX);
    let content_lines = RatSkin::default().parse(content_text_input, text_width_u16);
    
    // Convert ratskin lines to ratatui lines, preserving styling from spans
    // With ratskin 0.3.0 and ratatui 0.30.0, types should be compatible
    let ratatui_lines: Vec<Line> = content_lines.into_iter().map(|line| {
        // Convert each span, preserving its style and content
        let spans: Vec<Span> = line.spans.into_iter().map(|span| {
            Span::styled(
                span.content.to_string(),
                span.style
            )
        }).collect();
        Line::from(spans)
    }).collect();
    let content_text = Text::from(ratatui_lines);
    
    // Calculate total lines (before wrapping)
    let total_lines = content_text.lines.len();
    
    // Clamp scroll offset
    let max_scroll = total_lines.saturating_sub(viewport_height);
    let scroll_offset = cmp::min(scroll_offset, max_scroll);
    
    // Slice Text to show only visible lines
    let start_line = scroll_offset;
    let end_line = cmp::min(start_line + viewport_height, total_lines);
    let visible_text = if start_line < total_lines {
        Text::from(content_text.lines[start_line..end_line].to_vec())
    } else {
        Text::default()
    };
    
    // Render content with markdown styling
    let title = match item {
        SelectedItem::Task(_) => "Task",
        SelectedItem::Note(_) => "Note",
        SelectedItem::Journal(_) => "Journal Entry",
    };
    
    // Apply theme foreground color to the text if needed
    // Note: ratskin/termimad applies its own styling, but we can override the base style
    // Use trim: false to preserve indentation for nested lists
    let base_style = Style::default().fg(parse_color(&config.get_active_theme().fg));
    let paragraph = Paragraph::new(visible_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(base_style)
        .wrap(ratatui::widgets::Wrap { trim: false });
    
    f.render_widget(paragraph, content_area);
    
    // Render scrollbar if content exceeds viewport
    if total_lines > viewport_height {
        // Content inner height: content_area.height - 2 (top and bottom borders)
        let content_inner_height = content_area.height.saturating_sub(2);
        
        // Create scrollbar area that matches the content inner area exactly
        let scrollbar_inner_area = Rect::new(
            scrollbar_area.x,
            content_area.y + 1, // Start after top border
            scrollbar_area.width,
            content_inner_height, // Match inner content height
        );
        
        // ScrollbarState: content_length is total_lines, viewport_content_length is viewport_height
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .viewport_content_length(viewport_height)
            .position(scroll_offset);
        
        let scrollbar = Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        
        // Render scrollbar in the inner area that matches the content text area
        f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
    }
}

