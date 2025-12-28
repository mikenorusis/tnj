use ratatui::widgets::{Block, Borders, Paragraph, Clear, Scrollbar, ScrollbarState, ScrollbarOrientation};
use ratatui::style::Style;
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment, Constraint, Layout, Flex, Direction};
use ratatui::text::{Text, Line, Span};
use crate::Config;
use crate::tui::widgets::color::parse_color;
use ratskin::RatSkin;
use termimad::minimad::Text as MinimadText;
use std::cmp;

pub fn render_markdown_help(f: &mut Frame, area: Rect, config: &Config, example_scroll: usize, rendered_scroll: usize) {
    let active_theme = config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    
    // Calculate popup area (90% width, 85% height, centered)
    let popup_area = popup_area(area, 90, 85);
    
    // Clear the background first
    f.render_widget(Clear, popup_area);
    
    // Create outer block with title "Markdown Help"
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("Markdown Help")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(fg_color).bg(bg_color));
    
    // Get inner area (inside the outer block borders)
    let inner_area = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);
    
    // Split inner area into two columns: left (example) and right (rendered)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Example markdown
            Constraint::Percentage(50), // Right: Rendered markdown
        ])
        .split(inner_area);
    
    let example_area = columns[0];
    let rendered_area = columns[1];
    
    // Get the example markdown text
    let example_text = get_example_markdown();
    
    // Calculate example text lines (split by newlines)
    let example_lines: Vec<&str> = example_text.lines().collect();
    let example_total_lines = example_lines.len();
    let example_viewport_height = example_area.height.saturating_sub(2) as usize;
    
    // Clamp example scroll offset
    let example_max_scroll = example_total_lines.saturating_sub(example_viewport_height);
    let example_scroll = cmp::min(example_scroll, example_max_scroll);
    
    // Get visible example lines
    let example_start = example_scroll;
    let example_end = cmp::min(example_start + example_viewport_height, example_total_lines);
    let visible_example_lines = if example_start < example_total_lines {
        example_lines[example_start..example_end].join("\n")
    } else {
        String::new()
    };
    
    // Split example area to accommodate scrollbar if needed
    let (example_content_area, example_scrollbar_area) = if example_total_lines > example_viewport_height {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1), // Scrollbar
            ])
            .split(example_area);
        (horizontal[0], Some(horizontal[1]))
    } else {
        (example_area, None)
    };
    
    // Render example markdown on the left (raw text)
    let example_paragraph = Paragraph::new(visible_example_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Example")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(fg_color).bg(bg_color)))
        .style(Style::default().fg(fg_color).bg(bg_color))
        .wrap(ratatui::widgets::Wrap { trim: false });
    
    f.render_widget(example_paragraph, example_content_area);
    
    // Render example scrollbar if needed
    if let Some(scrollbar_area) = example_scrollbar_area {
        if scrollbar_area.width > 0 && example_content_area.height > 2 {
            let scrollbar_inner_area = Rect::new(
                scrollbar_area.x,
                example_content_area.y + 1,
                scrollbar_area.width,
                example_content_area.height.saturating_sub(2),
            );
            if scrollbar_inner_area.width > 0 && scrollbar_inner_area.height > 0 {
                let mut scrollbar_state = ScrollbarState::new(example_total_lines)
                    .viewport_content_length(example_viewport_height)
                    .position(example_scroll);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
            }
        }
    }
    
    // Render rendered markdown on the right (formatted)
    let text_width = (rendered_area.width.saturating_sub(2)) as usize;
    let text_width_u16: u16 = text_width.try_into().unwrap_or(u16::MAX);
    
    // Parse markdown with ratskin
    let content_text_input = MinimadText::from(example_text.as_str());
    let content_lines = RatSkin::default().parse(content_text_input, text_width_u16);
    
    // Convert ratskin lines to ratatui lines
    let ratatui_lines: Vec<Line> = content_lines.into_iter().map(|line| {
        let spans: Vec<Span> = line.spans.into_iter().map(|span| {
            Span::styled(
                span.content.to_string(),
                span.style
            )
        }).collect();
        Line::from(spans)
    }).collect();
    let rendered_text = Text::from(ratatui_lines);
    
    // Calculate rendered text lines
    let rendered_total_lines = rendered_text.lines.len();
    let rendered_viewport_height = rendered_area.height.saturating_sub(2) as usize;
    
    // Clamp rendered scroll offset
    let rendered_max_scroll = rendered_total_lines.saturating_sub(rendered_viewport_height);
    let rendered_scroll = cmp::min(rendered_scroll, rendered_max_scroll);
    
    // Get visible rendered lines
    let rendered_start = rendered_scroll;
    let rendered_end = cmp::min(rendered_start + rendered_viewport_height, rendered_total_lines);
    let visible_rendered_text = if rendered_start < rendered_total_lines {
        Text::from(rendered_text.lines[rendered_start..rendered_end].to_vec())
    } else {
        Text::default()
    };
    
    // Split rendered area to accommodate scrollbar if needed
    let (rendered_content_area, rendered_scrollbar_area) = if rendered_total_lines > rendered_viewport_height {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1), // Scrollbar
            ])
            .split(rendered_area);
        (horizontal[0], Some(horizontal[1]))
    } else {
        (rendered_area, None)
    };
    
    let base_style = Style::default().fg(fg_color);
    let rendered_paragraph = Paragraph::new(visible_rendered_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Rendered")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(fg_color).bg(bg_color)))
        .style(base_style)
        .wrap(ratatui::widgets::Wrap { trim: false });
    
    f.render_widget(rendered_paragraph, rendered_content_area);
    
    // Render rendered scrollbar if needed
    if let Some(scrollbar_area) = rendered_scrollbar_area {
        if scrollbar_area.width > 0 && rendered_content_area.height > 2 {
            let scrollbar_inner_area = Rect::new(
                scrollbar_area.x,
                rendered_content_area.y + 1,
                scrollbar_area.width,
                rendered_content_area.height.saturating_sub(2),
            );
            if scrollbar_inner_area.width > 0 && scrollbar_inner_area.height > 0 {
                let mut scrollbar_state = ScrollbarState::new(rendered_total_lines)
                    .viewport_content_length(rendered_viewport_height)
                    .position(rendered_scroll);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
            }
        }
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn get_example_markdown() -> String {
    r#"# Heading 1
## Heading 2

**Bold Text**
*Italicized Test*
***Bold and Iitalics***

**Ordered List**
1. Item 1
2. Item 2
  1. Sub-Item 1

**Unordered List**
* Item 1
* Item 2
  * Item 1

**Unordered List **
- Item 1
  - Sub-Item 1
  - Sub-Item 2
- Item 2

```
//Code Block
int i = 0;
```

|:-:|-
|**feature**|**details**|
|-:|-
| tables | pipe based, with or without alignments
| italic, bold | star based |
| inline code | `with backquotes` (it works in tables too)
| code bloc |with tabs or code fences
| crossed text| ~~like this~~
| horizontal rule | Use 3 or more dashes (`---`)
| lists |* unordered lists supported
|  |* ordered lists *not* supported
| quotes |> What a wonderful time to be alive!
|-"#.to_string()
}

