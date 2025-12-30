use ratatui::layout::Rect;
use std::cmp;

#[derive(Clone, Debug)]
pub enum EditOperation {
    InsertChar { line: usize, col: usize, ch: char },
    DeleteChar { line: usize, col: usize, ch: char },
    InsertNewline { line: usize, col: usize },
    DeleteNewline { line: usize, col: usize, next_line: String },
}

#[derive(Debug, Clone)]
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,      // Vertical scroll (line offset)
    pub scroll_col: usize,         // Horizontal scroll (column offset)
    pub selection_start: Option<(usize, usize)>,  // (line, col) - None if no selection
    pub undo_stack: Vec<EditOperation>,
    pub redo_stack: Vec<EditOperation>,
    pub max_history: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            scroll_col: 0,
            selection_start: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    /// Ensure cursor_line is within valid bounds
    fn ensure_cursor_valid(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        if self.cursor_line >= self.lines.len() {
            self.cursor_line = self.lines.len().saturating_sub(1);
        }
    }

    pub fn from_string(content: String) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        let cursor_line = lines.len().saturating_sub(1);
        // Use chars().count() for UTF-8 safe character count, not byte count
        let cursor_col = lines.last().map(|l| l.chars().count()).unwrap_or(0);
        Self {
            lines,
            cursor_line,
            cursor_col,
            scroll_offset: 0,
            scroll_col: 0,
            selection_start: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        // Clear selection if inserting
        if self.has_selection() {
            self.delete_selection();
        }
        
        // Record operation for undo
        let op = EditOperation::InsertChar {
            line: self.cursor_line,
            col: self.cursor_col,
            ch,
        };
        
        if ch == '\n' {
            self.insert_newline();
        } else {
            self.ensure_cursor_valid();
            let line = self.lines.get_mut(self.cursor_line)
                .expect("cursor_line should be valid after ensure_cursor_valid");
            let col = cmp::min(self.cursor_col, line.chars().count());
            let mut chars: Vec<char> = line.chars().collect();
            chars.insert(col, ch);
            *line = chars.into_iter().collect();
            self.cursor_col += 1;
        }
        
        // Add to undo stack and clear redo stack
        self.add_to_undo(op);
    }

    pub fn delete_char(&mut self) {
        // If there's a selection, delete it instead
        if self.has_selection() {
            self.delete_selection();
            return;
        }
        
        self.ensure_cursor_valid();
        let op = if self.cursor_col > 0 {
            // Delete character before cursor
            let line = self.lines.get(self.cursor_line)
                .expect("cursor_line should be valid after ensure_cursor_valid");
            let col = cmp::min(self.cursor_col, line.chars().count());
            if col > 0 {
                let chars: Vec<char> = line.chars().collect();
                let ch = chars[col - 1];
                Some(EditOperation::DeleteChar {
                    line: self.cursor_line,
                    col: col - 1,
                    ch,
                })
            } else {
                None
            }
        } else if self.cursor_line > 0 && self.cursor_line < self.lines.len() {
            // Merge with previous line
            let current_line = self.lines.get(self.cursor_line)
                .cloned()
                .unwrap_or_default();
            let prev_line_len = self.lines.get(self.cursor_line - 1)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            Some(EditOperation::DeleteNewline {
                line: self.cursor_line - 1,
                col: prev_line_len,
                next_line: current_line,
            })
        } else {
            None
        };
        
        if let Some(op) = op {
            // Perform the deletion
            if self.cursor_col > 0 {
                let line = self.lines.get_mut(self.cursor_line)
                    .expect("cursor_line should be valid after ensure_cursor_valid");
                let col = cmp::min(self.cursor_col, line.chars().count());
                if col > 0 {
                    let mut chars: Vec<char> = line.chars().collect();
                    chars.remove(col - 1);
                    *line = chars.into_iter().collect();
                    self.cursor_col -= 1;
                }
            } else if self.cursor_line > 0 && self.cursor_line < self.lines.len() {
                let current_line = self.lines.remove(self.cursor_line);
                self.cursor_line -= 1;
                let prev_line = self.lines.get_mut(self.cursor_line)
                    .expect("cursor_line should be valid after decrement");
                self.cursor_col = prev_line.chars().count();
                prev_line.push_str(&current_line);
            }
            
            self.add_to_undo(op);
        }
    }

    pub fn insert_newline(&mut self) {
        // Clear selection if inserting
        if self.has_selection() {
            self.delete_selection();
        }
        
        // Record operation for undo
        let op = EditOperation::InsertNewline {
            line: self.cursor_line,
            col: self.cursor_col,
        };
        
        self.ensure_cursor_valid();
        let line = self.lines.get_mut(self.cursor_line)
            .expect("cursor_line should be valid after ensure_cursor_valid");
        let col = cmp::min(self.cursor_col, line.chars().count());
        let mut chars: Vec<char> = line.chars().collect();
        let remainder: String = chars.split_off(col).into_iter().collect();
        *line = chars.into_iter().collect();
        self.lines.insert(self.cursor_line + 1, remainder);
        self.cursor_line += 1;
        self.cursor_col = 0;
        
        self.add_to_undo(op);
    }

    pub fn move_cursor_up(&mut self, extend_selection: bool) {
        if self.cursor_line > 0 {
            if !extend_selection && !self.has_selection() {
                self.clear_selection();
            } else if extend_selection && self.selection_start.is_none() {
                self.start_selection();
            }
            self.cursor_line -= 1;
            let line_len = self.lines.get(self.cursor_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            self.cursor_col = cmp::min(self.cursor_col, line_len);
        } else if !extend_selection {
            self.clear_selection();
        }
    }

    pub fn move_cursor_down(&mut self, extend_selection: bool) {
        if self.cursor_line < self.lines.len().saturating_sub(1) {
            if !extend_selection && !self.has_selection() {
                self.clear_selection();
            } else if extend_selection && self.selection_start.is_none() {
                self.start_selection();
            }
            self.cursor_line += 1;
            let line_len = self.lines.get(self.cursor_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
            self.cursor_col = cmp::min(self.cursor_col, line_len);
        } else if !extend_selection {
            self.clear_selection();
        }
    }

    pub fn move_cursor_left(&mut self, extend_selection: bool) {
        if !extend_selection && !self.has_selection() {
            self.clear_selection();
        } else if extend_selection && self.selection_start.is_none() {
            self.start_selection();
        }
        
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines.get(self.cursor_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
        }
    }

    pub fn move_cursor_right(&mut self, extend_selection: bool) {
        if !extend_selection && !self.has_selection() {
            self.clear_selection();
        } else if extend_selection && self.selection_start.is_none() {
            self.start_selection();
        }
        
        let line_len = self.lines.get(self.cursor_line)
            .map(|l| l.chars().count())
            .unwrap_or(0);
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line < self.lines.len().saturating_sub(1) {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_cursor_home(&mut self, extend_selection: bool) {
        if !extend_selection && !self.has_selection() {
            self.clear_selection();
        } else if extend_selection && self.selection_start.is_none() {
            self.start_selection();
        }
        self.cursor_col = 0;
    }

    pub fn move_cursor_end(&mut self, extend_selection: bool) {
        if !extend_selection && !self.has_selection() {
            self.clear_selection();
        } else if extend_selection && self.selection_start.is_none() {
            self.start_selection();
        }
        if let Some(line) = self.lines.get(self.cursor_line) {
            self.cursor_col = line.chars().count();
        }
    }

    // Selection methods
    pub fn start_selection(&mut self) {
        self.selection_start = Some((self.cursor_line, self.cursor_col));
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn select_all(&mut self) {
        if self.lines.is_empty() {
            self.selection_start = Some((0, 0));
            self.cursor_line = 0;
            self.cursor_col = 0;
        } else {
            // Start selection at beginning (0, 0)
            self.selection_start = Some((0, 0));
            // Move cursor to end of last line
            self.cursor_line = self.lines.len().saturating_sub(1);
            self.cursor_col = self.lines.get(self.cursor_line)
                .map(|l| l.chars().count())
                .unwrap_or(0);
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() && 
        self.selection_start != Some((self.cursor_line, self.cursor_col))
    }

    pub fn get_selection_bounds(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some((start_line, start_col)) = self.selection_start {
            let (end_line, end_col) = (self.cursor_line, self.cursor_col);
            
            // Normalize: ensure start is before end
            if start_line < end_line || (start_line == end_line && start_col <= end_col) {
                Some(((start_line, start_col), (end_line, end_col)))
            } else {
                Some(((end_line, end_col), (start_line, start_col)))
            }
        } else {
            None
        }
    }

    pub fn get_selected_text(&self) -> String {
        if let Some(((start_line, start_col), (end_line, end_col))) = self.get_selection_bounds() {
            if start_line == end_line {
                // Single line selection
                if let Some(line) = self.lines.get(start_line) {
                    let chars: Vec<char> = line.chars().collect();
                    if start_col < chars.len() && end_col <= chars.len() {
                        chars[start_col..end_col].iter().collect()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                // Multi-line selection
                let mut result = String::new();
                for line_idx in start_line..=end_line {
                    let line = match self.lines.get(line_idx) {
                        Some(l) => l,
                        None => continue,
                    };
                    let chars: Vec<char> = line.chars().collect();
                    if line_idx == start_line {
                        // First line: from start_col to end
                        if start_col < chars.len() {
                            result.push_str(&chars[start_col..].iter().collect::<String>());
                        }
                        result.push('\n');
                    } else if line_idx == end_line {
                        // Last line: from start to end_col
                        if end_col <= chars.len() {
                            result.push_str(&chars[..end_col].iter().collect::<String>());
                        }
                    } else {
                        // Middle lines: entire line
                        result.push_str(line);
                        result.push('\n');
                    }
                }
                result
            }
        } else {
            String::new()
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(((start_line, start_col), (end_line, end_col))) = self.get_selection_bounds() {
            // Record operation for undo (simplified - could be more detailed)
            // For now, we'll treat selection deletion as a single operation
            
            if start_line == end_line {
                // Single line deletion
                if let Some(line) = self.lines.get_mut(start_line) {
                    let mut chars: Vec<char> = line.chars().collect();
                    if start_col < chars.len() && end_col <= chars.len() {
                        chars.drain(start_col..end_col);
                        *line = chars.into_iter().collect();
                    }
                }
                self.cursor_line = start_line;
                self.cursor_col = start_col;
            } else {
                // Multi-line deletion
                // Get the parts to keep before borrowing
                let first_chars: Vec<char> = self.lines.get(start_line)
                    .map(|l| l.chars().collect())
                    .unwrap_or_default();
                let last_chars: Vec<char> = self.lines.get(end_line)
                    .map(|l| l.chars().collect())
                    .unwrap_or_default();
                
                let first_part: String = if start_col < first_chars.len() {
                    first_chars[..start_col].iter().collect()
                } else {
                    String::new()
                };
                
                let last_part: String = if end_col <= last_chars.len() {
                    last_chars[end_col..].iter().collect()
                } else {
                    String::new()
                };
                
                // Reconstruct first line
                if let Some(line) = self.lines.get_mut(start_line) {
                    *line = format!("{}{}", first_part, last_part);
                }
                
                // Remove middle lines
                self.lines.drain(start_line + 1..=end_line);
                
                self.cursor_line = start_line;
                self.cursor_col = start_col;
            }
            
            self.clear_selection();
        }
    }

    // Undo/Redo methods
    fn add_to_undo(&mut self, op: EditOperation) {
        self.undo_stack.push(op);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        // Clear redo stack when new operation is performed
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            match op {
                EditOperation::InsertChar { line, col, ch } => {
                    // Undo insert: delete the character
                    if let Some(line_str) = self.lines.get_mut(line) {
                        let mut chars: Vec<char> = line_str.chars().collect();
                        if col < chars.len() && chars[col] == ch {
                            chars.remove(col);
                            *line_str = chars.into_iter().collect();
                            self.cursor_line = line;
                            self.cursor_col = col;
                            self.clear_selection();
                        }
                    }
                }
                EditOperation::DeleteChar { line, col, ch } => {
                    // Undo delete: insert the character back
                    if let Some(line_str) = self.lines.get_mut(line) {
                        let mut chars: Vec<char> = line_str.chars().collect();
                        if col <= chars.len() {
                            chars.insert(col, ch);
                            *line_str = chars.into_iter().collect();
                            self.cursor_line = line;
                            self.cursor_col = col + 1;
                            self.clear_selection();
                        }
                    }
                }
                EditOperation::InsertNewline { line, col } => {
                    // Undo newline: merge lines back
                    if line + 1 < self.lines.len() {
                        let next_line = self.lines.remove(line + 1);
                        if let Some(line_str) = self.lines.get_mut(line) {
                            let mut chars: Vec<char> = line_str.chars().collect();
                            if col <= chars.len() {
                                let remainder: String = chars.split_off(col).into_iter().collect();
                                *line_str = chars.into_iter().collect();
                                line_str.push_str(&remainder);
                                line_str.push_str(&next_line);
                                self.cursor_line = line;
                                self.cursor_col = col;
                                self.clear_selection();
                            }
                        }
                    }
                }
                EditOperation::DeleteNewline { line, col, next_line } => {
                    // Undo delete newline: split line back
                    if let Some(line_str) = self.lines.get_mut(line) {
                        let mut chars: Vec<char> = line_str.chars().collect();
                        if col <= chars.len() {
                            let remainder: String = chars.split_off(col).into_iter().collect();
                            *line_str = chars.into_iter().collect();
                            self.lines.insert(line + 1, format!("{}{}", remainder, next_line));
                            self.cursor_line = line;
                            self.cursor_col = col;
                            self.clear_selection();
                        }
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        // For redo, we'd need to store inverse operations
        // For now, redo is not fully implemented - would need to store operations differently
        // This is a simplified version
        false
    }

    pub fn move_cursor_word_left(&mut self, extend_selection: bool) {
        if !extend_selection && !self.has_selection() {
            self.clear_selection();
        } else if extend_selection && self.selection_start.is_none() {
            self.start_selection();
        }
        
        if self.cursor_col == 0 {
            // At start of line, move to end of previous line
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self.lines.get(self.cursor_line)
                    .map(|l| l.chars().count())
                    .unwrap_or(0);
            }
            return;
        }

        let line = match self.lines.get(self.cursor_line) {
            Some(l) => l,
            None => return,
        };
        let chars: Vec<char> = line.chars().collect();
        let mut pos = cmp::min(self.cursor_col, chars.len());

        // Skip whitespace to the left
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip word characters to the left
        while pos > 0 && is_word_char(chars[pos - 1]) {
            pos -= 1;
        }

        self.cursor_col = pos;
    }

    pub fn move_cursor_word_right(&mut self, extend_selection: bool) {
        if !extend_selection && !self.has_selection() {
            self.clear_selection();
        } else if extend_selection && self.selection_start.is_none() {
            self.start_selection();
        }
        
        if self.lines.is_empty() || self.cursor_line >= self.lines.len() {
            return;
        }

        let line_len = self.lines.get(self.cursor_line)
            .map(|l| l.chars().count())
            .unwrap_or(0);

        if self.cursor_col >= line_len {
            // At end of line, move to start of next line
            if self.cursor_line < self.lines.len().saturating_sub(1) {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
            return;
        }

        let line = match self.lines.get(self.cursor_line) {
            Some(l) => l,
            None => return,
        };
        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            self.cursor_col = 0;
            return;
        }

        let mut pos = self.cursor_col;

        // Skip word characters to the right
        while pos < chars.len() && is_word_char(chars[pos]) {
            pos += 1;
        }

        // Skip whitespace to the right
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.cursor_col = pos;
    }

    pub fn get_visible_lines(&self, viewport_height: usize, viewport_width: usize) -> (usize, Vec<String>) {
        let start = cmp::min(self.scroll_offset, self.lines.len());
        let end = cmp::min(start + viewport_height, self.lines.len());
        
        // Calculate effective width (accounting for borders)
        let effective_width = viewport_width.saturating_sub(2);
        
        // Apply horizontal scrolling to each visible line and truncate to viewport width
        // Safe to use slice here because both start and end are bounded by self.lines.len()
        let visible: Vec<String> = self.lines[start..end]
            .iter()
            .map(|line| {
                let chars: Vec<char> = line.chars().collect();
                if self.scroll_col >= chars.len() {
                    String::new() // Line is scrolled past
                } else {
                    let start_idx = self.scroll_col;
                    let end_idx = cmp::min(start_idx + effective_width, chars.len());
                    chars[start_idx..end_idx].iter().collect()
                }
            })
            .collect();
        
        (start, visible)
    }

    pub fn update_scroll(&mut self, viewport_height: usize) {
        // Ensure cursor is visible vertically
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor_line.saturating_sub(viewport_height - 1);
        }
    }

    pub fn update_horizontal_scroll(&mut self, viewport_width: usize) {
        // Ensure cursor is visible horizontally
        // viewport_width should account for borders (width - 2)
        let effective_width = viewport_width.saturating_sub(2);
        
        if self.cursor_col < self.scroll_col {
            // Cursor is to the left of visible area
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + effective_width {
            // Cursor is to the right of visible area
            self.scroll_col = self.cursor_col.saturating_sub(effective_width - 1);
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn get_cursor_screen_pos(&self, area: Rect, viewport_height: usize) -> Option<(u16, u16)> {
        let visible_start = self.scroll_offset;
        if self.cursor_line < visible_start || self.cursor_line >= visible_start + viewport_height {
            return None;
        }
        let line_y = (self.cursor_line - visible_start) as u16;
        if line_y >= area.height - 2 {
            return None;
        }
        
        // Bounds check: ensure cursor_line is within lines array
        let line = match self.lines.get(self.cursor_line) {
            Some(l) => l,
            None => return None,
        };
        
        // Calculate x position accounting for horizontal scroll
        let col = cmp::min(self.cursor_col, line.chars().count());
        
        // Account for horizontal scroll offset
        let visible_col = if col >= self.scroll_col {
            col - self.scroll_col
        } else {
            return None; // Cursor is to the left of visible area
        };
        
        let max_x = area.width.saturating_sub(2); // Account for borders
        if visible_col >= max_x as usize {
            return None; // Cursor is to the right of visible area
        }
        
        let screen_x = area.x + 1 + visible_col as u16;
        let screen_y = area.y + 1 + line_y;
        
        // Ensure position is within terminal bounds
        if screen_x >= area.x + area.width || screen_y >= area.y + area.height {
            return None;
        }
        
        Some((screen_x, screen_y))
    }
}


fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}


