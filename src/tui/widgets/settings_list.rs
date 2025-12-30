use ratatui::widgets::{Block, Borders, List, ListItem, StatefulWidget};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};

pub fn render_settings_list(f: &mut Frame, area: Rect, categories: &[String], list_state: &mut ListState, config: &Config) {
    let items: Vec<ListItem> = categories.iter().map(|category| {
        ListItem::new(category.clone())
    }).collect();

    let active_theme = config.get_active_theme();
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = get_contrast_text_color(highlight_bg);
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .style(Style::default().fg(parse_color(&active_theme.fg)))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
        );

    StatefulWidget::render(list, area, f.buffer_mut(), list_state);
}

