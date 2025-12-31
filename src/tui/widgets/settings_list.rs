use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget};
use ratatui::style::{Style, Modifier};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};

pub fn render_settings_list(f: &mut Frame, area: Rect, categories: &[String], list_state: &mut ListState, config: &Config, is_active: bool) {
    let items: Vec<ListItem> = categories.iter().map(|category| {
        ListItem::new(category.clone())
    }).collect();

    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
                .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() })
        );

    StatefulWidget::render(list, area, f.buffer_mut(), list_state);
}

