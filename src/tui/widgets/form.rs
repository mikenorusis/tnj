use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::style::{Style, Modifier};
use ratatui::Frame;
use ratatui::layout::{Rect, Constraint, Layout, Direction};
use ratatui::text::{Line, Span};
use crate::Config;
use crate::tui::app::{TaskForm, NoteForm, JournalForm, TaskField, NoteField, JournalField};
use crate::tui::widgets::editor::Editor;
use crate::tui::widgets::color::{parse_color, get_contrast_text_color};
use crate::models::Notebook;

/// Helper function to wrap a long line to fit within a given width
/// Returns wrapped lines and the character offset where each wrapped line starts in the original line
fn wrap_line_with_offsets(line: &str, width: usize) -> Vec<(usize, String)> {
    if width == 0 {
        return vec![(0, String::new())];
    }
    
    if line.chars().count() <= width {
        return vec![(0, line.to_string())];
    }
    
    let mut wrapped = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut char_offset = 0;
    
    while char_offset < chars.len() {
        let remaining = chars.len() - char_offset;
        if remaining <= width {
            wrapped.push((char_offset, chars[char_offset..].iter().collect()));
            break;
        }
        
        // Try to find a word boundary to break at
        let mut break_pos = width;
        let search_end = (char_offset + width + 1).min(chars.len());
        
        // Look for the last space before the width limit
        for i in (char_offset..search_end).rev() {
            if i < chars.len() && chars[i].is_whitespace() {
                break_pos = i - char_offset + 1;
                break;
            }
        }
        
        // Extract the chunk
        let end_pos = char_offset + break_pos;
        wrapped.push((char_offset, chars[char_offset..end_pos].iter().collect()));
        char_offset = end_pos;
        
        // Skip leading whitespace on next line
        while char_offset < chars.len() && chars[char_offset].is_whitespace() {
            char_offset += 1;
        }
    }
    
    if wrapped.is_empty() {
        wrapped.push((0, String::new()));
    }
    
    wrapped
}

/// Structure to track wrapped line information
struct WrappedLineInfo {
    logical_line: usize,
    char_offset: usize,  // Character offset in the logical line where this wrapped line starts
    wrapped_line: String,
}

/// Build all wrapped lines from editor content
fn build_all_wrapped_lines(editor_lines: &[String], content_width: usize) -> Vec<WrappedLineInfo> {
    let mut all_wrapped = Vec::new();
    
    for (logical_idx, line_str) in editor_lines.iter().enumerate() {
        let wrapped = wrap_line_with_offsets(line_str, content_width);
        for (char_offset, wrapped_line) in wrapped {
            all_wrapped.push(WrappedLineInfo {
                logical_line: logical_idx,
                char_offset,
                wrapped_line,
            });
        }
    }
    
    all_wrapped
}

/// Find which wrapped line contains the cursor position
fn find_cursor_wrapped_line(wrapped_lines: &[WrappedLineInfo], cursor_logical_line: usize, cursor_col: usize) -> usize {
    // Find all wrapped lines for the cursor's logical line
    for (idx, wrapped) in wrapped_lines.iter().enumerate() {
        if wrapped.logical_line == cursor_logical_line {
            // Check if cursor is in this wrapped line
            let wrapped_line_len = wrapped.wrapped_line.chars().count();
            if cursor_col >= wrapped.char_offset && cursor_col < wrapped.char_offset + wrapped_line_len {
                return idx;
            }
            // If this is the last wrapped line for this logical line, check if cursor is at or beyond it
            if idx + 1 >= wrapped_lines.len() || wrapped_lines[idx + 1].logical_line != cursor_logical_line {
                if cursor_col >= wrapped.char_offset {
                    return idx;
                }
            }
        }
        if wrapped.logical_line > cursor_logical_line {
            break;
        }
    }
    
    // Fallback: return the first wrapped line of the cursor's logical line, or 0 if not found
    wrapped_lines.iter()
        .position(|w| w.logical_line == cursor_logical_line)
        .unwrap_or(0)
}

/// Helper function to check if a character position is within selection bounds
fn is_char_selected(
    logical_line: usize,
    char_offset: usize,
    selection_bounds: Option<((usize, usize), (usize, usize))>
) -> bool {
    if let Some(((start_line, start_col), (end_line, end_col))) = selection_bounds {
        if logical_line < start_line || logical_line > end_line {
            return false;
        }
        if logical_line == start_line && logical_line == end_line {
            // Single line selection
            return char_offset >= start_col && char_offset < end_col;
        } else if logical_line == start_line {
            // First line of multi-line selection
            return char_offset >= start_col;
        } else if logical_line == end_line {
            // Last line of multi-line selection
            return char_offset < end_col;
        } else {
            // Middle line of multi-line selection
            return true;
        }
    }
    false
}

