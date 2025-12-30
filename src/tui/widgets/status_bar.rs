use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Modifier};
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};

pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    message: Option<&String>,
    key_hints: &[String],
    config: &Config,
) {
    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);

    let (mut content, style) = if let Some(msg) = message {
        // Status messages get a highlighted background for visibility
        // Use contrast-aware text color
        let msg_fg = get_contrast_text_color(highlight_bg);
        (msg.clone(), Style::default().fg(msg_fg).bg(highlight_bg).add_modifier(Modifier::BOLD))
    } else {
        // Key hints use normal styling with bullet separators
        // Intelligently fit as many hints as possible when space is limited
        let max_width = area.width as usize;
        let separator = " • ";
        let separator_len = separator.chars().count();
        
        let mut hints_text = String::new();
        for (i, hint) in key_hints.iter().enumerate() {
            let hint_len = hint.chars().count();
            let current_len = hints_text.chars().count();
            
            // Calculate what the length would be if we add this hint
            let would_be_len = if i == 0 {
                hint_len
            } else {
                current_len + separator_len + hint_len
            };
            
            // If adding this hint would exceed the width, stop
            if would_be_len > max_width {
                // If we have at least one hint, add ellipsis to indicate more hints exist
                if !hints_text.is_empty() {
                    let ellipsis = "...";
                    let ellipsis_len = ellipsis.chars().count();
                    // Make sure we have room for ellipsis
                    if current_len + ellipsis_len <= max_width {
                        hints_text.push_str(ellipsis);
                    } else {
                        // Remove last separator and add ellipsis
                        let truncate_to = max_width.saturating_sub(ellipsis_len);
                        hints_text = hints_text.chars().take(truncate_to).collect::<String>();
                        hints_text.push_str(ellipsis);
                    }
                } else if i == 0 {
                    // Even the first hint is too long, truncate it with ellipsis
                    let ellipsis = "...";
                    let ellipsis_len = ellipsis.chars().count();
                    let truncate_to = max_width.saturating_sub(ellipsis_len);
                    hints_text = hint.chars().take(truncate_to).collect::<String>();
                    hints_text.push_str(ellipsis);
                }
                break;
            }
            
            // Add separator before hint (except for first one)
            if i > 0 {
                hints_text.push_str(separator);
            }
            hints_text.push_str(hint);
        }
        
        (hints_text, Style::default().fg(fg_color).bg(bg_color))
    };

    // For status messages, truncate if they exceed the available width
    // Reserve 3 characters for ellipsis
    if message.is_some() {
        let max_width = area.width as usize;
        if content.chars().count() > max_width {
            content = content.chars().take(max_width.saturating_sub(3)).collect::<String>() + "...";
        }
    }

    // Render status bar without Block wrapper - simple 1-line display
    // Following ratatui example pattern: content areas have borders, status bar is simple
    let paragraph = Paragraph::new(content)
        .style(style)
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

