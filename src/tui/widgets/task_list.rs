use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget, Scrollbar, ScrollbarState, ScrollbarOrientation};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Layout, Direction, Constraint};
use ratatui::widgets::ListState;
use ratatui::text::Line;
use crate::models::Task;
use crate::Config;
use crate::tui::app::ListViewMode;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};
use crate::tui::widgets::tags::{parse_tags, format_tags_brackets};
use std::collections::HashMap;

pub fn render_task_list(f: &mut Frame, area: Rect, tasks: &[Task], total_count: usize, list_state: &mut ListState, config: &Config, view_mode: ListViewMode) {
    // Calculate max width for truncation (account for borders and padding)
    let max_width = area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding
    
    let active_theme = config.get_active_theme();
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    
    let items: Vec<ListItem> = match view_mode {
        ListViewMode::Simple => {
            tasks.iter().map(|task| {
                let archived_prefix = if task.archived { "[A] " } else { "" };
                let status_indicator = match task.status.as_str() {
                    "done" => "✓",
                    _ => "○",
                };
                
                let due_str = task.due_date.as_ref()
                    .map(|d| format!(" [{}]", d))
                    .unwrap_or_default();
                
                let mut title = format!("{} {}{} {}", 
                    status_indicator,
                    archived_prefix,
                    task.title,
                    due_str
                );
                
                // Truncate title if too long
                if title.chars().count() > max_width {
                    title = title.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
                }
                
                ListItem::new(title)
            }).collect()
        }
        ListViewMode::TwoLine => {
            tasks.iter().map(|task| {
                let archived_prefix = if task.archived { "[A] " } else { "" };
                let status_indicator = match task.status.as_str() {
                    "done" => "✓",
                    _ => "○",
                };
                
                let due_str = task.due_date.as_ref()
                    .map(|d| format!(" [{}]", d))
                    .unwrap_or_default();
                
                let mut first_line = format!("{} {}{} {}", 
                    status_indicator,
                    archived_prefix,
                    task.title,
                    due_str
                );
                
                // Truncate first line if too long
                if first_line.chars().count() > max_width {
                    first_line = first_line.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
                }
                
                // Second line with tags or [Untagged]
                let tags = parse_tags(task.tags.as_ref());
                let mut tags_line = if tags.is_empty() {
                    "  [Untagged]".to_string()
                } else {
                    format!("  {}", format_tags_brackets(&tags))
                };
                
                // Truncate tags if too long
                if tags_line.chars().count() > max_width {
                    tags_line = tags_line.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
                }
                
                ListItem::new(vec![
                    Line::from(first_line),
                    Line::from(tags_line),
                ])
            }).collect()
        }
        ListViewMode::GroupedByTags => {
            // Collect all unique tags
            let mut tag_map: HashMap<String, Vec<&Task>> = HashMap::new();
            let mut untagged: Vec<&Task> = Vec::new();
            
            for task in tasks {
                let tags = parse_tags(task.tags.as_ref());
                if tags.is_empty() {
                    untagged.push(task);
                } else {
                    for tag in tags {
                        tag_map.entry(tag).or_insert_with(Vec::new).push(task);
                    }
                }
            }
            
            // Sort tags alphabetically
            let mut sorted_tags: Vec<String> = tag_map.keys().cloned().collect();
            sorted_tags.sort();
            
            let mut items: Vec<ListItem> = Vec::new();
            
            // Add untagged section if there are untagged items
            if !untagged.is_empty() {
                items.push(ListItem::new("[Untagged]").style(Style::default().fg(parse_color(&active_theme.tab_bg))));
                for task in untagged {
                    let archived_prefix = if task.archived { "[A] " } else { "" };
                    let status_indicator = match task.status.as_str() {
                        "done" => "✓",
                        _ => "○",
                    };
                    
                    let due_str = task.due_date.as_ref()
                        .map(|d| format!(" [{}]", d))
                        .unwrap_or_default();
                    
                    let mut title = format!("  {} {}{} {}", 
                        status_indicator,
                        archived_prefix,
                        task.title,
                        due_str
                    );
                    
                    // Truncate title if too long
                    if title.chars().count() > max_width {
                        title = title.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
                    }
                    
                    items.push(ListItem::new(title));
                }
            }
            
            // Add tagged sections
            for tag in sorted_tags {
                items.push(ListItem::new(format!("[{}]", tag)).style(Style::default().fg(parse_color(&active_theme.tab_bg))));
                for task in &tag_map[&tag] {
                    let archived_prefix = if task.archived { "[A] " } else { "" };
                    let status_indicator = match task.status.as_str() {
                        "done" => "✓",
                        _ => "○",
                    };
                    
                    let due_str = task.due_date.as_ref()
                        .map(|d| format!(" [{}]", d))
                        .unwrap_or_default();
                    
                    let mut title = format!("  {} {}{} {}", 
                        status_indicator,
                        archived_prefix,
                        task.title,
                        due_str
                    );
                    
                    // Truncate title if too long
                    if title.chars().count() > max_width {
                        title = title.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
                    }
                    
                    items.push(ListItem::new(title));
                }
            }
            
            items
        }
    };

    // Split area to reserve space for scrollbar
    let list_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1), // Scrollbar
        ])
        .split(area);
    
    let list_area = list_areas[0];
    let scrollbar_area = list_areas[1];

    let title = format!("Items ({} of {})", tasks.len(), total_count);
    let list = List::new(items.clone())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(parse_color(&active_theme.fg)))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
        );

    StatefulWidget::render(list, list_area, f.buffer_mut(), list_state);

    // Render scrollbar if needed
    let total_items = items.len();
    let list_inner_height = list_area.height.saturating_sub(2) as usize; // Account for borders
    let items_per_line = match view_mode {
        ListViewMode::TwoLine => 2,
        _ => 1,
    };
    let visible_items = if list_inner_height >= items_per_line {
        list_inner_height / items_per_line
    } else {
        0
    };

    if total_items > visible_items && scrollbar_area.width > 0 && list_area.height > 2 {
        let scrollbar_inner_area = Rect::new(
            scrollbar_area.x,
            list_area.y + 1, // Start after top border
            scrollbar_area.width,
            list_area.height.saturating_sub(2), // Match inner list height
        );

        if scrollbar_inner_area.width > 0 && scrollbar_inner_area.height > 0 {
            // Calculate scroll position based on selected index
            let selected_index = list_state.selected().unwrap_or(0);
            let scroll_position = if selected_index < visible_items {
                0
            } else {
                selected_index.saturating_sub(visible_items - 1)
            };

            let mut scrollbar_state = ScrollbarState::new(total_items)
                .viewport_content_length(visible_items)
                .position(scroll_position);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("█");

            f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
        }
    }
}