/// Helper function to build a Line with selection highlighting for single-line fields
fn build_single_line_with_selection(editor: &Editor, style: Style) -> Line<'static> {
    // For single-line fields, we only care about the first line
    let text = if editor.lines.is_empty() {
        String::new()
    } else {
        editor.lines[0].clone()
    };
    let selection_bounds = editor.get_selection_bounds();
    
    // If there's no selection, just use the normal style
    if selection_bounds.is_none() {
        return Line::from(Span::styled(text, style));
    }
    
    // Build spans with selection highlighting
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    
    let mut i = 0;
    while i < chars.len() {
        let is_selected = is_char_selected(0, i, selection_bounds);
        
        // Find the end of the current selection state
        let mut j = i;
        while j < chars.len() {
            let next_is_selected = is_char_selected(0, j, selection_bounds);
            if next_is_selected != is_selected {
                break;
            }
            j += 1;
        }
        
        // Create span for this segment
        let segment: String = chars[i..j].iter().collect();
        let segment_style = if is_selected {
            style.add_modifier(Modifier::REVERSED)
        } else {
            style
        };
        spans.push(Span::styled(segment, segment_style));
        
        i = j;
    }
    
    Line::from(spans)
}

/// Helper function to build lines from an Editor for multi-line fields
/// available_height is the total field area height (including borders)
/// available_width is the total field area width (including borders)
fn build_editor_lines(editor: &Editor, _is_active: bool, style: Style, available_height: u16, available_width: u16) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let editor_lines = editor.lines.clone();
    
    // Get selection bounds if there's a selection
    let selection_bounds = editor.get_selection_bounds();
    
    if editor_lines.is_empty() {
        // Empty content - show placeholder line
        lines.push(Line::from(Span::styled("", style)));
    } else {
        // Content area = total height - 2 (top border + bottom border)
        // Content width = total width - 2 (left border + right border)
        let content_height = available_height.saturating_sub(2) as usize;
        let content_width = available_width.saturating_sub(2) as usize;
        
        // Build all wrapped lines
        let all_wrapped = build_all_wrapped_lines(&editor_lines, content_width);
        
        if all_wrapped.is_empty() {
            lines.push(Line::from(Span::styled("", style)));
        } else {
            // Find which wrapped line the cursor is on
            let cursor_wrapped_line = find_cursor_wrapped_line(&all_wrapped, editor.cursor_line, editor.cursor_col);
            
            // Calculate scroll offset to ensure cursor is visible
            // We want the cursor's wrapped line to be visible
            let scroll_start = if cursor_wrapped_line < content_height {
                0
            } else {
                cursor_wrapped_line.saturating_sub(content_height - 1)
            };
            
            let start_line = scroll_start.min(all_wrapped.len());
            let end_line = std::cmp::min(start_line + content_height, all_wrapped.len());
            
            // Create Line objects from visible wrapped lines with selection highlighting
            for wrapped_info in all_wrapped[start_line..end_line].iter() {
                let logical_line = wrapped_info.logical_line;
                let char_offset_start = wrapped_info.char_offset;
                let wrapped_text = &wrapped_info.wrapped_line;
                
                // If there's no selection, just use the normal style
                if selection_bounds.is_none() {
                    lines.push(Line::from(Span::styled(wrapped_text.clone(), style)));
                } else {
                    // Build spans with selection highlighting
                    let mut spans = Vec::new();
                    let chars: Vec<char> = wrapped_text.chars().collect();
                    
                    let mut i = 0;
                    while i < chars.len() {
                        let char_pos = char_offset_start + i;
                        let is_selected = is_char_selected(logical_line, char_pos, selection_bounds);
                        
                        // Find the end of the current selection state
                        let mut j = i;
                        while j < chars.len() {
                            let next_char_pos = char_offset_start + j;
                            let next_is_selected = is_char_selected(logical_line, next_char_pos, selection_bounds);
                            if next_is_selected != is_selected {
                                break;
                            }
                            j += 1;
                        }
                        
                        // Create span for this segment
                        let segment: String = chars[i..j].iter().collect();
                        let segment_style = if is_selected {
                            style.add_modifier(Modifier::REVERSED)
                        } else {
                            style
                        };
                        spans.push(Span::styled(segment, segment_style));
                        
                        i = j;
                    }
                    
                    lines.push(Line::from(spans));
                }
            }
            
            // If we have no visible lines but there's content, show at least one empty line
            if lines.is_empty() {
                lines.push(Line::from(Span::styled("", style)));
            }
        }
    }
    
    lines
}

