use ratatui::widgets::Tabs;
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::Frame;
use ratatui::layout::{Rect, Layout, Direction, Constraint};
use crate::tui::app::{Tab, App};
use crate::Config;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};

pub fn render_tabs(f: &mut Frame, area: Rect, current_tab: Tab, config: &Config, app: &App) {
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

    // Split area horizontally: tabs on left, notebook selector on right
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(25)]) // Notebook selector needs ~25 chars
        .split(area);
    
    let tabs_area = horizontal[0];
    let notebook_area = horizontal[1];
    
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

    f.render_widget(tabs, tabs_area);
    
    // Render notebook selector on the right
    let notebook_name = app.get_notebook_display_name(app.notebooks.current_notebook_id);
    let notebook_text = if app.ui.mode == crate::tui::app::Mode::NotebookModal {
        format!("Notebook: {} ▼", notebook_name)
    } else {
        format!("Notebook: {}", notebook_name)
    };
    
    // Truncate if too long (using char count, not byte count, for safe UTF-8 handling)
    let max_chars = notebook_area.width.saturating_sub(2) as usize;
    let char_count = notebook_text.chars().count();
    let display_text = if char_count > max_chars {
        let truncate_to = max_chars.saturating_sub(3); // Reserve 3 chars for "..."
        format!("{}...", notebook_text.chars().take(truncate_to).collect::<String>())
    } else {
        notebook_text
    };
    
    let notebook_line = Line::from(vec![
        Span::styled("  ", Style::default().bg(tab_bg)), // Left padding
        Span::styled(display_text, Style::default().fg(tab_fg).bg(tab_bg)),
        Span::styled("  ", Style::default().bg(tab_bg)), // Right padding
    ]);
    
    use ratatui::widgets::Paragraph;
    let notebook_widget = Paragraph::new(notebook_line)
        .style(Style::default().fg(fg_color).bg(bg_color));
    f.render_widget(notebook_widget, notebook_area);
}

