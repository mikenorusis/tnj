use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::Config;
use crate::tui::widgets::color::parse_color;

pub fn render_filters_box(
    f: &mut Frame,
    area: Rect,
    summary: &str,
    config: &Config,
) {
    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    
    let paragraph = Paragraph::new(summary)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("f: Filters")
            .style(Style::default().fg(fg_color).bg(bg_color)))
        .style(Style::default().fg(fg_color))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