/// Helper function to calculate viewport height for a multi-line field
/// Returns the content height (excluding borders) for scroll calculations
pub fn calculate_field_viewport_height(field_area_height: u16) -> usize {
    // Content area = total height - 2 (top border + bottom border)
    field_area_height.saturating_sub(2) as usize
}

/// Form type identifier for calculating field heights
#[derive(Clone, Copy)]
pub enum FormType {
    Task,
    Note,
    Journal,
}

impl From<&crate::tui::app::CreateForm> for FormType {
    fn from(form: &crate::tui::app::CreateForm) -> Self {
        match form {
            crate::tui::app::CreateForm::Task(_) => FormType::Task,
            crate::tui::app::CreateForm::Note(_) => FormType::Note,
            crate::tui::app::CreateForm::Journal(_) => FormType::Journal,
        }
    }
}

/// Calculate the height of a multi-line field area based on form type and main area height
/// This matches the layout constraints used in the render functions
pub fn calculate_multi_line_field_height(main_area_height: u16, form_type: FormType) -> u16 {
    use ratatui::layout::{Layout, Direction, Constraint};
    
    // Determine which field is multi-line and get constraints
    let (constraints, multi_line_index, single_line_count) = match form_type {
        FormType::Task => {
            (
                vec![
                    Constraint::Length(3), // Title
                    Constraint::Min(5),   // Description (multi-line)
                    Constraint::Length(3), // Due Date
                    Constraint::Length(3), // Tags
                ],
                1, // Description index
                3, // 3 single-line fields
            )
        }
        FormType::Note => {
            (
                vec![
                    Constraint::Length(3), // Title
                    Constraint::Length(3), // Tags
                    Constraint::Min(5),   // Content (multi-line)
                ],
                2, // Content index
                2, // 2 single-line fields
            )
        }
        FormType::Journal => {
            (
                vec![
                    Constraint::Length(3), // Date
                    Constraint::Length(3), // Title
                    Constraint::Length(3), // Tags
                    Constraint::Min(5),   // Content (multi-line)
                ],
                3, // Content index
                3, // 3 single-line fields
            )
        }
    };
    
    // Calculate layout to get actual field area height
    let test_area = Rect::new(0, 0, 80, main_area_height); // Use a test width
    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(test_area);
    
    if multi_line_index < field_areas.len() {
        field_areas[multi_line_index].height
    } else {
        // Fallback: estimate based on remaining space
        let single_line_fields = single_line_count * 3; // Each single-line field is 3 lines
        main_area_height.saturating_sub(single_line_fields).max(5)
    }
}

