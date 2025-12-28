use ratatui::widgets::Tabs;
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::tui::app::Tab;
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};

pub fn render_tabs(f: &mut Frame, area: Rect, current_tab: Tab, config: &Config) {
    let active_theme = config.get_active_theme();
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let tab_bg = parse_color(&active_theme.tab_bg);
    
    // Use contrast-aware text color for non-selected tabs based on tab_bg
    // This ensures good readability regardless of terminal's gray rendering
    let tab_fg = get_contrast_text_color(tab_bg);

    // Create styled tabs with background colors to create box effect
    // Each tab uses background color with padding to look like a box
    let titles: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("  ", Style::default().bg(tab_bg)), // Left padding
            Span::styled("Tasks", Style::default().fg(tab_fg).bg(tab_bg)),
            Span::styled("  ", Style::default().bg(tab_bg)), // Right padding
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default().bg(tab_bg)), // Left padding
            Span::styled("Notes", Style::default().fg(tab_fg).bg(tab_bg)),
            Span::styled("  ", Style::default().bg(tab_bg)), // Right padding
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default().bg(tab_bg)), // Left padding
            Span::styled("Journal", Style::default().fg(tab_fg).bg(tab_bg)),
            Span::styled("  ", Style::default().bg(tab_bg)), // Right padding
        ]),
    ];

    let tab_index = match current_tab {
        Tab::Tasks => 0,
        Tab::Notes => 1,
        Tab::Journal => 2,
    };

    // Render tabs with space divider between boxes
    // Use contrast-aware text color for selected tab
    let highlight_fg = get_contrast_text_color(highlight_bg);
    
    let tabs = Tabs::new(titles)
        .select(tab_index)
        .style(Style::default().fg(fg_color).bg(bg_color))
        .highlight_style(
            Style::default()
                .fg(highlight_fg)
                .bg(highlight_bg)
                .add_modifier(Modifier::BOLD)
        )
        .divider("  ") // Space between tab boxes
        .padding("", "");

    f.render_widget(tabs, area);
}

