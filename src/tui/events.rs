use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
// Cursor positioning is now handled by ratatui's Frame::set_cursor_position() inside render
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, size as terminal_size};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use crate::tui::App;
use crate::tui::error::TuiError;
use crate::tui::widgets::editor::Editor;
use crate::tui::widgets::item_view::get_content_string;
use crate::utils::parse_key_binding;

/// Guard that ensures terminal state is restored even on panic
/// This is critical for TUI applications - if the terminal is left in raw mode
/// or alternate screen, the user's terminal will be unusable.
struct TerminalGuard {
    /// Track if we successfully entered raw mode
    raw_mode_enabled: bool,
    /// Track if we successfully entered alternate screen
    alternate_screen_enabled: bool,
}

impl TerminalGuard {
    /// Initialize terminal state and return a guard
    /// The guard will restore terminal state when dropped (even on panic)
    fn new() -> Result<Self, TuiError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        Ok(Self {
            raw_mode_enabled: true,
            alternate_screen_enabled: true,
        })
    }
    
    /// Manually restore terminal state (called on normal exit)
    /// After calling this, the guard will do nothing on drop
    fn restore(&mut self) -> Result<(), TuiError> {
        if self.raw_mode_enabled {
            disable_raw_mode()?;
            self.raw_mode_enabled = false;
        }
        if self.alternate_screen_enabled {
            execute!(io::stdout(), LeaveAlternateScreen)?;
            self.alternate_screen_enabled = false;
        }
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Restore terminal state even if we panic
        // Ignore errors in drop - we're already in a cleanup path
        if self.raw_mode_enabled {
            let _ = disable_raw_mode();
        }
        if self.alternate_screen_enabled {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        }
    }
}

pub fn run_event_loop(mut app: App) -> Result<(), TuiError> {
    // Check terminal size before entering alternate screen
    // This allows us to show a helpful error message in the normal terminal
    let (width, height) = terminal_size()
        .map_err(|e| TuiError::IoError(e))?;
    
    use crate::tui::layout::Layout;
    let min_width_with_border = Layout::MIN_WIDTH + 2; // +2 for borders
    let min_height_with_border = Layout::MIN_HEIGHT + 2; // +2 for borders
    
    if width < min_width_with_border || height < min_height_with_border {
        return Err(TuiError::RenderError(format!(
            "Terminal size too small. Current: {}x{}, Minimum required: {}x{}. Please resize your terminal window.",
            width, height, min_width_with_border, min_height_with_border
        )));
    }
    
    // Setup terminal with guard to ensure restoration on panic
    let mut guard = TerminalGuard::new()?;
    
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        // Check if status message should be auto-cleared
        app.check_status_message_timeout();

        // Update form editor scroll before rendering
        if app.ui.mode == crate::tui::app::Mode::Create {
            // Extract values before borrowing editor
            let sidebar_width_percent = app.config.sidebar_width_percent;
            let sidebar_collapsed = app.ui.sidebar_state == crate::tui::app::SidebarState::Collapsed;
            
            // Check if current field is a multi-line field and get form type
            let (is_multi_line, form_type) = if let Some(ref form) = app.form.create_form {
                use crate::tui::widgets::form::FormType;
                let is_multi = match form {
                    crate::tui::app::CreateForm::Task(task_form) => {
                        task_form.current_field == crate::tui::app::TaskField::Description
                    }
                    crate::tui::app::CreateForm::Note(note_form) => {
                        note_form.current_field == crate::tui::app::NoteField::Content
                    }
                    crate::tui::app::CreateForm::Journal(journal_form) => {
                        journal_form.current_field == crate::tui::app::JournalField::Content
                    }
                };
                (is_multi, Some(FormType::from(form)))
            } else {
                (false, None)
            };
            
            if let Some(ref mut editor) = app.get_current_form_editor() {
                use crate::tui::layout::Layout;
                let size = terminal.size()?;
                use ratatui::layout::Rect;
                let rect = Rect::new(0, 0, size.width, size.height);
                let layout = Layout::calculate(
                    rect,
                    sidebar_width_percent,
                    sidebar_collapsed,
                );
                
                // Calculate viewport height for multi-line fields
                let viewport_height = if is_multi_line {
                    // Calculate the actual field area height for the multi-line field
                    if let Some(form_type) = form_type {
                        use crate::tui::widgets::form::{calculate_multi_line_field_height, calculate_field_viewport_height};
                        let field_height = calculate_multi_line_field_height(layout.main_area.height, form_type);
                        calculate_field_viewport_height(field_height)
                    } else {
                        (layout.main_area.height - 2) as usize
                    }
                } else {
                    // For single-line fields, use full height (doesn't matter)
                    (layout.main_area.height - 2) as usize
                };
                
                let viewport_width = layout.main_area.width as usize;
                editor.update_scroll(viewport_height);
                editor.update_horizontal_scroll(viewport_width);
            }
        }

        // Render
        // for edit mode, which handles cursor positioning atomically with rendering
        // Get terminal size explicitly to ensure compatibility across different terminals
        let terminal_size = terminal.size()?;
        use ratatui::layout::Rect;
        let terminal_rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);
        terminal.draw(|f| {
            use crate::tui::layout::Layout;
            // Use explicit terminal size instead of f.area() for better compatibility
            // f.area() should match, but some terminals (like Ghostty) may report differently
            let layout = Layout::calculate(
                terminal_rect,
                app.config.sidebar_width_percent,
                app.ui.sidebar_state == crate::tui::app::SidebarState::Collapsed,
            );
            crate::tui::render::render(f, &mut app, &layout);
        })?;

        // Handle events - only process Press events to avoid duplicate processing on Windows
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) => {
                    // Only process Press events (ignore Release events to prevent double-processing on Windows)
                    if key_event.kind == KeyEventKind::Press {
                        if handle_key_event(&mut app, key_event)? {
                            break; // Quit requested
                        }
                    }
                }
                Event::Resize(_width, _height) => {
                    // Force immediate redraw on resize to ensure layout updates correctly
                    // Some terminals (like Ghostty) may need explicit handling
                    // The terminal.size() will be refreshed on next draw
                }
                _ => {
                    // Ignore other event types (mouse, etc.)
                }
            }
        }
    }

    // Restore terminal state explicitly (guard will also restore on drop, but this is cleaner)
    guard.restore()?;

    Ok(())
}