pub fn render_task_form(f: &mut Frame, area: Rect, form: &TaskForm, config: &Config, notebooks: &[Notebook]) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let active_theme = config.get_active_theme();
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    let highlight_style = Style::default()
        .bg(highlight_bg)
        .fg(highlight_fg);
    let inactive_field_style = Style::default()
        .fg(parse_color(&active_theme.fg))
        .add_modifier(Modifier::DIM);

    // Split area vertically into field sections
    // Single-line fields: 3 lines each (border top + content + border bottom)
    // Description field: flexible height
    let constraints = vec![
        Constraint::Length(3), // Title
        Constraint::Min(5),   // Description (minimum 5 lines for multi-line)
        Constraint::Length(3), // Due Date
        Constraint::Length(3), // Tags
        Constraint::Length(3), // Notebook
    ];
    
    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Title field
    let is_title_active = form.current_field == TaskField::Title;
    let title_style = if is_title_active { highlight_style } else { inactive_field_style };
    let title_line = build_single_line_with_selection(&form.title, title_style);
    let title_paragraph = Paragraph::new(title_line)
        .block(Block::default().borders(Borders::ALL).title("Title"));
    f.render_widget(title_paragraph, field_areas[0]);

    // Description field (multi-line)
    let is_desc_active = form.current_field == TaskField::Description;
    let desc_style = if is_desc_active { highlight_style } else { inactive_field_style };
    
    // Calculate if we need vertical scrollbar (lines wrap, so no horizontal scrollbar needed)
    let content_height = field_areas[1].height.saturating_sub(2) as usize;
    let content_width = field_areas[1].width.saturating_sub(2) as usize;
    let all_wrapped = build_all_wrapped_lines(&form.description.lines, content_width);
    let total_wrapped_lines = all_wrapped.len();
    
    // Calculate vertical scroll position (same logic as build_editor_lines)
    let cursor_wrapped_line = find_cursor_wrapped_line(&all_wrapped, form.description.cursor_line, form.description.cursor_col);
    let vertical_scroll_pos = if cursor_wrapped_line < content_height {
        0
    } else {
        cursor_wrapped_line.saturating_sub(content_height - 1)
    };
    
    let needs_vertical_scrollbar = total_wrapped_lines > content_height;
    
    // Split content area to accommodate vertical scrollbar only
    let (desc_area, vertical_scrollbar_area) = if needs_vertical_scrollbar {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1), // Vertical scrollbar
            ])
            .split(field_areas[1]);
        (horizontal_chunks[0], Some(horizontal_chunks[1]))
    } else {
        (field_areas[1], None)
    };
    
    let desc_lines = build_editor_lines(&form.description, is_desc_active, desc_style, desc_area.height, desc_area.width);
    let desc_paragraph = Paragraph::new(desc_lines)
        .style(desc_style)
        .block(Block::default().borders(Borders::ALL).title("Description/Notes"));
    f.render_widget(desc_paragraph, desc_area);
    
    // Render vertical scrollbar
    if let Some(v_scrollbar_area) = vertical_scrollbar_area {
        if v_scrollbar_area.width > 0 && desc_area.height > 2 {
            let scrollbar_inner_area = Rect::new(
                v_scrollbar_area.x,
                desc_area.y + 1,
                v_scrollbar_area.width,
                desc_area.height.saturating_sub(2),
            );
            if scrollbar_inner_area.width > 0 && scrollbar_inner_area.height > 0 {
                let mut scrollbar_state = ScrollbarState::new(total_wrapped_lines)
                    .viewport_content_length(content_height)
                    .position(vertical_scroll_pos);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
            }
        }
    }

    // Due Date field
    let is_date_active = form.current_field == TaskField::DueDate;
    let date_style = if is_date_active { highlight_style } else { inactive_field_style };
    let date_line = build_single_line_with_selection(&form.due_date, date_style);
    let date_paragraph = Paragraph::new(date_line)
        .block(Block::default().borders(Borders::ALL).title("Due Date (YYYY-MM-DD)"));
    f.render_widget(date_paragraph, field_areas[2]);

    // Tags field
    let is_tags_active = form.current_field == TaskField::Tags;
    let tags_style = if is_tags_active { highlight_style } else { inactive_field_style };
    let tags_line = build_single_line_with_selection(&form.tags, tags_style);
    let tags_paragraph = Paragraph::new(tags_line)
        .block(Block::default().borders(Borders::ALL).title("Tags"));
    f.render_widget(tags_paragraph, field_areas[3]);

    // Notebook field
    let is_notebook_active = form.current_field == TaskField::Notebook;
    let notebook_style = if is_notebook_active { highlight_style } else { inactive_field_style };
    let notebook_display = if form.notebook_selected_index == 0 {
        "[None]".to_string()
    } else {
        notebooks.get(form.notebook_selected_index - 1)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "[None]".to_string())
    };
    let notebook_paragraph = Paragraph::new(notebook_display)
        .block(Block::default().borders(Borders::ALL).title("Notebook"))
        .style(notebook_style);
    f.render_widget(notebook_paragraph, field_areas[4]);

    // Set cursor position for active field
    if let Some((x, y)) = get_cursor_position_for_task_field(area, form, &field_areas) {
        f.set_cursor_position((x, y));
    }
}

