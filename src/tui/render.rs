use ratatui::Frame;
use ratatui::widgets::{Block, Borders};
use ratatui::style::Style;
use crate::tui::{App, Layout};
use crate::tui::widgets::{
    tabs::render_tabs,
    task_list::render_task_list,
    note_list::render_note_list,
    journal_list::render_journal_list,
    item_view::render_item_view,
    status_bar::render_status_bar,
    help::render_help,
    form::{render_task_form, render_note_form, render_journal_form},
    color::parse_color,
    confirm_delete::render_confirm_delete,
    filters_box::render_filters_box,
    filter_modal::render_filter_modal,
};

pub fn render(f: &mut Frame, app: &mut App, layout: &Layout) {
    // Render outer border with "TNJ" title centered in top border
    // Use theme colors for consistent appearance
    let active_theme = app.config.get_active_theme();
    let fg_color = parse_color(&active_theme.fg);
    let bg_color = parse_color(&active_theme.bg);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title("TNJ")
        .title_alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(fg_color).bg(bg_color));
    f.render_widget(outer_block, f.area());

    // Render tabs - following ratatui example: tabs render in 1 line without Block
    // Content areas below have borders that visually connect
    render_tabs(f, layout.tabs_area, app.ui.current_tab, &app.config, app);

    // Render sidebar if not collapsed
    if app.ui.sidebar_state == crate::tui::app::SidebarState::Expanded && layout.sidebar_area.width > 0 {
        let items = app.get_current_items();
        match app.ui.current_tab {
            crate::tui::app::Tab::Tasks => {
                let tasks: Vec<_> = items.iter()
                    .filter_map(|item| {
                        if let crate::tui::app::Item::Task(task) = item {
                            Some(task.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                let total_count = app.tasks.len();
                render_task_list(f, layout.sidebar_area, &tasks, total_count, &mut app.ui.list_state, &app.config, app.ui.list_view_mode);
            }
            crate::tui::app::Tab::Notes => {
                let notes: Vec<_> = items.iter()
                    .filter_map(|item| {
                        if let crate::tui::app::Item::Note(note) = item {
                            Some(note.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                let total_count = app.notes.len();
                render_note_list(f, layout.sidebar_area, &notes, total_count, &mut app.ui.list_state, &app.config, app.ui.list_view_mode);
            }
            crate::tui::app::Tab::Journal => {
                let journals: Vec<_> = items.iter()
                    .filter_map(|item| {
                        if let crate::tui::app::Item::Journal(journal) = item {
                            Some(journal.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                let total_count = app.journals.len();
                render_journal_list(f, layout.sidebar_area, &journals, total_count, &mut app.ui.list_state, &app.config, app.ui.list_view_mode);
            }
        }
    }

    // Render main pane (always render normal content first)
    // Note: Help mode and Settings mode render popup overlays separately after normal content
    match app.ui.mode {
            crate::tui::app::Mode::Help | crate::tui::app::Mode::View | crate::tui::app::Mode::Filter | crate::tui::app::Mode::NotebookModal => {
                // View mode - show selected item details (Help mode shows same content with overlay)
                if let Some(ref item) = app.ui.selected_item {
                    render_item_view(f, layout.main_area, item, &app.config, app.ui.item_view_scroll);
                } else {
                    // Empty state
                    use ratatui::widgets::{Block, Borders, Paragraph};
                    let active_theme = app.config.get_active_theme();
                    let fg_color = parse_color(&active_theme.fg);
                    let paragraph = Paragraph::new("Select an item to view details")
                        .block(Block::default().borders(Borders::ALL).title("Content"))
                        .style(Style::default().fg(fg_color));
                    f.render_widget(paragraph, layout.main_area);
                }
            }
            crate::tui::app::Mode::Search => {
                // Show search query in main pane
                use ratatui::widgets::{Block, Borders, Paragraph};
                let active_theme = app.config.get_active_theme();
                let fg_color = parse_color(&active_theme.fg);
                let search_text = format!("Search: {}", app.search.query);
                let paragraph = Paragraph::new(search_text)
                    .block(Block::default().borders(Borders::ALL).title("Search"))
                    .style(Style::default().fg(fg_color));
                f.render_widget(paragraph, layout.main_area);
            }
            crate::tui::app::Mode::Create | crate::tui::app::Mode::MarkdownHelp => {
                // Create mode - render form (MarkdownHelp shows same content with overlay)
                if let Some(ref form) = app.form.create_form {
                    match form {
                        crate::tui::app::CreateForm::Task(task_form) => {
                            render_task_form(f, layout.main_area, task_form, &app.config, &app.notebooks.notebooks);
                        }
                        crate::tui::app::CreateForm::Note(note_form) => {
                            render_note_form(f, layout.main_area, note_form, &app.config, &app.notebooks.notebooks);
                        }
                        crate::tui::app::CreateForm::Journal(journal_form) => {
                            render_journal_form(f, layout.main_area, journal_form, &app.config, &app.notebooks.notebooks);
                        }
                    }
                } else {
                    // Empty state (shouldn't happen)
                    use ratatui::widgets::{Block, Borders, Paragraph};
                    let active_theme = app.config.get_active_theme();
                    let fg_color = parse_color(&active_theme.fg);
                    let paragraph = Paragraph::new("No form")
                        .block(Block::default().borders(Borders::ALL).title("Content"))
                        .style(Style::default().fg(fg_color));
                    f.render_widget(paragraph, layout.main_area);
                }
            }
            crate::tui::app::Mode::Settings => {
                // Settings mode - show normal content (will be overlaid)
                if let Some(ref item) = app.ui.selected_item {
                    render_item_view(f, layout.main_area, item, &app.config, app.ui.item_view_scroll);
                } else {
                    // Empty state
                    use ratatui::widgets::{Block, Borders, Paragraph};
                    let active_theme = app.config.get_active_theme();
                    let fg_color = parse_color(&active_theme.fg);
                    let paragraph = Paragraph::new("Select an item to view details")
                        .block(Block::default().borders(Borders::ALL).title("Content"))
                        .style(Style::default().fg(fg_color));
                    f.render_widget(paragraph, layout.main_area);
                }
            }
        }

    // Render help popup overlay if in help mode (after normal content)
    if app.ui.mode == crate::tui::app::Mode::Help {
        render_help(f, f.area(), &app.config);
    }

    // Render markdown help popup overlay if in markdown help mode (after normal content)
    if app.ui.mode == crate::tui::app::Mode::MarkdownHelp {
        use crate::tui::widgets::markdown_help::render_markdown_help;
        render_markdown_help(f, f.area(), &app.config, app.ui.markdown_help_example_scroll, app.ui.markdown_help_rendered_scroll);
    }

    // Render settings popup overlay if in settings mode (after normal content)
    if app.ui.mode == crate::tui::app::Mode::Settings {
        use crate::tui::widgets::settings_view::render_settings_view_modal;
        render_settings_view_modal(f, f.area(), app);
    }

    // Render delete confirmation modal if pending (after normal content)
    if let Some(ref item) = app.modals.delete_confirmation {
        render_confirm_delete(f, f.area(), item, app.modals.delete_modal_selection, &app.config);
    }

    // Render filters box
    let filter_summary = app.get_filter_summary();
    render_filters_box(f, layout.filters_area, &filter_summary, &app.config);

    // Render filter modal overlay if in filter mode (after normal content)
    if app.ui.mode == crate::tui::app::Mode::Filter {
        render_filter_modal(f, f.area(), app);
    }

    // Render notebook modal overlay if in notebook modal mode (after normal content)
    if app.ui.mode == crate::tui::app::Mode::NotebookModal {
        use crate::tui::widgets::notebook_modal::render_notebook_modal;
        render_notebook_modal(f, f.area(), app);
    }

    // Render status bar
    let key_hints = get_key_hints(app);
    render_status_bar(f, layout.status_area, app.status.message.as_ref(), &key_hints, &app.config);
}

fn get_key_hints(app: &App) -> Vec<String> {
    match app.ui.mode {
        crate::tui::app::Mode::Help => {
            vec![
                format!("Esc or {}: Exit help", crate::utils::format_key_binding_for_display(&app.config.key_bindings.help)),
            ]
        }
        crate::tui::app::Mode::Search => {
            vec![
                "Esc: Exit search".to_string(),
            ]
        }
        crate::tui::app::Mode::Settings => {
            let categories = app.get_settings_categories();
            let is_theme_settings = categories.get(app.settings.category_index)
                .map(|c| c == "Theme Settings")
                .unwrap_or(false);
            
            let mut hints = vec![
                format!("Esc/{}: Exit", crate::utils::format_key_binding_for_display(&app.config.key_bindings.settings)),
            ];
            
            match app.settings.current_field {
                crate::tui::app::SettingsField::CategoryList => {
                    hints.push("↑/↓: Navigate categories".to_string());
                    hints.push("Tab: Enter settings".to_string());
                    hints.push(format!("{}: Enter settings", crate::utils::format_key_binding_for_display(&app.config.key_bindings.select)));
                }
                crate::tui::app::SettingsField::SettingsContent => {
                    hints.push("Tab: Back".to_string());
                    
                    if is_theme_settings {
                        if app.settings.color_input_mode {
                            hints.push("Enter: Apply color".to_string());
                            hints.push("Esc: Cancel input".to_string());
                        } else if app.settings.color_save_theme_name_editor.is_some() {
                            hints.push("Enter: Save theme".to_string());
                            hints.push("Esc: Cancel".to_string());
                        } else {
                            if app.settings.in_theme_list_area {
                                hints.push("↑/↓: Navigate themes".to_string());
                                hints.push(format!("{}: Select theme", crate::utils::format_key_binding_for_display(&app.config.key_bindings.select)));
                                hints.push("Tab: Color Options".to_string());
                                hints.push("Shift+Tab: Back".to_string());
                            } else {
                                hints.push("↑/↓: Navigate fields".to_string());
                                hints.push("Tab/Shift+Tab: Move".to_string());
                                hints.push("←/→: Cycle colors".to_string());
                                hints.push("i: Input mode".to_string());
                                hints.push("r: Reset to theme".to_string());
                                hints.push("s: Save as theme".to_string());
                            }
                        }
                    } else {
                        hints.push("↑/↓: Navigate options".to_string());
                        hints.push(format!("{}: Apply setting", crate::utils::format_key_binding_for_display(&app.config.key_bindings.select)));
                    }
                }
            }
            
            hints
        }
        crate::tui::app::Mode::Create => {
            vec![
                "Tab/Enter: Next field".to_string(),
                "Shift+Tab: Previous field".to_string(),
                format!("{}: Save", crate::utils::format_key_binding_for_display(&app.config.key_bindings.save)),
                format!("{}: Markdown help", crate::utils::format_key_binding_for_display(&app.config.key_bindings.help)),
                "Esc: Cancel".to_string(),
            ]
        }
        crate::tui::app::Mode::MarkdownHelp => {
            vec![
                format!("Esc or {}: Exit markdown help", crate::utils::format_key_binding_for_display(&app.config.key_bindings.help)),
            ]
        }
        crate::tui::app::Mode::Filter => {
            vec![
                "Tab/Shift+Tab: Navigate fields".to_string(),
                format!("{}: Apply filters", crate::utils::format_key_binding_for_display(&app.config.key_bindings.select)),
                "Esc: Cancel".to_string(),
            ]
        }
        crate::tui::app::Mode::NotebookModal => {
            vec![
                "Tab/Shift+Tab: Navigate actions".to_string(),
                "↑/↓: Navigate notebooks".to_string(),
                format!("{}: Select/Switch", crate::utils::format_key_binding_for_display(&app.config.key_bindings.select)),
                format!("{}: Notebooks", crate::utils::format_key_binding_for_display(&app.config.key_bindings.notebook_modal)),
                "Esc: Cancel".to_string(),
            ]
        }
        _ => {
            let mut hints = vec![
                format!("{}: Quit", crate::utils::format_key_binding_for_display(&app.config.key_bindings.quit)),
                format!("{}: New", crate::utils::format_key_binding_for_display(&app.config.key_bindings.new)),
                format!("{}: Edit", crate::utils::format_key_binding_for_display(&app.config.key_bindings.edit)),
                format!("{}: Delete", crate::utils::format_key_binding_for_display(&app.config.key_bindings.delete)),
                format!("{}: Search", crate::utils::format_key_binding_for_display(&app.config.key_bindings.search)),
                format!("{}: Filters", crate::utils::format_key_binding_for_display(&app.config.key_bindings.filter)),
                format!("{}: Sidebar", crate::utils::format_key_binding_for_display(&app.config.key_bindings.toggle_sidebar)),
            ];
            
            // Add task-specific shortcuts when on Tasks tab
            if app.ui.current_tab == crate::tui::app::Tab::Tasks {
                #[cfg(target_os = "macos")]
                {
                    hints.push("Opt+↑/↓: Reorder".to_string());
                }
                #[cfg(not(target_os = "macos"))]
                {
                    hints.push("Ctrl+↑/↓: Reorder".to_string());
                }
            }
            
            // Add tags toggle hint (available on all tabs)
            hints.push(format!("{}: Tags", crate::utils::format_key_binding_for_display(&app.config.key_bindings.toggle_list_view)));
            
            // Add notebook modal hint
            hints.push(format!("{}: Notebooks", crate::utils::format_key_binding_for_display(&app.config.key_bindings.notebook_modal)));
            
            // Add F1 (Help) and F2 (Settings) at the end
            hints.push(format!("{}: Settings", crate::utils::format_key_binding_for_display(&app.config.key_bindings.settings)));
            hints.push(format!("{}: Help", crate::utils::format_key_binding_for_display(&app.config.key_bindings.help)));
            
            hints
        }
    }
}