fn handle_delete_confirmation_modal(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    match key_event.code {
        KeyCode::Up => {
            // Move selection up (wrapping from Archive to Cancel)
            if app.modals.delete_modal_selection == 0 {
                app.modals.delete_modal_selection = 2; // Wrap to Cancel
            } else {
                app.modals.delete_modal_selection -= 1;
            }
            return Ok(false);
        }
        KeyCode::Down => {
            // Move selection down (wrapping from Cancel to Archive)
            if app.modals.delete_modal_selection == 2 {
                app.modals.delete_modal_selection = 0; // Wrap to Archive
            } else {
                app.modals.delete_modal_selection += 1;
            }
            return Ok(false);
        }
        KeyCode::Enter => {
            // Execute selected action
            if app.modals.delete_modal_selection == 2 {
                // Cancel - just close modal
                app.modals.delete_confirmation = None;
                return Ok(false);
            }
            
            if let Some(ref item) = app.modals.delete_confirmation {
                match item {
                    crate::tui::app::SelectedItem::Task(task) => {
                        if let Some(id) = task.id {
                            if app.modals.delete_modal_selection == 0 {
                                // Archive
                                if let Err(e) = app.database.archive_task(id) {
                                    app.set_status_message(format!("Failed to archive task: {}", e));
                                } else {
                                    if let Err(e) = app.load_data() {
                                        app.set_status_message(format!("Failed to reload data: {}", e));
                                    } else {
                                        app.adjust_selected_index();
                                        app.select_current_item();
                                        app.set_status_message("Task archived".to_string());
                                    }
                                }
                            } else if app.modals.delete_modal_selection == 1 {
                                // Delete
                                if let Err(e) = app.database.delete_task(id) {
                                    app.set_status_message(format!("Failed to delete task: {}", e));
                                } else {
                                    if let Err(e) = app.load_data() {
                                        app.set_status_message(format!("Failed to reload data: {}", e));
                                    } else {
                                        app.adjust_selected_index();
                                        app.select_current_item();
                                        app.set_status_message("Task deleted".to_string());
                                    }
                                }
                            }
                        } else {
                            app.set_status_message("Task has no ID".to_string());
                        }
                    }
                    crate::tui::app::SelectedItem::Note(note) => {
                        if let Some(id) = note.id {
                            if app.modals.delete_modal_selection == 0 {
                                // Archive
                                if let Err(e) = app.database.archive_note(id) {
                                    app.set_status_message(format!("Failed to archive note: {}", e));
                                } else {
                                    if let Err(e) = app.load_data() {
                                        app.set_status_message(format!("Failed to reload data: {}", e));
                                    } else {
                                        app.adjust_selected_index();
                                        app.select_current_item();
                                        app.set_status_message("Note archived".to_string());
                                    }
                                }
                            } else if app.modals.delete_modal_selection == 1 {
                                // Delete
                                if let Err(e) = app.database.delete_note(id) {
                                    app.set_status_message(format!("Failed to delete note: {}", e));
                                } else {
                                    if let Err(e) = app.load_data() {
                                        app.set_status_message(format!("Failed to reload data: {}", e));
                                    } else {
                                        app.adjust_selected_index();
                                        app.select_current_item();
                                        app.set_status_message("Note deleted".to_string());
                                    }
                                }
                            }
                        } else {
                            app.set_status_message("Note has no ID".to_string());
                        }
                    }
                    crate::tui::app::SelectedItem::Journal(journal) => {
                        if let Some(id) = journal.id {
                            if app.modals.delete_modal_selection == 0 {
                                // Archive
                                if let Err(e) = app.database.archive_journal(id) {
                                    app.set_status_message(format!("Failed to archive journal entry: {}", e));
                                } else {
                                    if let Err(e) = app.load_data() {
                                        app.set_status_message(format!("Failed to reload data: {}", e));
                                    } else {
                                        app.adjust_selected_index();
                                        app.select_current_item();
                                        app.set_status_message("Journal archived".to_string());
                                    }
                                }
                            } else if app.modals.delete_modal_selection == 1 {
                                // Delete
                                if let Err(e) = app.database.delete_journal(id) {
                                    app.set_status_message(format!("Failed to delete journal entry: {}", e));
                                } else {
                                    if let Err(e) = app.load_data() {
                                        app.set_status_message(format!("Failed to reload data: {}", e));
                                    } else {
                                        app.adjust_selected_index();
                                        app.select_current_item();
                                        app.set_status_message("Journal deleted".to_string());
                                    }
                                }
                            }
                        } else {
                            app.set_status_message("Journal entry has no ID".to_string());
                        }
                    }
                }
            }
            app.modals.delete_confirmation = None;
            return Ok(false);
        }
        KeyCode::Esc => {
            // Cancel deletion
            app.modals.delete_confirmation = None;
            return Ok(false);
        }
        _ => {
            // Ignore all other keys when confirmation modal is shown
            return Ok(false);
        }
    }
}

fn handle_markdown_help_mode(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    match key_event.code {
        KeyCode::Esc => {
            app.exit_markdown_help_mode();
            return Ok(false);
        }
        KeyCode::Up => {
            app.scroll_markdown_help_example_up();
            app.scroll_markdown_help_rendered_up();
            return Ok(false);
        }
        KeyCode::Down => {
            app.scroll_markdown_help_example_down();
            app.scroll_markdown_help_rendered_down();
            return Ok(false);
        }
        KeyCode::PageUp => {
            if let Ok((_, height)) = terminal_size() {
                // Calculate viewport height for markdown help (85% of screen height, minus borders)
                let popup_height = (height as f32 * 0.85) as u16;
                let inner_height = popup_height.saturating_sub(2); // Outer block borders
                let panel_height = inner_height.saturating_sub(2); // Panel borders
                let viewport_height = panel_height as usize;
                
                // Get example text to calculate total lines
                use crate::tui::widgets::markdown_help::get_example_markdown;
                let example_text = get_example_markdown();
                let _example_total_lines = example_text.lines().count();
                
                // Scroll both panels together
                app.scroll_markdown_help_example_page_up(viewport_height);
                app.scroll_markdown_help_rendered_page_up(viewport_height);
            }
            return Ok(false);
        }
        KeyCode::PageDown => {
            if let Ok((_, height)) = terminal_size() {
                // Calculate viewport height for markdown help
                let popup_height = (height as f32 * 0.85) as u16;
                let inner_height = popup_height.saturating_sub(2);
                let panel_height = inner_height.saturating_sub(2);
                let viewport_height = panel_height as usize;
                
                // Get example text to calculate total lines
                use crate::tui::widgets::markdown_help::get_example_markdown;
                let example_text = get_example_markdown();
                let example_total_lines = example_text.lines().count();
                
                // Use example lines as approximation for rendered lines
                app.scroll_markdown_help_example_page_down(viewport_height, example_total_lines);
                app.scroll_markdown_help_rendered_page_down(viewport_height, example_total_lines);
            }
            return Ok(false);
        }
        KeyCode::Home => {
            app.ui.markdown_help_example_scroll = 0;
            app.ui.markdown_help_rendered_scroll = 0;
            return Ok(false);
        }
        KeyCode::End => {
            if let Ok((_, height)) = terminal_size() {
                let popup_height = (height as f32 * 0.85) as u16;
                let inner_height = popup_height.saturating_sub(2);
                let panel_height = inner_height.saturating_sub(2);
                let viewport_height = panel_height as usize;
                
                use crate::tui::widgets::markdown_help::get_example_markdown;
                let example_text = get_example_markdown();
                let example_total_lines = example_text.lines().count();
                
                // Use example lines as approximation for rendered lines
                app.ui.markdown_help_example_scroll = example_total_lines.saturating_sub(viewport_height);
                app.ui.markdown_help_rendered_scroll = example_total_lines.saturating_sub(viewport_height);
            }
            return Ok(false);
        }
        _ => {
            // Check if help binding is pressed again to toggle off
            let help_binding = parse_key_binding(&app.config.key_bindings.help)
                .map_err(|e| TuiError::KeyBindingError(e))?;
            if matches_key_event(key_event, &help_binding) {
                app.exit_markdown_help_mode();
                return Ok(false);
            }
            // Ignore all other keys in markdown help mode
            return Ok(false);
        }
    }
}

fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    // Handle delete confirmation modal first (before other modes)
    if app.modals.delete_confirmation.is_some() {
        return handle_delete_confirmation_modal(app, key_event);
    }

    // Handle markdown help mode first (before create mode)
    if app.ui.mode == crate::tui::app::Mode::MarkdownHelp {
        return handle_markdown_help_mode(app, key_event);
    }

    // Handle create mode (before edit mode)
    // When in create mode, handle form navigation and editor input
    if app.ui.mode == crate::tui::app::Mode::Create {
        return handle_create_mode(app, key_event);
    }

    // Handle help mode
    if app.ui.mode == crate::tui::app::Mode::Help {
        return handle_help_mode(app, key_event);
    }

    // Handle settings mode
    // Only handle Esc and settings binding here; other keys fall through to global bindings
    if app.ui.mode == crate::tui::app::Mode::Settings {
        let categories = app.get_settings_categories();
        let is_theme_settings = categories.get(app.settings.category_index)
            .map(|c| c == "Theme Settings")
            .unwrap_or(false);
        
        // Handle Tab/Shift+Tab to navigate between fields
        // But only if not in a special input mode
        match key_event.code {
            KeyCode::Tab | KeyCode::BackTab => {
                // Don't handle Tab if in color input mode or save theme name mode
                if !(is_theme_settings && app.settings.color_input_mode) &&
                   !(is_theme_settings && app.settings.color_save_theme_name_editor.is_some()) {
                    let forward = key_event.code == KeyCode::Tab && 
                                 !key_event.modifiers.contains(KeyModifiers::SHIFT);
                    
                    // If in SettingsContent and Theme Settings, Tab moves between Theme and Color Options fields
                    if app.settings.current_field == crate::tui::app::SettingsField::SettingsContent && 
                       is_theme_settings {
                        if app.settings.in_theme_list_area {
                            // In Theme field: Shift+Tab moves back to CategoryList, Tab moves to Color Options
                            if !forward {
                                // Shift+Tab from Theme: move back to CategoryList
                                app.settings.current_field = crate::tui::app::SettingsField::CategoryList;
                            } else {
                                // Tab from Theme: move to Color Options
                                app.settings.in_theme_list_area = false;
                                app.settings.color_field_index = 0;
                                // Initialize cycle indices based on current color values
                                app.initialize_color_cycle_indices();
                            }
                        } else {
                            // In Color Options: Tab/Shift+Tab moves between color fields
                            // Tab from last field moves back to CategoryList, Shift+Tab from first field moves back to Theme
                            let is_first_field = app.settings.color_field_index == 0;
                            let is_last_field = app.settings.color_field_index == 4;
                            
                            if forward {
                                // Tab: move to next color field, or back to CategoryList if at last field
                                if is_last_field {
                                    app.settings.current_field = crate::tui::app::SettingsField::CategoryList;
                                } else {
                                    app.move_color_field(true);
                                }
                            } else {
                                // Shift+Tab: move to previous color field, or back to Theme if at first field
                                if is_first_field {
                                    app.settings.in_theme_list_area = true;
                                } else {
                                    app.move_color_field(false);
                                }
                            }
                        }
                        return Ok(false);
                    } else {
                        // For other settings or when in CategoryList:
                        // Tab from SettingsContent or Shift+Tab from CategoryList moves between CategoryList and SettingsContent
                        if app.settings.current_field == crate::tui::app::SettingsField::SettingsContent && forward {
                            // Tab from SettingsContent: move back to CategoryList
                            app.settings.current_field = crate::tui::app::SettingsField::CategoryList;
                        } else if app.settings.current_field == crate::tui::app::SettingsField::CategoryList && !forward {
                            // Shift+Tab from CategoryList: move to SettingsContent
                            app.settings.current_field = crate::tui::app::SettingsField::SettingsContent;
                            if is_theme_settings {
                                app.settings.in_theme_list_area = true;
                            }
                        } else {
                            // Regular Tab navigation
                            app.navigate_settings_fields();
                        }
                        return Ok(false);
                    }
                }
            }
            _ => {}
        }
        
        // Handle navigation based on current field
        match app.settings.current_field {
            crate::tui::app::SettingsField::CategoryList => {
                // In category list: Up/Down navigates categories
                match key_event.code {
                    KeyCode::Up => {
                        app.move_settings_category_up();
                        return Ok(false);
                    }
                    KeyCode::Down => {
                        app.move_settings_category_down();
                        return Ok(false);
                    }
                    _ => {
                        // Other keys fall through to settings content handling
                    }
                }
            }
            crate::tui::app::SettingsField::SettingsContent => {
                // In settings content: handle based on category
                // This will be handled below
            }
        }
        
        // Handle color editor input mode
        if is_theme_settings && app.settings.color_input_mode {
            // Extract undo binding before borrowing editor
            let undo_binding = parse_key_binding(&app.config.key_bindings.undo)
                .map_err(|e| TuiError::KeyBindingError(e))?;
            let is_undo = matches_key_event(key_event, &undo_binding);
            
            if let Some(ref mut editor) = app.get_color_input_editor() {
                match key_event.code {
                    KeyCode::Enter => {
                        match app.validate_and_apply_color_input() {
                            Ok(()) => {
                                // Success - already handled in validate_and_apply_color_input
                            }
                            Err(e) => {
                                app.settings.color_input_error = Some(e);
                            }
                        }
                        return Ok(false);
                    }
                    KeyCode::Esc => {
                        app.exit_color_input_mode();
                        return Ok(false);
                    }
                    KeyCode::Tab | KeyCode::BackTab => {
                        // Exit input mode and move to next field
                        app.exit_color_input_mode();
                        let forward = key_event.code == KeyCode::Tab && 
                                     !key_event.modifiers.contains(KeyModifiers::SHIFT);
                        app.move_color_field(forward);
                        return Ok(false);
                    }
                    KeyCode::Up | KeyCode::Down => {
                        // Exit input mode and move field
                        app.exit_color_input_mode();
                        if key_event.code == KeyCode::Up {
                            app.move_color_field_up();
                        } else {
                            app.move_color_field_down();
                        }
                        return Ok(false);
                    }
                    _ => {
                        // Forward to editor
                        if is_undo {
                            editor.undo();
                            return Ok(false);
                        }
                        
                        let extend_selection = key_event.modifiers.contains(KeyModifiers::SHIFT);
                        match key_event.code {
                            KeyCode::Char(c) => {
                                if crate::utils::has_primary_modifier(key_event.modifiers) {
                                    return Ok(false);
                                }
                                editor.insert_char(c);
                                return Ok(false);
                            }
                            KeyCode::Backspace => {
                                editor.delete_char();
                                return Ok(false);
                            }
                            KeyCode::Left => {
                                editor.move_cursor_left(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::Right => {
                                editor.move_cursor_right(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::Home => {
                                editor.move_cursor_home(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::End => {
                                editor.move_cursor_end(extend_selection);
                                return Ok(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Handle settings content navigation (only when SettingsContent is active)
        if app.settings.current_field == crate::tui::app::SettingsField::SettingsContent {
            // Handle color editor navigation and cycling (when not in input mode)
            if is_theme_settings && !app.settings.color_input_mode {
                match key_event.code {
                    KeyCode::Up => {
                        if app.settings.in_theme_list_area {
                            // Navigate within theme list only
                            if app.settings.theme_index > 0 {
                                app.settings.theme_index -= 1;
                                app.settings.theme_list_state.select(Some(app.settings.theme_index));
                            }
                        } else {
                            // Navigate within color fields only
                            app.move_color_field_up();
                        }
                        return Ok(false);
                    }
                    KeyCode::Down => {
                        if app.settings.in_theme_list_area {
                            // Navigate within theme list only
                            let themes = app.get_available_themes();
                            if app.settings.theme_index < themes.len().saturating_sub(1) {
                                app.settings.theme_index += 1;
                                app.settings.theme_list_state.select(Some(app.settings.theme_index));
                            }
                        } else {
                            // Navigate within color fields only
                            app.move_color_field_down();
                        }
                        return Ok(false);
                    }
                    KeyCode::Left => {
                        // Only cycle colors when in color fields area
                        if !app.settings.in_theme_list_area {
                            app.cycle_color_left();
                        }
                        return Ok(false);
                    }
                    KeyCode::Right => {
                        // Only cycle colors when in color fields area
                        if !app.settings.in_theme_list_area {
                            app.cycle_color_right();
                        }
                        return Ok(false);
                    }
                    KeyCode::Char('i') => {
                        // Only enter input mode when in color fields area
                        if !app.settings.in_theme_list_area {
                            app.enter_color_input_mode();
                        }
                        return Ok(false);
                    }
                    KeyCode::Char('r') => {
                        // Only reset colors when in color fields area
                        if !app.settings.in_theme_list_area {
                            if let Err(e) = app.reset_color_overrides() {
                                app.set_status_message(format!("Failed to reset colors: {}", e));
                            }
                        }
                        return Ok(false);
                    }
                    KeyCode::Char('s') => {
                        // Only save as theme when in color fields area
                        if !app.settings.in_theme_list_area {
                            app.enter_save_theme_name_mode();
                        }
                        return Ok(false);
                    }
                    KeyCode::Tab | KeyCode::BackTab => {
                        // Tab navigation is handled above in the main Tab handler
                        // This should not be reached, but keep for safety
                        return Ok(false);
                    }
                    _ => {}
                }
            }
            
            // Handle other settings categories (Appearance, Display, System)
            if !is_theme_settings {
                match key_event.code {
                    KeyCode::Up => {
                        let category = categories.get(app.settings.category_index);
                        if let Some(cat) = category {
                            if cat == "Appearance Settings" {
                                app.move_settings_sidebar_width_up();
                            } else if cat == "Display Settings" {
                                app.move_settings_display_mode_up();
                            }
                        }
                        return Ok(false);
                    }
                    KeyCode::Down => {
                        let category = categories.get(app.settings.category_index);
                        if let Some(cat) = category {
                            if cat == "Appearance Settings" {
                                app.move_settings_sidebar_width_down();
                            } else if cat == "Display Settings" {
                                app.move_settings_display_mode_down();
                            }
                        }
                        return Ok(false);
                    }
                    KeyCode::Tab | KeyCode::BackTab => {
                        // For other categories, Tab doesn't do anything special yet
                        // Could be used for future field navigation
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }
        
        // Handle save theme name input mode
        if is_theme_settings && app.settings.color_save_theme_name_editor.is_some() {
            // Extract undo binding before borrowing editor
            let undo_binding = parse_key_binding(&app.config.key_bindings.undo)
                .map_err(|e| TuiError::KeyBindingError(e))?;
            let is_undo = matches_key_event(key_event, &undo_binding);
            
            if let Some(ref mut editor) = app.get_save_theme_name_editor() {
                match key_event.code {
                    KeyCode::Enter => {
                        let name = if editor.lines.is_empty() {
                            String::new()
                        } else {
                            editor.lines[0].clone()
                        };
                        if name.is_empty() {
                            app.set_status_message("Theme name cannot be empty".to_string());
                            // Exit save theme name mode on empty name error
                            app.exit_save_theme_name_mode();
                        } else {
                            match app.save_theme_from_overrides(&name) {
                                Ok(()) => {
                                    app.exit_save_theme_name_mode();
                                }
                                Err(e) => {
                                    app.set_status_message(format!("Failed to save theme: {}", e));
                                    // Exit save theme name mode on error so user can continue editing
                                    app.exit_save_theme_name_mode();
                                }
                            }
                        }
                        return Ok(false);
                    }
                    KeyCode::Esc => {
                        app.exit_save_theme_name_mode();
                        return Ok(false);
                    }
                    _ => {
                        // Forward to editor
                        if is_undo {
                            editor.undo();
                            return Ok(false);
                        }
                        
                        let extend_selection = key_event.modifiers.contains(KeyModifiers::SHIFT);
                        match key_event.code {
                            KeyCode::Char(c) => {
                                if crate::utils::has_primary_modifier(key_event.modifiers) {
                                    return Ok(false);
                                }
                                editor.insert_char(c);
                                return Ok(false);
                            }
                            KeyCode::Backspace => {
                                editor.delete_char();
                                return Ok(false);
                            }
                            KeyCode::Left => {
                                editor.move_cursor_left(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::Right => {
                                editor.move_cursor_right(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::Home => {
                                editor.move_cursor_home(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::End => {
                                editor.move_cursor_end(extend_selection);
                                return Ok(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        match key_event.code {
            KeyCode::Esc => {
                app.exit_settings_mode();
                return Ok(false);
            }
            _ => {
                // Check if settings binding is pressed again to toggle off
                let settings_binding = parse_key_binding(&app.config.key_bindings.settings)
                    .map_err(|e| TuiError::KeyBindingError(e))?;
                if matches_key_event(key_event, &settings_binding) {
                    app.exit_settings_mode();
                    return Ok(false);
                }
                // Allow arrow keys and select in settings mode
                // These will be handled in handle_global_key_bindings
            }
        }
    }

    // Handle notebook modal mode
    if app.ui.mode == crate::tui::app::Mode::NotebookModal {
        return handle_notebook_modal_mode(app, key_event);
    }

    // Handle search mode
    if app.ui.mode == crate::tui::app::Mode::Search {
        return handle_search_mode(app, key_event);
    }

    // Handle filter mode
    if app.ui.mode == crate::tui::app::Mode::Filter {
        return handle_filter_mode(app, key_event);
    }

    // Handle view mode specific key bindings (Ctrl+A and Ctrl+C) before global bindings
    // This needs to be early to prevent terminal from intercepting Ctrl+A
    if app.ui.mode == crate::tui::app::Mode::View && app.ui.selected_item.is_some() {
        // Check for select all (Ctrl+A or Alt+A on macOS) in view mode
        if crate::utils::has_primary_modifier(key_event.modifiers) 
            && (key_event.code == KeyCode::Char('a') || key_event.code == KeyCode::Char('A')) {
            app.ui.view_content_selected = true;
            app.set_status_message("All content selected".to_string());
            return Ok(false);
        }

        // Check for copy (Ctrl+C or Alt+C on macOS) in view mode
        if crate::utils::has_primary_modifier(key_event.modifiers) 
            && (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C')) {
            if let Some(ref item) = app.ui.selected_item {
                let content = get_content_string(item);
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Err(e) = clipboard.set_text(&content) {
                        app.set_status_message(format!("Failed to copy to clipboard: {}", e));
                    } else {
                        app.set_status_message("Copied to clipboard".to_string());
                        app.ui.view_content_selected = false; // Reset selection flag after copy
                    }
                } else {
                    app.set_status_message("Failed to access clipboard".to_string());
                }
            }
            return Ok(false);
        }
    }

    // Handle global key bindings (work across all modes)
    return handle_global_key_bindings(app, key_event);
}

fn handle_create_mode(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    // Check for save binding (Ctrl+s or Alt+s on macOS)
    let save_binding = parse_key_binding(&app.config.key_bindings.save)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    let mut is_save = matches_key_event(key_event, &save_binding);
    
    // On macOS, Option+s may produce a special character (like 'ś') without ALT modifier
    // Check for this case before the character gets inserted into the editor
    #[cfg(target_os = "macos")]
    {
        if !is_save {
            is_save = match key_event.code {
                KeyCode::Char(c) => {
                    // Option+s on macOS typically produces 'ś' (U+015B)
                    // Also check for other possible Option+s results depending on keyboard layout
                    c == 'ś' || c == 'Ś' || c == 'ß' || c == '§'
                }
                _ => false,
            };
        }
    }
    
    if is_save {
        // Save with error handling - most errors are already shown via status messages
        // But if there's an unexpected error, show it
        match app.save_create_form() {
            Ok(()) => {
                // Success - status message already set in save_create_form
            }
            Err(e) => {
                // This should rarely happen since save_create_form handles most errors internally
                app.set_status_message(format!("Unexpected error while saving: {}", e));
            }
        }
        return Ok(false);
    }

    // Check for notebook field navigation (up/down arrows)
    // Tab/Shift+Tab should still work to navigate away from notebook field
    if app.is_notebook_field_active() {
        match key_event.code {
            KeyCode::Up => {
                if let Some(ref mut form) = app.form.create_form {
                    match form {
                        crate::tui::app::CreateForm::Task(task_form) => {
                            if task_form.notebook_selected_index > 0 {
                                task_form.notebook_selected_index -= 1;
                                task_form.notebook_id = if task_form.notebook_selected_index == 0 {
                                    None
                                } else {
                                    app.notebooks.notebooks.get(task_form.notebook_selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                            }
                        }
                        crate::tui::app::CreateForm::Note(note_form) => {
                            if note_form.notebook_selected_index > 0 {
                                note_form.notebook_selected_index -= 1;
                                note_form.notebook_id = if note_form.notebook_selected_index == 0 {
                                    None
                                } else {
                                    app.notebooks.notebooks.get(note_form.notebook_selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                            }
                        }
                        crate::tui::app::CreateForm::Journal(journal_form) => {
                            if journal_form.notebook_selected_index > 0 {
                                journal_form.notebook_selected_index -= 1;
                                journal_form.notebook_id = if journal_form.notebook_selected_index == 0 {
                                    None
                                } else {
                                    app.notebooks.notebooks.get(journal_form.notebook_selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                            }
                        }
                    }
                }
                return Ok(false);
            }
            KeyCode::Down => {
                if let Some(ref mut form) = app.form.create_form {
                    // max_valid_index is app.notebooks.len() (the last valid index)
                    // Index 0 = "[None]", indices 1..=len() = notebooks
                    // So if we have 3 notebooks, valid indices are 0, 1, 2, 3
                    // We use len() + 1 for the comparison to allow reaching index len()
                    let max_valid_index = app.notebooks.notebooks.len();
                    let max_index_for_check = app.notebooks.notebooks.len() + 1;
                    match form {
                        crate::tui::app::CreateForm::Task(task_form) => {
                            if task_form.notebook_selected_index < max_index_for_check {
                                task_form.notebook_selected_index += 1;
                                // Clamp to maximum valid index to prevent going out of bounds
                                if task_form.notebook_selected_index > max_valid_index {
                                    task_form.notebook_selected_index = max_valid_index;
                                }
                                task_form.notebook_id = if task_form.notebook_selected_index == 0 {
                                    None
                                } else {
                                    app.notebooks.notebooks.get(task_form.notebook_selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                            }
                        }
                        crate::tui::app::CreateForm::Note(note_form) => {
                            if note_form.notebook_selected_index < max_index_for_check {
                                note_form.notebook_selected_index += 1;
                                // Clamp to maximum valid index to prevent going out of bounds
                                if note_form.notebook_selected_index > max_valid_index {
                                    note_form.notebook_selected_index = max_valid_index;
                                }
                                note_form.notebook_id = if note_form.notebook_selected_index == 0 {
                                    None
                                } else {
                                    app.notebooks.notebooks.get(note_form.notebook_selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                            }
                        }
                        crate::tui::app::CreateForm::Journal(journal_form) => {
                            if journal_form.notebook_selected_index < max_index_for_check {
                                journal_form.notebook_selected_index += 1;
                                // Clamp to maximum valid index to prevent going out of bounds
                                if journal_form.notebook_selected_index > max_valid_index {
                                    journal_form.notebook_selected_index = max_valid_index;
                                }
                                journal_form.notebook_id = if journal_form.notebook_selected_index == 0 {
                                    None
                                } else {
                                    app.notebooks.notebooks.get(journal_form.notebook_selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                            }
                        }
                    }
                }
                return Ok(false);
            }
            // Allow Tab/Shift+Tab to pass through for field navigation
            KeyCode::Tab | KeyCode::BackTab => {
                // Let the Tab handling code below handle this
            }
            _ => {}
        }
    }

    // Check for Tab/Shift+Tab/Enter for field navigation
    // Enter behavior: insert newline if Content field is active, otherwise navigate to next field
    match key_event.code {
        KeyCode::BackTab => {
            // Shift+Tab is sometimes sent as BackTab on some terminals
            app.navigate_form_field(false);
            return Ok(false);
        }
        KeyCode::Tab => {
            let forward = !key_event.modifiers.contains(KeyModifiers::SHIFT);
            app.navigate_form_field(forward);
            return Ok(false);
        }
        KeyCode::Enter => {
            // Check if we're in a Content field - if so, insert newline instead of navigating
            if app.is_content_field_active() {
                // Content field is active - insert newline
                if let Some(ref mut editor) = app.get_current_form_editor() {
                    editor.insert_newline();
                }
                return Ok(false);
            } else {
                // Not in Content field - navigate to next field
                app.navigate_form_field(true);
                return Ok(false);
            }
        }
        KeyCode::Esc => {
            // Cancel creation
            app.exit_create_mode();
            return Ok(false);
        }
        _ => {
            // Check for help binding before default handling
            let help_binding = parse_key_binding(&app.config.key_bindings.help)
                .map_err(|e| TuiError::KeyBindingError(e))?;
            if matches_key_event(key_event, &help_binding) {
                app.enter_markdown_help_mode();
                return Ok(false);
            }
        }
    }

    // Forward all other keys to the current form field's editor
    // Extract config values before borrowing editor
    let undo_binding = parse_key_binding(&app.config.key_bindings.undo)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    let word_left_binding = parse_key_binding(&app.config.key_bindings.word_left)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    let word_right_binding = parse_key_binding(&app.config.key_bindings.word_right)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    
    if let Some(ref mut editor) = app.get_current_form_editor() {
        // Handle undo using config binding
        if matches_key_event(key_event, &undo_binding) {
            editor.undo();
            return Ok(false);
        }
        
        // Handle copy (Ctrl+C or Alt+C on macOS)
        if crate::utils::has_primary_modifier(key_event.modifiers) && 
           (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C')) {
            let selected_text = editor.get_selected_text();
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Err(e) = clipboard.set_text(&selected_text) {
                    app.set_status_message(format!("Failed to copy to clipboard: {}", e));
                } else if !selected_text.is_empty() {
                    app.set_status_message("Copied to clipboard".to_string());
                }
            } else {
                app.set_status_message("Failed to access clipboard".to_string());
            }
            return Ok(false);
        }
        
        // Handle cut (Ctrl+X or Alt+X on macOS)
        if crate::utils::has_primary_modifier(key_event.modifiers) && 
           (key_event.code == KeyCode::Char('x') || key_event.code == KeyCode::Char('X')) {
            let selected_text = editor.get_selected_text();
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Err(e) = clipboard.set_text(&selected_text) {
                    app.set_status_message(format!("Failed to copy to clipboard: {}", e));
                } else {
                    if !selected_text.is_empty() {
                        editor.delete_selection();
                        app.set_status_message("Cut to clipboard".to_string());
                    }
                }
            } else {
                app.set_status_message("Failed to access clipboard".to_string());
            }
            return Ok(false);
        }
        
        // Handle select all (Ctrl+A or Alt+A on macOS)
        if crate::utils::has_primary_modifier(key_event.modifiers) && 
           (key_event.code == KeyCode::Char('a') || key_event.code == KeyCode::Char('A')) {
            editor.select_all();
            return Ok(false);
        }
        
        // Handle word navigation using config bindings
        
        let extend_selection = key_event.modifiers.contains(KeyModifiers::SHIFT);
        
        match key_event.code {
            KeyCode::Char(c) => {
                // Skip if primary modifier is held (to avoid inserting 'c' or 'x' when copy/cut is intended)
                if crate::utils::has_primary_modifier(key_event.modifiers) {
                    return Ok(false);
                }
                editor.insert_char(c);
                return Ok(false);
            }
            KeyCode::Backspace => {
                editor.delete_char();
                return Ok(false);
            }
            KeyCode::Up => {
                editor.move_cursor_up(extend_selection);
                return Ok(false);
            }
            KeyCode::Down => {
                editor.move_cursor_down(extend_selection);
                return Ok(false);
            }
            KeyCode::Left => {
                if matches_key_event(key_event, &word_left_binding) {
                    editor.move_cursor_word_left(extend_selection);
                } else {
                    editor.move_cursor_left(extend_selection);
                }
                return Ok(false);
            }
            KeyCode::Right => {
                if matches_key_event(key_event, &word_right_binding) {
                    editor.move_cursor_word_right(extend_selection);
                } else {
                    editor.move_cursor_right(extend_selection);
                }
                return Ok(false);
            }
            KeyCode::Home => {
                editor.move_cursor_home(extend_selection);
                return Ok(false);
            }
            KeyCode::End => {
                editor.move_cursor_end(extend_selection);
                return Ok(false);
            }
            _ => {
                // Ignore other keys in create mode
                return Ok(false);
            }
        }
    }
    Ok(false)
}

fn handle_help_mode(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    match key_event.code {
        KeyCode::Esc => {
            app.exit_help_mode();
            return Ok(false);
        }
        _ => {
            // Check if help binding is pressed again to toggle off
            let help_binding = parse_key_binding(&app.config.key_bindings.help)
                .map_err(|e| TuiError::KeyBindingError(e))?;
            if matches_key_event(key_event, &help_binding) {
                app.exit_help_mode();
                return Ok(false);
            }
            // Ignore all other keys in help mode
            return Ok(false);
        }
    }
}

fn handle_notebook_modal_mode(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    match key_event.code {
        KeyCode::Esc => {
            app.exit_notebook_modal_mode();
            return Ok(false);
        }
        _ => {
            // Check if notebook modal binding is pressed again to toggle off
            let notebook_modal_binding = parse_key_binding(&app.config.key_bindings.notebook_modal)
                .map_err(|e| TuiError::KeyBindingError(e))?;
            let mut is_notebook_modal = matches_key_event(key_event, &notebook_modal_binding);
            
            // On macOS, Option+n may produce a special character (like '˜') without ALT modifier
            // Check for this case before the character gets inserted into the editor
            #[cfg(target_os = "macos")]
            {
                if !is_notebook_modal {
                    is_notebook_modal = match key_event.code {
                        KeyCode::Char(c) => {
                            // Option+n on macOS typically produces '˜' (U+02DC, small tilde)
                            // Also check for regular tilde '~' (U+007E) depending on keyboard layout
                            c == '˜' || c == '~'
                        }
                        _ => false,
                    };
                }
            }
            
            if is_notebook_modal {
                app.exit_notebook_modal_mode();
                return Ok(false);
            }
        }
    }

    if let Some(ref mut state) = app.notebooks.modal_state {
        // Handle field navigation
        match key_event.code {
            KeyCode::Tab | KeyCode::BackTab => {
                app.navigate_notebook_modal();
                return Ok(false);
            }
            KeyCode::Up => {
                match state.current_field {
                    crate::tui::app::NotebookModalField::NotebookList => {
                        app.move_notebook_selection_up();
                    }
                    crate::tui::app::NotebookModalField::ActionsList => {
                        app.move_actions_selection_up();
                    }
                }
                return Ok(false);
            }
            KeyCode::Down => {
                match state.current_field {
                    crate::tui::app::NotebookModalField::NotebookList => {
                        app.move_notebook_selection_down();
                    }
                    crate::tui::app::NotebookModalField::ActionsList => {
                        app.move_actions_selection_down();
                    }
                }
                return Ok(false);
            }
            KeyCode::Enter => {
                // If in Add or Rename mode, save the notebook
                if matches!(state.mode, crate::tui::app::NotebookModalMode::Add | crate::tui::app::NotebookModalMode::Rename) {
                    let name = if state.name_editor.lines.is_empty() {
                        String::new()
                    } else {
                        state.name_editor.lines[0].clone()
                    };
                    
                    let selected_idx = state.selected_index;
                    let mode = state.mode.clone();
                    let notebook_id_opt = if selected_idx > 0 {
                        app.notebooks.notebooks.get(selected_idx - 1).and_then(|n| n.id)
                    } else {
                        None
                    };
                    
                    // Release the borrow on state by ending the if let block
                    // We'll re-borrow after calling methods
                    
                    match mode {
                        crate::tui::app::NotebookModalMode::Add => {
                            if let Err(e) = app.add_notebook(name) {
                                app.set_status_message(format!("Failed to add notebook: {}", e));
                            } else {
                                // Reload notebooks
                                app.notebooks.notebooks = app.database.get_all_notebooks().unwrap_or_default();
                                if let Some(ref mut new_state) = app.notebooks.modal_state {
                                    new_state.mode = crate::tui::app::NotebookModalMode::View;
                                    new_state.name_editor = Editor::new();
                                }
                            }
                        }
                        crate::tui::app::NotebookModalMode::Rename => {
                            if let Some(id) = notebook_id_opt {
                                if let Err(e) = app.rename_notebook(id, name) {
                                    app.set_status_message(format!("Failed to rename notebook: {}", e));
                                } else {
                                    // Reload notebooks
                                    app.notebooks.notebooks = app.database.get_all_notebooks().unwrap_or_default();
                                    if let Some(ref mut new_state) = app.notebooks.modal_state {
                                        new_state.mode = crate::tui::app::NotebookModalMode::View;
                                        new_state.name_editor = Editor::new();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                
                // Otherwise, handle field actions
                match state.current_field {
                    crate::tui::app::NotebookModalField::ActionsList => {
                        // Handle action based on actions_selected_index
                        // 0 = Add, 1 = Rename, 2 = Delete, 3 = Switch
                        match state.actions_selected_index {
                            0 => {
                                // Add
                                state.mode = crate::tui::app::NotebookModalMode::Add;
                                state.name_editor = Editor::new();
                            }
                            1 => {
                                // Rename
                                if state.selected_index > 0 {
                                    // Can't rename "[None]"
                                    let notebook_id = app.notebooks.notebooks.get(state.selected_index - 1)
                                        .and_then(|n| n.id);
                                    if let Some(id) = notebook_id {
                                        state.mode = crate::tui::app::NotebookModalMode::Rename;
                                        state.name_editor = Editor::from_string(
                                            app.notebooks.notebooks.iter()
                                                .find(|n| n.id == Some(id))
                                                .map(|n| n.name.clone())
                                                .unwrap_or_default()
                                        );
                                    }
                                }
                            }
                            2 => {
                                // Delete
                                if state.selected_index > 0 {
                                    // Can't delete "[None]"
                                    let notebook_id = app.notebooks.notebooks.get(state.selected_index - 1)
                                        .and_then(|n| n.id);
                                    if let Some(id) = notebook_id {
                                        let selected_idx = state.selected_index;
                                        
                                        // Release the borrow on state by ending the if let block
                                        
                                        if let Err(e) = app.delete_notebook(id) {
                                            app.set_status_message(format!("Failed to delete notebook: {}", e));
                                        } else {
                                            // Reload notebooks and reset selection
                                            app.notebooks.notebooks = app.database.get_all_notebooks().unwrap_or_default();
                                            if let Some(ref mut new_state) = app.notebooks.modal_state {
                                                if selected_idx > app.notebooks.notebooks.len() {
                                                    new_state.selected_index = app.notebooks.notebooks.len();
                                                }
                                                new_state.list_state.select(Some(new_state.selected_index));
                                            }
                                        }
                                    }
                                }
                            }
                            3 => {
                                // Switch
                                let notebook_id = if state.selected_index == 0 {
                                    None // "[None]"
                                } else {
                                    app.notebooks.notebooks.get(state.selected_index - 1)
                                        .and_then(|n| n.id)
                                };
                                if let Err(e) = app.switch_notebook(notebook_id) {
                                    app.set_status_message(format!("Failed to switch notebook: {}", e));
                                } else {
                                    app.exit_notebook_modal_mode();
                                }
                                return Ok(false);
                            }
                            _ => {}
                        }
                    }
                    crate::tui::app::NotebookModalField::NotebookList => {
                        // Switch to the selected notebook
                        let notebook_id = if state.selected_index == 0 {
                            None // "[None]"
                        } else {
                            app.notebooks.notebooks.get(state.selected_index - 1)
                                .and_then(|n| n.id)
                        };
                        if let Err(e) = app.switch_notebook(notebook_id) {
                            app.set_status_message(format!("Failed to switch notebook: {}", e));
                        } else {
                            app.exit_notebook_modal_mode();
                        }
                        return Ok(false);
                    }
                }
                return Ok(false);
            }
            _ => {
                // Handle text input for add/rename mode
                if matches!(state.mode, crate::tui::app::NotebookModalMode::Add | crate::tui::app::NotebookModalMode::Rename) {
                    let undo_binding = parse_key_binding(&app.config.key_bindings.undo)
                        .map_err(|e| TuiError::KeyBindingError(e))?;
                    
                    if let Some(ref mut editor) = app.get_notebook_modal_editor() {
                        if matches_key_event(key_event, &undo_binding) {
                            editor.undo();
                            return Ok(false);
                        }
                        
                        let extend_selection = key_event.modifiers.contains(KeyModifiers::SHIFT);
                        
                        match key_event.code {
                            KeyCode::Char(c) => {
                                if crate::utils::has_primary_modifier(key_event.modifiers) {
                                    return Ok(false);
                                }
                                editor.insert_char(c);
                                return Ok(false);
                            }
                            KeyCode::Backspace => {
                                editor.delete_char();
                                return Ok(false);
                            }
                            KeyCode::Left => {
                                editor.move_cursor_left(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::Right => {
                                editor.move_cursor_right(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::Home => {
                                editor.move_cursor_home(extend_selection);
                                return Ok(false);
                            }
                            KeyCode::End => {
                                editor.move_cursor_end(extend_selection);
                                return Ok(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

fn handle_search_mode(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    match key_event.code {
        KeyCode::Esc => {
            app.exit_search_mode();
            return Ok(false);
        }
        KeyCode::Enter => {
            app.exit_search_mode();
            return Ok(false);
        }
        KeyCode::Char(c) => {
            app.add_to_search(c);
            return Ok(false);
        }
        KeyCode::Backspace => {
            app.remove_from_search();
            return Ok(false);
        }
        _ => Ok(false)
    }
}

fn handle_filter_mode(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    match key_event.code {
        KeyCode::Esc => {
            app.exit_filter_mode();
            return Ok(false);
        }
        KeyCode::BackTab => {
            app.navigate_filter_field(false);
            return Ok(false);
        }
        KeyCode::Tab => {
            let forward = !key_event.modifiers.contains(KeyModifiers::SHIFT);
            app.navigate_filter_field(forward);
            return Ok(false);
        }
        KeyCode::Enter => {
            // Check which field is active
            if let Some(ref state) = app.filter.form_state {
                match state.current_field {
                    crate::tui::app::FilterFormField::Apply => {
                        app.apply_filters();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Clear => {
                        app.clear_filters();
                        app.exit_filter_mode();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Cancel => {
                        app.exit_filter_mode();
                        return Ok(false);
                    }
                    _ => {
                        // Navigate to next field
                        app.navigate_filter_field(true);
                        return Ok(false);
                    }
                }
            } else {
                // No form state available, ignore the Enter key
                return Ok(false);
            }
        }
        KeyCode::Up => {
            if let Some(ref mut state) = app.filter.form_state {
                match state.current_field {
                    crate::tui::app::FilterFormField::Archived => {
                        app.move_filter_archived_up();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Status => {
                        app.move_filter_status_up();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::TagLogic => {
                        app.move_filter_tag_logic_up();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Apply => {
                        // Wrap to Cancel
                        state.current_field = crate::tui::app::FilterFormField::Cancel;
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Clear => {
                        // Move to Apply
                        state.current_field = crate::tui::app::FilterFormField::Apply;
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Cancel => {
                        // Move to Clear
                        state.current_field = crate::tui::app::FilterFormField::Clear;
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Down => {
            if let Some(ref mut state) = app.filter.form_state {
                match state.current_field {
                    crate::tui::app::FilterFormField::Archived => {
                        app.move_filter_archived_down();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Status => {
                        app.move_filter_status_down();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::TagLogic => {
                        app.move_filter_tag_logic_down();
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Apply => {
                        // Move to Clear
                        state.current_field = crate::tui::app::FilterFormField::Clear;
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Clear => {
                        // Move to Cancel
                        state.current_field = crate::tui::app::FilterFormField::Cancel;
                        return Ok(false);
                    }
                    crate::tui::app::FilterFormField::Cancel => {
                        // Wrap to Apply
                        state.current_field = crate::tui::app::FilterFormField::Apply;
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }
        _ => {
            // Handle text input for tags field
            if app.is_filter_tags_field_active() {
                let undo_binding = parse_key_binding(&app.config.key_bindings.undo)
                    .map_err(|e| TuiError::KeyBindingError(e))?;
                let word_left_binding = parse_key_binding(&app.config.key_bindings.word_left)
                    .map_err(|e| TuiError::KeyBindingError(e))?;
                let word_right_binding = parse_key_binding(&app.config.key_bindings.word_right)
                    .map_err(|e| TuiError::KeyBindingError(e))?;
                
                if let Some(ref mut editor) = app.get_current_filter_editor() {
                    if matches_key_event(key_event, &undo_binding) {
                        editor.undo();
                        return Ok(false);
                    }
                    
                    // Handle copy (Ctrl+C or Alt+C on macOS)
                    if crate::utils::has_primary_modifier(key_event.modifiers) && 
                       (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C')) {
                        let selected_text = editor.get_selected_text();
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Err(e) = clipboard.set_text(&selected_text) {
                                app.set_status_message(format!("Failed to copy to clipboard: {}", e));
                            } else if !selected_text.is_empty() {
                                app.set_status_message("Copied to clipboard".to_string());
                            }
                        } else {
                            app.set_status_message("Failed to access clipboard".to_string());
                        }
                        return Ok(false);
                    }
                    
                    // Handle cut (Ctrl+X or Alt+X on macOS)
                    if crate::utils::has_primary_modifier(key_event.modifiers) && 
                       (key_event.code == KeyCode::Char('x') || key_event.code == KeyCode::Char('X')) {
                        let selected_text = editor.get_selected_text();
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Err(e) = clipboard.set_text(&selected_text) {
                                app.set_status_message(format!("Failed to copy to clipboard: {}", e));
                            } else {
                                if !selected_text.is_empty() {
                                    editor.delete_selection();
                                    app.set_status_message("Cut to clipboard".to_string());
                                }
                            }
                        } else {
                            app.set_status_message("Failed to access clipboard".to_string());
                        }
                        return Ok(false);
                    }
                    
                    // Handle select all (Ctrl+A or Alt+A on macOS)
                    if crate::utils::has_primary_modifier(key_event.modifiers) && 
                       (key_event.code == KeyCode::Char('a') || key_event.code == KeyCode::Char('A')) {
                        editor.select_all();
                        return Ok(false);
                    }
                    
                    let extend_selection = key_event.modifiers.contains(KeyModifiers::SHIFT);
                    
                    match key_event.code {
                        KeyCode::Char(c) => {
                            // Skip if primary modifier is held (to avoid inserting 'c' or 'x' when copy/cut is intended)
                            if crate::utils::has_primary_modifier(key_event.modifiers) {
                                return Ok(false);
                            }
                            editor.insert_char(c);
                            return Ok(false);
                        }
                        KeyCode::Backspace => {
                            editor.delete_char();
                            return Ok(false);
                        }
                        KeyCode::Up => {
                            editor.move_cursor_up(extend_selection);
                            return Ok(false);
                        }
                        KeyCode::Down => {
                            editor.move_cursor_down(extend_selection);
                            return Ok(false);
                        }
                        KeyCode::Left => {
                            if matches_key_event(key_event, &word_left_binding) {
                                editor.move_cursor_word_left(extend_selection);
                            } else {
                                editor.move_cursor_left(extend_selection);
                            }
                            return Ok(false);
                        }
                        KeyCode::Right => {
                            if matches_key_event(key_event, &word_right_binding) {
                                editor.move_cursor_word_right(extend_selection);
                            } else {
                                editor.move_cursor_right(extend_selection);
                            }
                            return Ok(false);
                        }
                        KeyCode::Home => {
                            editor.move_cursor_home(extend_selection);
                            return Ok(false);
                        }
                        KeyCode::End => {
                            editor.move_cursor_end(extend_selection);
                            return Ok(false);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(false)
}

fn handle_global_key_bindings(app: &mut App, key_event: KeyEvent) -> Result<bool, TuiError> {
    // Check for quit key
    let quit_binding = parse_key_binding(&app.config.key_bindings.quit)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &quit_binding) {
        return Ok(true); // Quit
    }

    // Check for toggle sidebar (Ctrl+b or Alt+b on macOS)
    let toggle_binding = parse_key_binding(&app.config.key_bindings.toggle_sidebar)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &toggle_binding) {
        app.toggle_sidebar();
        return Ok(false);
    }

    // Check for notebook modal binding - process early to catch Option+n before other handlers
    let notebook_modal_binding = parse_key_binding(&app.config.key_bindings.notebook_modal)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    let mut is_notebook_modal = matches_key_event(key_event, &notebook_modal_binding);
    
    // On macOS, Option+n may produce a special character (like '˜') without ALT modifier
    // Check for this case before the character gets inserted into the editor
    #[cfg(target_os = "macos")]
    {
        if !is_notebook_modal {
            is_notebook_modal = match key_event.code {
                KeyCode::Char(c) => {
                    // Option+n on macOS typically produces '˜' (U+02DC, small tilde)
                    // Also check for regular tilde '~' (U+007E) depending on keyboard layout
                    c == '˜' || c == '~'
                }
                _ => false,
            };
        }
    }
    
    if is_notebook_modal {
        if app.ui.mode == crate::tui::app::Mode::NotebookModal {
            app.exit_notebook_modal_mode();
        } else {
            app.enter_notebook_modal_mode();
        }
        return Ok(false);
    }

    // Check for tab navigation - process these early and return to prevent double-processing
    let tab_left_binding = parse_key_binding(&app.config.key_bindings.tab_left)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &tab_left_binding) {
        match app.ui.current_tab {
            crate::tui::app::Tab::Tasks => {
                // Already at first tab, do nothing
            }
            crate::tui::app::Tab::Notes => {
                app.switch_tab(crate::tui::app::Tab::Tasks);
            }
            crate::tui::app::Tab::Journal => {
                app.switch_tab(crate::tui::app::Tab::Notes);
            }
        }
        return Ok(false);
    }

    let tab_right_binding = parse_key_binding(&app.config.key_bindings.tab_right)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &tab_right_binding) {
        match app.ui.current_tab {
            crate::tui::app::Tab::Tasks => {
                app.switch_tab(crate::tui::app::Tab::Notes);
            }
            crate::tui::app::Tab::Notes => {
                app.switch_tab(crate::tui::app::Tab::Journal);
            }
            crate::tui::app::Tab::Journal => {
                // Already at last tab, do nothing
            }
        }
        return Ok(false);
    }

    // Check for task reordering (Ctrl+Up/Down or Alt+Up/Down on macOS) - only on Tasks tab in View mode
    if app.ui.mode == crate::tui::app::Mode::View 
        && app.ui.current_tab == crate::tui::app::Tab::Tasks
        && crate::utils::has_primary_modifier(key_event.modifiers) {
        match key_event.code {
            KeyCode::Up => {
                if let Err(e) = app.reorder_task_up() {
                    app.set_status_message(format!("Failed to reorder task: {}", e));
                }
                return Ok(false);
            }
            KeyCode::Down => {
                if let Err(e) = app.reorder_task_down() {
                    app.set_status_message(format!("Failed to reorder task: {}", e));
                }
                return Ok(false);
            }
            _ => {}
        }
    }

    // Check for arrow key navigation (when not in create mode)
    // Arrow keys work as an alternative to configured bindings
    // In Settings mode, Up/Down arrows navigate between categories
    if app.ui.mode != crate::tui::app::Mode::Create {
        match key_event.code {
            KeyCode::Up => {
                if app.ui.mode == crate::tui::app::Mode::Settings {
                    // Up/Down arrows navigate between settings categories
                    app.move_settings_category_up();
                } else {
                    app.move_selection_up();
                }
                return Ok(false);
            }
            KeyCode::Down => {
                if app.ui.mode == crate::tui::app::Mode::Settings {
                    // Up/Down arrows navigate between settings categories
                    app.move_settings_category_down();
                } else {
                    app.move_selection_down();
                }
                return Ok(false);
            }
            KeyCode::PageUp => {
                // Scroll item view page up if in View mode with selected item
                if app.ui.mode == crate::tui::app::Mode::View && app.ui.selected_item.is_some() {
                    if let Ok((_, height)) = terminal_size() {
                        use crate::tui::layout::Layout;
                        use ratatui::layout::Rect;
                        let rect = Rect::new(0, 0, 80, height); // Width doesn't matter for height calculation
                        let layout = Layout::calculate(
                            rect,
                            app.config.sidebar_width_percent,
                            app.ui.sidebar_state == crate::tui::app::SidebarState::Collapsed,
                        );
                        let viewport_height = (layout.main_area.height - 2) as usize;
                        app.scroll_item_view_page_up(viewport_height);
                    }
                }
                return Ok(false);
            }
            KeyCode::PageDown => {
                // Scroll item view page down if in View mode with selected item
                if app.ui.mode == crate::tui::app::Mode::View && app.ui.selected_item.is_some() {
                    if let Ok((_, height)) = terminal_size() {
                        use crate::tui::layout::Layout;
                        use ratatui::layout::Rect;
                        let rect = Rect::new(0, 0, 80, height); // Width doesn't matter for height calculation
                        let layout = Layout::calculate(
                            rect,
                            app.config.sidebar_width_percent,
                            app.ui.sidebar_state == crate::tui::app::SidebarState::Collapsed,
                        );
                        let viewport_height = (layout.main_area.height - 2) as usize;
                        app.scroll_item_view_page_down(viewport_height);
                    }
                }
                return Ok(false);
            }
            KeyCode::Home => {
                // Scroll item view to top if in View mode with selected item
                if app.ui.mode == crate::tui::app::Mode::View && app.ui.selected_item.is_some() {
                    app.scroll_item_view_to_top();
                }
                return Ok(false);
            }
            KeyCode::End => {
                // Scroll item view to bottom if in View mode with selected item
                if app.ui.mode == crate::tui::app::Mode::View && app.ui.selected_item.is_some() {
                    if let Ok((_, height)) = terminal_size() {
                        use crate::tui::layout::Layout;
                        use ratatui::layout::Rect;
                        let rect = Rect::new(0, 0, 80, height); // Width doesn't matter for height calculation
                        let layout = Layout::calculate(
                            rect,
                            app.config.sidebar_width_percent,
                            app.ui.sidebar_state == crate::tui::app::SidebarState::Collapsed,
                        );
                        let viewport_height = (layout.main_area.height - 2) as usize;
                        app.scroll_item_view_to_bottom(viewport_height);
                    }
                }
                return Ok(false);
            }
            _ => {}
        }
    }

    // Check for list navigation bindings
    // In Settings mode, j/k navigate within the current settings form
    let list_down_binding = parse_key_binding(&app.config.key_bindings.list_down)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &list_down_binding) {
        if app.ui.mode == crate::tui::app::Mode::Settings {
            // Navigate within current settings category
            let categories = app.get_settings_categories();
            if let Some(category) = categories.get(app.settings.category_index) {
                if category == "Theme Settings" {
                    app.move_settings_theme_selection_down();
                } else if category == "Appearance Settings" {
                    app.move_settings_sidebar_width_down();
                } else if category == "Display Settings" {
                    app.move_settings_display_mode_down();
                }
            }
        } else {
            app.move_selection_down();
        }
        return Ok(false);
    }

    let list_up_binding = parse_key_binding(&app.config.key_bindings.list_up)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &list_up_binding) {
        if app.ui.mode == crate::tui::app::Mode::Settings {
            // Navigate within current settings category
            let categories = app.get_settings_categories();
            if let Some(category) = categories.get(app.settings.category_index) {
                if category == "Theme Settings" {
                    app.move_settings_theme_selection_up();
                } else if category == "Appearance Settings" {
                    app.move_settings_sidebar_width_up();
                } else if category == "Display Settings" {
                    app.move_settings_display_mode_up();
                }
            }
        } else {
            app.move_selection_up();
        }
        return Ok(false);
    }

    // Check for tab number bindings
    let tab_1_binding = parse_key_binding(&app.config.key_bindings.tab_1)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &tab_1_binding) {
        app.switch_tab(crate::tui::app::Tab::Tasks);
        return Ok(false);
    }

    let tab_2_binding = parse_key_binding(&app.config.key_bindings.tab_2)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &tab_2_binding) {
        app.switch_tab(crate::tui::app::Tab::Notes);
        return Ok(false);
    }

    let tab_3_binding = parse_key_binding(&app.config.key_bindings.tab_3)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &tab_3_binding) {
        app.switch_tab(crate::tui::app::Tab::Journal);
        return Ok(false);
    }

    // Check for settings binding
    let settings_binding = parse_key_binding(&app.config.key_bindings.settings)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &settings_binding) {
        if app.ui.mode == crate::tui::app::Mode::Settings {
            app.exit_settings_mode();
        } else {
            app.enter_settings_mode();
        }
        return Ok(false);
    }

    // Check for select binding
    let select_binding = parse_key_binding(&app.config.key_bindings.select)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &select_binding) {
        // In Settings mode, Enter applies selected setting based on category and current field
        if app.ui.mode == crate::tui::app::Mode::Settings {
            let categories = app.get_settings_categories();
            if let Some(category) = categories.get(app.settings.category_index) {
                // If in CategoryList, Enter moves to SettingsContent
                if app.settings.current_field == crate::tui::app::SettingsField::CategoryList {
                    app.settings.current_field = crate::tui::app::SettingsField::SettingsContent;
                    // Reset theme list area state when entering SettingsContent
                    app.settings.in_theme_list_area = true;
                } else {
                    // In SettingsContent, Enter applies the selected setting
                    if category == "Theme Settings" {
                        // If in theme list area, select theme
                        if app.settings.in_theme_list_area {
                            let themes = app.get_available_themes();
                            if let Some(theme_name) = themes.get(app.settings.theme_index) {
                                if let Err(e) = app.select_theme(theme_name) {
                                    app.set_status_message(format!("Failed to change theme: {}", e));
                                }
                            }
                        }
                        // If in color fields area, Enter doesn't do anything (use 'i' for input mode)
                    } else if category == "Appearance Settings" {
                        if let Err(e) = app.apply_sidebar_width() {
                            app.set_status_message(format!("Failed to change sidebar width: {}", e));
                        }
                    } else if category == "Display Settings" {
                        if let Err(e) = app.apply_display_mode() {
                            app.set_status_message(format!("Failed to change display mode: {}", e));
                        }
                    }
                }
            }
        } else {
            app.select_current_item();
        }
        return Ok(false);
    }

    // Check for new binding
    let new_binding = parse_key_binding(&app.config.key_bindings.new)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &new_binding) {
        // New item - enter create mode
        app.enter_create_mode();
        return Ok(false);
    }

    // Check for edit binding
    let edit_binding = parse_key_binding(&app.config.key_bindings.edit)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &edit_binding) {
        // Edit item
        app.enter_edit_mode();
        return Ok(false);
    }

    // Check for delete binding
    let delete_binding = parse_key_binding(&app.config.key_bindings.delete)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &delete_binding) {
        // Show confirmation modal instead of deleting immediately
        if let Some(ref item) = app.ui.selected_item {
            app.modals.delete_confirmation = Some(item.clone());
            app.modals.delete_modal_selection = 0; // Initialize to Archive option
        } else {
            app.set_status_message("No item selected".to_string());
        }
        return Ok(false);
    }

    // Check for toggle task status binding
    let toggle_task_status_binding = parse_key_binding(&app.config.key_bindings.toggle_task_status)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &toggle_task_status_binding) {
        // Toggle task status (only works on Tasks tab)
        match app.toggle_task_status() {
            Ok(()) => {
                // Success - status message already set in toggle_task_status
            }
            Err(e) => {
                app.set_status_message(format!("Failed to toggle task status: {}", e));
            }
        }
        return Ok(false);
    }

    // Check for help binding
    let help_binding = parse_key_binding(&app.config.key_bindings.help)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &help_binding) {
        // If in Create mode, show markdown help; otherwise show regular help
        if app.ui.mode == crate::tui::app::Mode::Create {
            app.enter_markdown_help_mode();
        } else {
            app.enter_help_mode();
        }
        return Ok(false);
    }

    // Check for search binding
    let search_binding = parse_key_binding(&app.config.key_bindings.search)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &search_binding) {
        app.enter_search_mode();
        return Ok(false);
    }

    // Check for filter binding
    let filter_binding = parse_key_binding(&app.config.key_bindings.filter)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &filter_binding) {
        app.enter_filter_mode();
        return Ok(false);
    }

    // Check for toggle list view binding
    let toggle_list_view_binding = parse_key_binding(&app.config.key_bindings.toggle_list_view)
        .map_err(|e| TuiError::KeyBindingError(e))?;
    if matches_key_event(key_event, &toggle_list_view_binding) {
        app.toggle_list_view_mode();
        return Ok(false);
    }

    Ok(false)
}

fn matches_key_event(key_event: KeyEvent, binding: &crate::utils::ParsedKeyBinding) -> bool {
    // Check modifiers
    // Use primary modifier check (Ctrl on Windows/Linux, Option/Alt on macOS)
    // This follows cross-platform TUI best practices
    let has_primary_mod = crate::utils::has_primary_modifier(key_event.modifiers);
    if binding.requires_ctrl != has_primary_mod {
        return false;
    }

    // Check key code
    binding.key_code == key_event.code
}