pub fn render_note_form(f: &mut Frame, area: Rect, form: &NoteForm, config: &Config, notebooks: &[Notebook]) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let active_theme = config.get_active_theme();
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    let highlight_style = Style::default()
        .bg(highlight_bg)
        .fg(highlight_fg);
    let inactive_field_style = Style::default()
        .fg(parse_color(&active_theme.fg))
        .add_modifier(Modifier::DIM);

    // Split area vertically into field sections
    let constraints = vec![
        Constraint::Length(3), // Title
        Constraint::Length(3), // Tags
        Constraint::Length(3), // Notebook
        Constraint::Min(5),   // Content (minimum 5 lines for multi-line)
    ];
    
    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Title field
    let is_title_active = form.current_field == NoteField::Title;
    let title_style = if is_title_active { highlight_style } else { inactive_field_style };
    let title_line = build_single_line_with_selection(&form.title, title_style);
    let title_paragraph = Paragraph::new(title_line)
        .block(Block::default().borders(Borders::ALL).title("Title"));
    f.render_widget(title_paragraph, field_areas[0]);

    // Tags field
    let is_tags_active = form.current_field == NoteField::Tags;
    let tags_style = if is_tags_active { highlight_style } else { inactive_field_style };
    let tags_line = build_single_line_with_selection(&form.tags, tags_style);
    let tags_paragraph = Paragraph::new(tags_line)
        .block(Block::default().borders(Borders::ALL).title("Tags"));
    f.render_widget(tags_paragraph, field_areas[1]);

    // Notebook field
    let is_notebook_active = form.current_field == NoteField::Notebook;
    let notebook_style = if is_notebook_active { highlight_style } else { inactive_field_style };
    let notebook_display = if form.notebook_selected_index == 0 {
        "[None]".to_string()
    } else {
        notebooks.get(form.notebook_selected_index - 1)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "[None]".to_string())
    };
    let notebook_paragraph = Paragraph::new(notebook_display)
        .block(Block::default().borders(Borders::ALL).title("Notebook"))
        .style(notebook_style);
    f.render_widget(notebook_paragraph, field_areas[2]);

    // Content field (multi-line)
    let is_content_active = form.current_field == NoteField::Content;
    let content_style = if is_content_active { highlight_style } else { inactive_field_style };
    
    // Calculate if we need vertical scrollbar (lines wrap, so no horizontal scrollbar needed)
    let content_height = field_areas[3].height.saturating_sub(2) as usize;
    let content_width = field_areas[3].width.saturating_sub(2) as usize;
    let all_wrapped = build_all_wrapped_lines(&form.content.lines, content_width);
    let total_wrapped_lines = all_wrapped.len();
    
    // Calculate vertical scroll position (same logic as build_editor_lines)
    let cursor_wrapped_line = find_cursor_wrapped_line(&all_wrapped, form.content.cursor_line, form.content.cursor_col);
    let vertical_scroll_pos = if cursor_wrapped_line < content_height {
        0
    } else {
        cursor_wrapped_line.saturating_sub(content_height - 1)
    };
    
    let needs_vertical_scrollbar = total_wrapped_lines > content_height;
    
    // Split content area to accommodate vertical scrollbar only
    let (content_area, vertical_scrollbar_area) = if needs_vertical_scrollbar {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1), // Vertical scrollbar
            ])
            .split(field_areas[3]);
        (horizontal_chunks[0], Some(horizontal_chunks[1]))
    } else {
        (field_areas[3], None)
    };
    
    let content_lines = build_editor_lines(&form.content, is_content_active, content_style, content_area.height, content_area.width);
    let content_paragraph = Paragraph::new(content_lines)
        .style(content_style)
        .block(Block::default().borders(Borders::ALL).title("Content"));
    f.render_widget(content_paragraph, content_area);
    
    // Render vertical scrollbar
    if let Some(v_scrollbar_area) = vertical_scrollbar_area {
        if v_scrollbar_area.width > 0 && content_area.height > 2 {
            let scrollbar_inner_area = Rect::new(
                v_scrollbar_area.x,
                content_area.y + 1,
                v_scrollbar_area.width,
                content_area.height.saturating_sub(2),
            );
            if scrollbar_inner_area.width > 0 && scrollbar_inner_area.height > 0 {
                let mut scrollbar_state = ScrollbarState::new(total_wrapped_lines)
                    .viewport_content_length(content_height)
                    .position(vertical_scroll_pos);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
            }
        }
    }
    

    // Set cursor position for active field
    if let Some((x, y)) = get_cursor_position_for_note_field(area, form, &field_areas) {
        f.set_cursor_position((x, y));
    }
}

pub fn render_journal_form(f: &mut Frame, area: Rect, form: &JournalForm, config: &Config, notebooks: &[Notebook]) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let active_theme = config.get_active_theme();
    let highlight_bg = parse_color(&active_theme.highlight_bg);
    let highlight_fg = if active_theme.highlight_fg.is_empty() {
        get_contrast_text_color(highlight_bg)
    } else {
        parse_color(&active_theme.highlight_fg)
    };
    let highlight_style = Style::default()
        .bg(highlight_bg)
        .fg(highlight_fg);
    let inactive_field_style = Style::default()
        .fg(parse_color(&active_theme.fg))
        .add_modifier(Modifier::DIM);

    // Split area vertically into field sections
    let constraints = vec![
        Constraint::Length(3), // Date
        Constraint::Length(3), // Title
        Constraint::Length(3), // Tags
        Constraint::Length(3), // Notebook
        Constraint::Min(5),   // Content (minimum 5 lines for multi-line)
    ];
    
    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Date field
    let is_date_active = form.current_field == JournalField::Date;
    let date_style = if is_date_active { highlight_style } else { inactive_field_style };
    let date_line = build_single_line_with_selection(&form.date, date_style);
    let date_paragraph = Paragraph::new(date_line)
        .block(Block::default().borders(Borders::ALL).title("Date (YYYY-MM-DD)"));
    f.render_widget(date_paragraph, field_areas[0]);

    // Title field
    let is_title_active = form.current_field == JournalField::Title;
    let title_style = if is_title_active { highlight_style } else { inactive_field_style };
    let title_line = build_single_line_with_selection(&form.title, title_style);
    let title_paragraph = Paragraph::new(title_line)
        .block(Block::default().borders(Borders::ALL).title("Title"));
    f.render_widget(title_paragraph, field_areas[1]);

    // Tags field
    let is_tags_active = form.current_field == JournalField::Tags;
    let tags_style = if is_tags_active { highlight_style } else { inactive_field_style };
    let tags_line = build_single_line_with_selection(&form.tags, tags_style);
    let tags_paragraph = Paragraph::new(tags_line)
        .block(Block::default().borders(Borders::ALL).title("Tags"));
    f.render_widget(tags_paragraph, field_areas[2]);

    // Notebook field
    let is_notebook_active = form.current_field == JournalField::Notebook;
    let notebook_style = if is_notebook_active { highlight_style } else { inactive_field_style };
    let notebook_display = if form.notebook_selected_index == 0 {
        "[None]".to_string()
    } else {
        notebooks.get(form.notebook_selected_index - 1)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| "[None]".to_string())
    };
    let notebook_paragraph = Paragraph::new(notebook_display)
        .block(Block::default().borders(Borders::ALL).title("Notebook"))
        .style(notebook_style);
    f.render_widget(notebook_paragraph, field_areas[3]);

    // Content field (multi-line)
    let is_content_active = form.current_field == JournalField::Content;
    let content_style = if is_content_active { highlight_style } else { inactive_field_style };
    
    // Calculate if we need vertical scrollbar (lines wrap, so no horizontal scrollbar needed)
    let content_height = field_areas[4].height.saturating_sub(2) as usize;
    let content_width = field_areas[4].width.saturating_sub(2) as usize;
    let all_wrapped = build_all_wrapped_lines(&form.content.lines, content_width);
    let total_wrapped_lines = all_wrapped.len();
    
    // Calculate vertical scroll position (same logic as build_editor_lines)
    let cursor_wrapped_line = find_cursor_wrapped_line(&all_wrapped, form.content.cursor_line, form.content.cursor_col);
    let vertical_scroll_pos = if cursor_wrapped_line < content_height {
        0
    } else {
        cursor_wrapped_line.saturating_sub(content_height - 1)
    };
    
    let needs_vertical_scrollbar = total_wrapped_lines > content_height;
    
    // Split content area to accommodate vertical scrollbar only
    let (content_area, vertical_scrollbar_area) = if needs_vertical_scrollbar {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1), // Vertical scrollbar
            ])
            .split(field_areas[4]);
        (horizontal_chunks[0], Some(horizontal_chunks[1]))
    } else {
        (field_areas[4], None)
    };
    
    let content_lines = build_editor_lines(&form.content, is_content_active, content_style, content_area.height, content_area.width);
    let content_paragraph = Paragraph::new(content_lines)
        .style(content_style)
        .block(Block::default().borders(Borders::ALL).title("Content"));
    f.render_widget(content_paragraph, content_area);
    
    // Render vertical scrollbar
    if let Some(v_scrollbar_area) = vertical_scrollbar_area {
        if v_scrollbar_area.width > 0 && content_area.height > 2 {
            let scrollbar_inner_area = Rect::new(
                v_scrollbar_area.x,
                content_area.y + 1,
                v_scrollbar_area.width,
                content_area.height.saturating_sub(2),
            );
            if scrollbar_inner_area.width > 0 && scrollbar_inner_area.height > 0 {
                let mut scrollbar_state = ScrollbarState::new(total_wrapped_lines)
                    .viewport_content_length(content_height)
                    .position(vertical_scroll_pos);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                f.render_stateful_widget(scrollbar, scrollbar_inner_area, &mut scrollbar_state);
            }
        }
    }
    

    // Set cursor position for active field
    if let Some((x, y)) = get_cursor_position_for_journal_field(area, form, &field_areas) {
        f.set_cursor_position((x, y));
    }
}

fn get_cursor_position_for_task_field(_area: Rect, form: &TaskForm, field_areas: &[Rect]) -> Option<(u16, u16)> {
    let editor = match form.current_field {
        TaskField::Title => &form.title,
        TaskField::Description => &form.description,
        TaskField::DueDate => &form.due_date,
        TaskField::Tags => &form.tags,
        TaskField::Notebook => return None, // Notebook field doesn't use cursor
    };

    let field_index = match form.current_field {
        TaskField::Title => 0,
        TaskField::Description => 1,
        TaskField::DueDate => 2,
        TaskField::Tags => 3,
        TaskField::Notebook => return None,
    };

    if field_index >= field_areas.len() {
        return None;
    }

    let field_area = field_areas[field_index];
    
    // For multi-line fields (Description), calculate cursor position accounting for wrapping
    if form.current_field == TaskField::Description {
        let content_width = field_area.width.saturating_sub(2) as usize;
        let content_height = field_area.height.saturating_sub(2) as usize;
        
        // Build wrapped lines
        let all_wrapped = build_all_wrapped_lines(&editor.lines, content_width);
        if all_wrapped.is_empty() {
            return Some((field_area.x + 1, field_area.y + 1));
        }
        
        // Find which wrapped line contains the cursor
        let cursor_wrapped_idx = find_cursor_wrapped_line(&all_wrapped, editor.cursor_line, editor.cursor_col);
        
        // Calculate scroll offset to ensure cursor is visible
        let scroll_start = if cursor_wrapped_idx < content_height {
            0
        } else {
            cursor_wrapped_idx.saturating_sub(content_height - 1)
        };
        
        // Calculate visible wrapped line index
        let visible_wrapped_idx = cursor_wrapped_idx.saturating_sub(scroll_start);
        if visible_wrapped_idx >= content_height {
            return None; // Cursor is not visible
        }
        
        // Bounds check: ensure cursor_wrapped_idx is within all_wrapped array
        if cursor_wrapped_idx >= all_wrapped.len() {
            return None;
        }
        
        // Calculate column within the wrapped line
        let wrapped_info = &all_wrapped[cursor_wrapped_idx];
        let col_in_wrapped = editor.cursor_col.saturating_sub(wrapped_info.char_offset);
        let wrapped_line_len = wrapped_info.wrapped_line.chars().count();
        let cursor_col = col_in_wrapped.min(wrapped_line_len);
        
        let x = field_area.x + 1 + (cursor_col.min((field_area.width.saturating_sub(2)) as usize) as u16);
        let y = field_area.y + 1 + (visible_wrapped_idx.min((field_area.height.saturating_sub(2)) as usize) as u16);
        
        if x < field_area.x + field_area.width && y < field_area.y + field_area.height {
            Some((x, y))
        } else {
            None
        }
    } else {
        // Single-line fields
        let cursor_col = editor.cursor_col;
        let x = field_area.x + 1 + (cursor_col.min((field_area.width.saturating_sub(2)) as usize) as u16);
        let y = field_area.y + 1;
        
        if x < field_area.x + field_area.width && y < field_area.y + field_area.height {
            Some((x, y))
        } else {
            None
        }
    }
}

fn get_cursor_position_for_note_field(_area: Rect, form: &NoteForm, field_areas: &[Rect]) -> Option<(u16, u16)> {
    let editor = match form.current_field {
        NoteField::Title => &form.title,
        NoteField::Tags => &form.tags,
        NoteField::Notebook => return None, // Notebook field doesn't use cursor
        NoteField::Content => &form.content,
    };

    let field_index = match form.current_field {
        NoteField::Title => 0,
        NoteField::Tags => 1,
        NoteField::Notebook => return None,
        NoteField::Content => 3,
    };

    if field_index >= field_areas.len() {
        return None;
    }

    let field_area = field_areas[field_index];
    
    // For multi-line fields (Content), calculate cursor position accounting for wrapping
    if form.current_field == NoteField::Content {
        let content_width = field_area.width.saturating_sub(2) as usize;
        let content_height = field_area.height.saturating_sub(2) as usize;
        
        // Build wrapped lines
        let all_wrapped = build_all_wrapped_lines(&editor.lines, content_width);
        if all_wrapped.is_empty() {
            return Some((field_area.x + 1, field_area.y + 1));
        }
        
        // Find which wrapped line contains the cursor
        let cursor_wrapped_idx = find_cursor_wrapped_line(&all_wrapped, editor.cursor_line, editor.cursor_col);
        
        // Calculate scroll offset to ensure cursor is visible
        let scroll_start = if cursor_wrapped_idx < content_height {
            0
        } else {
            cursor_wrapped_idx.saturating_sub(content_height - 1)
        };
        
        // Calculate visible wrapped line index
        let visible_wrapped_idx = cursor_wrapped_idx.saturating_sub(scroll_start);
        if visible_wrapped_idx >= content_height {
            return None; // Cursor is not visible
        }
        
        // Bounds check: ensure cursor_wrapped_idx is within all_wrapped array
        if cursor_wrapped_idx >= all_wrapped.len() {
            return None;
        }
        
        // Calculate column within the wrapped line
        let wrapped_info = &all_wrapped[cursor_wrapped_idx];
        let col_in_wrapped = editor.cursor_col.saturating_sub(wrapped_info.char_offset);
        let wrapped_line_len = wrapped_info.wrapped_line.chars().count();
        let cursor_col = col_in_wrapped.min(wrapped_line_len);
        
        let x = field_area.x + 1 + (cursor_col.min((field_area.width.saturating_sub(2)) as usize) as u16);
        let y = field_area.y + 1 + (visible_wrapped_idx.min((field_area.height.saturating_sub(2)) as usize) as u16);
        
        if x < field_area.x + field_area.width && y < field_area.y + field_area.height {
            Some((x, y))
        } else {
            None
        }
    } else {
        // Single-line fields
        let cursor_col = editor.cursor_col;
        let x = field_area.x + 1 + (cursor_col.min((field_area.width.saturating_sub(2)) as usize) as u16);
        let y = field_area.y + 1;
        
        if x < field_area.x + field_area.width && y < field_area.y + field_area.height {
            Some((x, y))
        } else {
            None
        }
    }
}

fn get_cursor_position_for_journal_field(_area: Rect, form: &JournalForm, field_areas: &[Rect]) -> Option<(u16, u16)> {
    let editor = match form.current_field {
        JournalField::Date => &form.date,
        JournalField::Title => &form.title,
        JournalField::Tags => &form.tags,
        JournalField::Notebook => return None, // Notebook field doesn't use cursor
        JournalField::Content => &form.content,
    };

    let field_index = match form.current_field {
        JournalField::Date => 0,
        JournalField::Title => 1,
        JournalField::Tags => 2,
        JournalField::Notebook => return None,
        JournalField::Content => 4,
    };

    if field_index >= field_areas.len() {
        return None;
    }

    let field_area = field_areas[field_index];
    
    // For multi-line fields (Content), calculate cursor position accounting for wrapping
    if form.current_field == JournalField::Content {
        let content_width = field_area.width.saturating_sub(2) as usize;
        let content_height = field_area.height.saturating_sub(2) as usize;
        
        // Build wrapped lines
        let all_wrapped = build_all_wrapped_lines(&editor.lines, content_width);
        if all_wrapped.is_empty() {
            return Some((field_area.x + 1, field_area.y + 1));
        }
        
        // Find which wrapped line contains the cursor
        let cursor_wrapped_idx = find_cursor_wrapped_line(&all_wrapped, editor.cursor_line, editor.cursor_col);
        
        // Calculate scroll offset to ensure cursor is visible
        let scroll_start = if cursor_wrapped_idx < content_height {
            0
        } else {
            cursor_wrapped_idx.saturating_sub(content_height - 1)
        };
        
        // Calculate visible wrapped line index
        let visible_wrapped_idx = cursor_wrapped_idx.saturating_sub(scroll_start);
        if visible_wrapped_idx >= content_height {
            return None; // Cursor is not visible
        }
        
        // Bounds check: ensure cursor_wrapped_idx is within all_wrapped array
        if cursor_wrapped_idx >= all_wrapped.len() {
            return None;
        }
        
        // Calculate column within the wrapped line
        let wrapped_info = &all_wrapped[cursor_wrapped_idx];
        let col_in_wrapped = editor.cursor_col.saturating_sub(wrapped_info.char_offset);
        let wrapped_line_len = wrapped_info.wrapped_line.chars().count();
        let cursor_col = col_in_wrapped.min(wrapped_line_len);
        
        let x = field_area.x + 1 + (cursor_col.min((field_area.width.saturating_sub(2)) as usize) as u16);
        let y = field_area.y + 1 + (visible_wrapped_idx.min((field_area.height.saturating_sub(2)) as usize) as u16);
        
        if x < field_area.x + field_area.width && y < field_area.y + field_area.height {
            Some((x, y))
        } else {
            None
        }
    } else {
        // Single-line fields
        let cursor_col = editor.cursor_col;
        let x = field_area.x + 1 + (cursor_col.min((field_area.width.saturating_sub(2)) as usize) as u16);
        let y = field_area.y + 1;
        
        if x < field_area.x + field_area.width && y < field_area.y + field_area.height {
            Some((x, y))
        } else {
            None
        }
    }
}

