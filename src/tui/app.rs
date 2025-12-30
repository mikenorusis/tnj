use crate::{Config, Database, models::{Task, Note, JournalEntry, Notebook}};
use crate::database::DatabaseError;
use crate::tui::widgets::editor::Editor;
use ratatui::widgets::ListState;
use std::cmp;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tasks,
    Notes,
    Journal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarState {
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListViewMode {
    Simple,
    TwoLine,
    GroupedByTags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterArchivedStatus {
    Active,
    Archived,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterTagLogic {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterTaskStatus {
    Todo,
    Done,
    All,
}

#[derive(Debug, Clone)]
pub enum FilterFormField {
    Tags,
    Archived,
    Status,
    TagLogic,
    Apply,
    Clear,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct FilterFormState {
    pub current_field: FilterFormField,
    pub tags: Editor,
    pub archived_index: usize, // 0=Active, 1=Archived, 2=All
    pub status_index: usize, // 0=Todo, 1=Done, 2=All
    pub tag_logic_index: usize, // 0=AND, 1=OR
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotebookModalMode {
    View,
    Add,
    Rename,
    Delete,
}

#[derive(Debug, Clone)]
pub enum NotebookModalField {
    NotebookList,
    ActionsList,
}

#[derive(Debug, Clone)]
pub struct NotebookModalState {
    pub mode: NotebookModalMode,
    pub selected_index: usize, // 0 = "[None]", 1+ = actual notebooks
    pub actions_selected_index: usize, // 0 = Add, 1 = Rename, 2 = Delete, 3 = Switch
    pub name_editor: Editor,
    pub list_state: ListState,
    pub current_field: NotebookModalField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    View,
    Search,
    Help,
    Create,
    Settings,
    MarkdownHelp,
    Filter,
    NotebookModal,
}

#[derive(Debug, Clone)]
pub enum ItemForm {
    Task(TaskForm),
    Note(NoteForm),
    Journal(JournalForm),
}

// Keep CreateForm as an alias for backward compatibility during refactoring
pub type CreateForm = ItemForm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskField {
    Title,
    Description,
    DueDate,
    Tags,
    Notebook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteField {
    Title,
    Tags,
    Notebook,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalField {
    Date,
    Title,
    Tags,
    Notebook,
    Content,
}

#[derive(Debug, Clone)]
pub struct TaskForm {
    pub current_field: TaskField,
    pub title: Editor,
    pub description: Editor,
    pub due_date: Editor,
    pub tags: Editor,
    pub notebook_id: Option<i64>,
    pub notebook_selected_index: usize, // 0 = "[None]", 1+ = actual notebooks
    pub editing_item_id: Option<i64>, // None for new items, Some(id) for editing
}

#[derive(Debug, Clone)]
pub struct NoteForm {
    pub current_field: NoteField,
    pub title: Editor,
    pub content: Editor,
    pub tags: Editor,
    pub notebook_id: Option<i64>,
    pub notebook_selected_index: usize, // 0 = "[None]", 1+ = actual notebooks
    pub editing_item_id: Option<i64>, // None for new items, Some(id) for editing
}

#[derive(Debug, Clone)]
pub struct JournalForm {
    pub current_field: JournalField,
    pub date: Editor,
    pub title: Editor,
    pub content: Editor,
    pub tags: Editor,
    pub notebook_id: Option<i64>,
    pub notebook_selected_index: usize, // 0 = "[None]", 1+ = actual notebooks
    pub editing_item_id: Option<i64>, // None for new items, Some(id) for editing
}

#[derive(Debug, Clone)]
pub enum SelectedItem {
    Task(Task),
    Note(Note),
    Journal(JournalEntry),
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub current_tab: Tab,
    pub sidebar_state: SidebarState,
    pub mode: Mode,
    pub selected_index: usize,
    pub list_state: ListState,
    pub selected_item: Option<SelectedItem>,
    pub item_view_scroll: usize,
    pub markdown_help_example_scroll: usize,
    pub markdown_help_rendered_scroll: usize,
    pub list_view_mode: ListViewMode,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_tab: Tab::Tasks,
            sidebar_state: SidebarState::Expanded,
            mode: Mode::View,
            selected_index: 0,
            list_state: ListState::default(),
            selected_item: None,
            item_view_scroll: 0,
            markdown_help_example_scroll: 0,
            markdown_help_rendered_scroll: 0,
            list_view_mode: ListViewMode::Simple,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub tags: Option<String>,
    pub archived: Option<FilterArchivedStatus>,
    pub task_status: Option<FilterTaskStatus>,
    pub tag_logic: FilterTagLogic,
    pub form_state: Option<FilterFormState>,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            tags: None,
            archived: Some(FilterArchivedStatus::Active),
            task_status: None,
            tag_logic: FilterTagLogic::And,
            form_state: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub category_index: usize,
    pub theme_index: usize,
    pub list_state: ListState,
    pub theme_list_state: ListState,
    pub sidebar_width_index: usize,
    pub display_mode_index: usize,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            category_index: 0,
            theme_index: 0,
            list_state: ListState::default(),
            theme_list_state: ListState::default(),
            sidebar_width_index: 0,
            display_mode_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModalState {
    pub delete_confirmation: Option<SelectedItem>,
    pub delete_modal_selection: usize,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            delete_confirmation: None,
            delete_modal_selection: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotebookState {
    pub current_notebook_id: Option<i64>,
    pub notebooks: Vec<Notebook>,
    pub modal_state: Option<NotebookModalState>,
}

impl Default for NotebookState {
    fn default() -> Self {
        Self {
            current_notebook_id: None,
            notebooks: Vec::new(),
            modal_state: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusState {
    pub message: Option<String>,
    pub message_time: Option<Instant>,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            message: None,
            message_time: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormState {
    pub create_form: Option<CreateForm>,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            create_form: None,
        }
    }
}

pub struct App {
    // Core infrastructure
    pub config: Config,
    pub database: Database,
    
    // Data collections (frequently accessed, keep at top level)
    pub tasks: Vec<Task>,
    pub notes: Vec<Note>,
    pub journals: Vec<JournalEntry>,
    
    // Grouped state
    pub ui: UiState,
    pub filter: FilterState,
    pub settings: SettingsState,
    pub modals: ModalState,
    pub notebooks: NotebookState,
    pub status: StatusState,
    pub search: SearchState,
    pub form: FormState,
}

impl App {
    pub fn new(config: Config, database: Database) -> Result<Self, DatabaseError> {
        // Load list_view_mode from config before moving config
        let list_view_mode = match config.list_view_mode.as_str() {
            "Simple" => ListViewMode::Simple,
            "TwoLine" => ListViewMode::TwoLine,
            "GroupedByTags" => ListViewMode::GroupedByTags,
            _ => ListViewMode::Simple,
        };
        
        // Load notebooks from database
        let notebooks = database.get_all_notebooks()?;
        
        // If no notebooks exist, create a "Default" notebook
        let notebooks = if notebooks.is_empty() {
            let default_notebook = Notebook::new("Default".to_string());
            let notebook_id = database.insert_notebook(&default_notebook)?;
            vec![Notebook {
                id: Some(notebook_id),
                ..default_notebook
            }]
        } else {
            notebooks
        };
        
        let mut app = Self {
            config,
            database,
            tasks: Vec::new(),
            notes: Vec::new(),
            journals: Vec::new(),
            ui: UiState {
                current_tab: Tab::Tasks,
                sidebar_state: SidebarState::Expanded,
                mode: Mode::View,
                selected_index: 0,
                list_state: ListState::default(),
                selected_item: None,
                item_view_scroll: 0,
                markdown_help_example_scroll: 0,
                markdown_help_rendered_scroll: 0,
                list_view_mode,
            },
            filter: FilterState {
                tags: None,
                archived: Some(FilterArchivedStatus::Active),
                task_status: None,
                tag_logic: FilterTagLogic::And,
                form_state: None,
            },
            settings: SettingsState {
                category_index: 0,
                theme_index: 0,
                list_state: ListState::default(),
                theme_list_state: ListState::default(),
                sidebar_width_index: 0,
                display_mode_index: 0,
            },
            modals: ModalState {
                delete_confirmation: None,
                delete_modal_selection: 0,
            },
            notebooks: NotebookState {
                current_notebook_id: None, // Start with "[None]" selected
                notebooks,
                modal_state: None,
            },
            status: StatusState {
                message: None,
                message_time: None,
            },
            search: SearchState {
                query: String::new(),
            },
            form: FormState {
                create_form: None,
            },
        };
        
        app.load_data()?;
        app.sync_list_state();
        // Auto-select the first item if available
        app.select_current_item();
        Ok(app)
    }

    pub fn load_data(&mut self) -> Result<(), DatabaseError> {
        // Load archived items if filter requires them
        let need_archived = matches!(self.filter.archived, Some(FilterArchivedStatus::Archived) | Some(FilterArchivedStatus::All));
        
        if need_archived {
            self.tasks = self.database.get_all_tasks_including_archived(self.notebooks.current_notebook_id)?;
            self.notes = self.database.get_all_notes_including_archived(self.notebooks.current_notebook_id)?;
            self.journals = self.database.get_all_journals_including_archived(self.notebooks.current_notebook_id)?;
        } else {
            self.tasks = self.database.get_all_tasks(self.notebooks.current_notebook_id)?;
            self.notes = self.database.get_all_notes(self.notebooks.current_notebook_id)?;
            self.journals = self.database.get_all_journals(self.notebooks.current_notebook_id)?;
        }
        
        // Assign order values to tasks that don't have them (migration)
        // Check if all tasks have order 0 (need migration)
        let all_zero = self.tasks.iter().all(|t| t.order == 0);
        if all_zero && !self.tasks.is_empty() {
            // Update all tasks with sequential order values
            for (index, task) in self.tasks.iter_mut().enumerate() {
                task.order = index as i64;
            }
            
            // Update tasks in database with new order values
            for task in &self.tasks {
                if let Some(task_id) = task.id {
                    self.database.update_task_order(task_id, task.order)?;
                }
            }
            // Reload to get updated data, respecting the archived filter
            if need_archived {
                self.tasks = self.database.get_all_tasks_including_archived(self.notebooks.current_notebook_id)?;
            } else {
                self.tasks = self.database.get_all_tasks(self.notebooks.current_notebook_id)?;
            }
        }
        
        // Ensure selected_index is within bounds
        self.adjust_selected_index();
        
        // Always select an item if there are items available (especially for tasks)
        if self.ui.current_tab == Tab::Tasks && !self.tasks.is_empty() {
            self.select_current_item();
        }
        
        Ok(())
    }

    pub fn get_current_items(&self) -> Vec<Item> {
        // Create base iterator from current tab (lazy, no allocation yet)
        let base_iter: Box<dyn Iterator<Item = Item>> = match self.ui.current_tab {
            Tab::Tasks => Box::new(self.tasks.iter().map(|t| Item::Task(t.clone()))),
            Tab::Notes => Box::new(self.notes.iter().map(|n| Item::Note(n.clone()))),
            Tab::Journal => Box::new(self.journals.iter().map(|j| Item::Journal(j.clone()))),
        };

        // Chain all filters into a single iterator (lazy, no allocation until collect)
        let filtered_iter = base_iter
            // Filter by search query if in search mode
            .filter(|item: &Item| {
                if self.ui.mode == Mode::Search && !self.search.query.is_empty() {
                    item.matches_search(&self.search.query)
                } else {
                    true
                }
            })
            // Filter by archived status
            .filter(|item: &Item| {
                if let Some(archived_status) = self.filter.archived {
                    let item_archived = match item {
                        Item::Task(t) => t.archived,
                        Item::Note(n) => n.archived,
                        Item::Journal(j) => j.archived,
                    };
                    match archived_status {
                        FilterArchivedStatus::Active => !item_archived,
                        FilterArchivedStatus::Archived => item_archived,
                        FilterArchivedStatus::All => true,
                    }
                } else {
                    true
                }
            })
            // Filter by tags
            .filter(|item: &Item| {
                if let Some(ref filter_tags) = self.filter.tags {
                    if !filter_tags.trim().is_empty() {
                        item.matches_tag_filter(filter_tags, self.filter.tag_logic)
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            // Filter by task status (only for tasks)
            .filter(|item: &Item| {
                if self.ui.current_tab == Tab::Tasks {
                    if let Some(task_status) = self.filter.task_status {
                        match item {
                            Item::Task(t) => {
                                match task_status {
                                    FilterTaskStatus::Todo => t.status == "todo",
                                    FilterTaskStatus::Done => t.status == "done",
                                    FilterTaskStatus::All => true,
                                }
                            }
                            _ => true, // Non-task items are not affected
                        }
                    } else {
                        true
                    }
                } else {
                    true
                }
            });

        // Collect only once at the end - only items that pass all filters are cloned
        filtered_iter.collect()
    }

    pub fn get_current_item(&self) -> Option<&SelectedItem> {
        self.ui.selected_item.as_ref()
    }

    /// Get the display index to item index mapping for GroupedByTags mode
    /// Returns a vector where each element indicates if that display index is a heading
    /// and a mapping from display index to item index (for non-heading indices)
    fn get_display_index_mapping(&self) -> (Vec<bool>, Vec<Option<usize>>) {
        use crate::tui::widgets::tags::parse_tags;
        use std::collections::HashMap;

        let items = self.get_current_items();
        
        if self.ui.list_view_mode != ListViewMode::GroupedByTags {
            // For non-grouped modes, all display indices map directly to item indices
            let is_heading: Vec<bool> = vec![false; items.len()];
            let item_indices: Vec<Option<usize>> = (0..items.len()).map(Some).collect();
            return (is_heading, item_indices);
        }

        // Build the same structure as the displayed list
        let mut is_heading: Vec<bool> = Vec::new();
        let mut item_indices: Vec<Option<usize>> = Vec::new();

        // Group items by tags
        let mut tag_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut untagged: Vec<usize> = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            let tags = match item {
                Item::Task(task) => parse_tags(task.tags.as_ref()),
                Item::Note(note) => parse_tags(note.tags.as_ref()),
                Item::Journal(journal) => parse_tags(journal.tags.as_ref()),
            };

            if tags.is_empty() {
                untagged.push(idx);
            } else {
                for tag in tags {
                    tag_map.entry(tag).or_insert_with(Vec::new).push(idx);
                }
            }
        }

        // Sort tags alphabetically
        let mut sorted_tags: Vec<String> = tag_map.keys().cloned().collect();
        sorted_tags.sort();

        // Add untagged section if there are untagged items
        if !untagged.is_empty() {
            is_heading.push(true); // [Untagged] heading
            item_indices.push(None);
            
            for item_idx in &untagged {
                is_heading.push(false);
                item_indices.push(Some(*item_idx));
            }
        }

        // Add tagged sections
        for tag in sorted_tags {
            is_heading.push(true); // [tag] heading
            item_indices.push(None);
            
            for item_idx in &tag_map[&tag] {
                is_heading.push(false);
                item_indices.push(Some(*item_idx));
            }
        }

        (is_heading, item_indices)
    }

    pub fn select_current_item(&mut self) {
        let items = self.get_current_items();
        
        if self.ui.list_view_mode == ListViewMode::GroupedByTags {
            let (is_heading, item_indices) = self.get_display_index_mapping();
            
            // Check if current display index is valid
            if self.ui.selected_index >= is_heading.len() {
                self.ui.selected_item = None;
                return;
            }
            
            // If it's a heading, don't select an item
            if let Some(&is_heading_val) = is_heading.get(self.ui.selected_index) {
                if is_heading_val {
                    self.ui.selected_item = None;
                    return;
                }
            }
            
            // Map display index to item index
            if let Some(Some(item_idx)) = item_indices.get(self.ui.selected_index) {
                if let Some(item) = items.get(*item_idx) {
                    self.ui.selected_item = Some(match item {
                        Item::Task(task) => {
                            SelectedItem::Task(task.clone())
                        }
                        Item::Note(note) => {
                            SelectedItem::Note(note.clone())
                        }
                        Item::Journal(journal) => {
                            SelectedItem::Journal(journal.clone())
                        }
                    });
                    // Only change mode to View if not in Search mode (navigation in search should keep search mode)
                    if self.ui.mode != Mode::Search {
                        self.ui.mode = Mode::View;
                    }
                    self.ui.item_view_scroll = 0;
                } else {
                    self.ui.selected_item = None;
                }
            } else {
                self.ui.selected_item = None;
            }
        } else {
            // For non-grouped modes, use direct indexing
            if !items.is_empty() {
                // Ensure selected_index is valid
                // BUT: If we have a valid selected_item, try to find its index instead of resetting
                if self.ui.selected_index >= items.len() {
                    if let Some(ref selected) = self.ui.selected_item {
                        // Try to find the selected item's index
                        let found_idx = items.iter().position(|item| {
                            match (item, selected) {
                                (Item::Task(t), SelectedItem::Task(st)) => t.id == st.id,
                                (Item::Note(n), SelectedItem::Note(sn)) => n.id == sn.id,
                                (Item::Journal(j), SelectedItem::Journal(sj)) => j.id == sj.id,
                                _ => false,
                            }
                        });
                        if let Some(idx) = found_idx {
                            self.ui.selected_index = idx;
                        } else {
                            self.ui.selected_index = 0;
                        }
                    } else {
                        self.ui.selected_index = 0;
                    }
                }
                
                if let Some(item) = items.get(self.ui.selected_index) {
                    self.ui.selected_item = Some(match item {
                        Item::Task(task) => {
                            SelectedItem::Task(task.clone())
                        }
                        Item::Note(note) => {
                            SelectedItem::Note(note.clone())
                        }
                        Item::Journal(journal) => {
                            SelectedItem::Journal(journal.clone())
                        }
                    });
                    // Only change mode to View if not in Search mode (navigation in search should keep search mode)
                    if self.ui.mode != Mode::Search {
                        self.ui.mode = Mode::View;
                    }
                    // Reset scroll when selecting a new item
                    self.ui.item_view_scroll = 0;
                }
            } else {
                // No items available, clear selection
                self.ui.selected_item = None;
            }
        }
    }

    pub fn adjust_selected_index(&mut self) {
        if self.ui.list_view_mode == ListViewMode::GroupedByTags {
            let (is_heading, _) = self.get_display_index_mapping();
            let display_len = is_heading.len();
            
            if display_len == 0 {
                self.ui.selected_index = 0;
                self.ui.selected_item = None;
            } else {
                // Ensure we have a valid selection - if index is out of bounds, select first item
                if self.ui.selected_index >= display_len {
                    self.ui.selected_index = 0;
                } else {
                    self.ui.selected_index = cmp::min(self.ui.selected_index, display_len.saturating_sub(1));
                }
                
                // Skip headings - find the first non-heading item if current selection is a heading
                if is_heading[self.ui.selected_index] {
                    // Find the first non-heading item
                    let mut found = false;
                    for i in 0..display_len {
                        if !is_heading[i] {
                            self.ui.selected_index = i;
                            found = true;
                            break;
                        }
                    }
                    // If no non-heading items exist, clear selection
                    if !found {
                        self.ui.selected_index = 0;
                        self.ui.selected_item = None;
                    }
                }
            }
        } else {
            let items = self.get_current_items();
            if items.is_empty() {
                self.ui.selected_index = 0;
                self.ui.selected_item = None;
            } else {
                // Ensure we have a valid selection - if index is out of bounds, select first item
                // BUT: If we have a valid selected_item, try to find its index instead of resetting
                if self.ui.selected_index >= items.len() {
                    // If we have a selected_item, try to find its index in the items list
                    if let Some(ref selected) = self.ui.selected_item {
                        let found_idx = items.iter().position(|item| {
                            match (item, selected) {
                                (Item::Task(t), SelectedItem::Task(st)) => t.id == st.id,
                                (Item::Note(n), SelectedItem::Note(sn)) => n.id == sn.id,
                                (Item::Journal(j), SelectedItem::Journal(sj)) => j.id == sj.id,
                                _ => false,
                            }
                        });
                        if let Some(idx) = found_idx {
                            self.ui.selected_index = idx;
                        } else {
                            // Couldn't find the item, reset to 0
                            self.ui.selected_index = 0;
                        }
                    } else {
                        self.ui.selected_index = 0;
                    }
                } else if self.ui.selected_index == 0 && self.ui.selected_item.is_some() {
                    // If selected_index is 0 but we have a selected_item, try to find its index
                    // This handles the case where selected_index was reset to 0 but we have the correct item
                    if let Some(ref selected) = self.ui.selected_item {
                        let found_idx = items.iter().position(|item| {
                            match (item, selected) {
                                (Item::Task(t), SelectedItem::Task(st)) => t.id == st.id,
                                (Item::Note(n), SelectedItem::Note(sn)) => n.id == sn.id,
                                (Item::Journal(j), SelectedItem::Journal(sj)) => j.id == sj.id,
                                _ => false,
                            }
                        });
                        if let Some(idx) = found_idx {
                            self.ui.selected_index = idx;
                        }
                    }
                } else {
                    self.ui.selected_index = cmp::min(self.ui.selected_index, items.len().saturating_sub(1));
                }
            }
        }
        
        self.sync_list_state();
    }

    /// Sync ListState with selected_index for proper scrolling
    pub fn sync_list_state(&mut self) {
        self.ui.list_state.select(Some(self.ui.selected_index));
    }

    pub fn move_selection_up(&mut self) {
        if self.ui.list_view_mode == ListViewMode::GroupedByTags {
            let (is_heading, _) = self.get_display_index_mapping();
            
            // Find the previous non-heading item
            let mut new_index = self.ui.selected_index;
            loop {
                if new_index == 0 {
                    break;
                }
                new_index -= 1;
                if !is_heading[new_index] {
                    self.ui.selected_index = new_index;
                    self.sync_list_state();
                    self.select_current_item();
                    return;
                }
            }
            // If we couldn't find a non-heading item above, we're already at the top
            // Don't change selection - stay where we are
        } else {
            if self.ui.selected_index > 0 {
                self.ui.selected_index -= 1;
                self.sync_list_state();
                // Auto-select the item when navigating
                self.select_current_item();
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.ui.list_view_mode == ListViewMode::GroupedByTags {
            let (is_heading, _) = self.get_display_index_mapping();
            let display_len = is_heading.len();
            
            // Find the next non-heading item, or stop at the last item
            let mut new_index = self.ui.selected_index;
            loop {
                if new_index >= display_len.saturating_sub(1) {
                    break;
                }
                new_index += 1;
                if !is_heading[new_index] {
                    self.ui.selected_index = new_index;
                    self.sync_list_state();
                    self.select_current_item();
                    return;
                }
            }
            // If we couldn't find a non-heading item, just move to the last index
            if self.ui.selected_index < display_len.saturating_sub(1) {
                self.ui.selected_index = display_len.saturating_sub(1);
                self.sync_list_state();
                self.select_current_item();
            }
        } else {
            let items = self.get_current_items();
            if self.ui.selected_index < items.len().saturating_sub(1) {
                self.ui.selected_index += 1;
                self.sync_list_state();
                // Auto-select the item when navigating
                self.select_current_item();
            }
        }
    }

    pub fn toggle_sidebar(&mut self) {
        self.ui.sidebar_state = match self.ui.sidebar_state {
            SidebarState::Expanded => SidebarState::Collapsed,
            SidebarState::Collapsed => SidebarState::Expanded,
        };
    }

    pub fn toggle_list_view_mode(&mut self) {
        self.ui.list_view_mode = match self.ui.list_view_mode {
            ListViewMode::Simple => ListViewMode::TwoLine,
            ListViewMode::TwoLine => ListViewMode::GroupedByTags,
            ListViewMode::GroupedByTags => ListViewMode::Simple,
        };
        
        // Save to config
        let mode_str = match self.ui.list_view_mode {
            ListViewMode::Simple => "Simple",
            ListViewMode::TwoLine => "TwoLine",
            ListViewMode::GroupedByTags => "GroupedByTags",
        };
        self.config.list_view_mode = mode_str.to_string();
        if let Err(e) = self.config.save() {
            // Log error but don't fail - this is a non-critical operation
            eprintln!("Failed to save display mode: {}", e);
        }
        
        // Adjust selection to ensure it's valid for the new mode (skip headings in GroupedByTags)
        self.adjust_selected_index();
        self.select_current_item();
    }

    /// Switch to a new tab and auto-select the first item if available
    pub fn switch_tab(&mut self, new_tab: Tab) {
        self.ui.current_tab = new_tab;
        self.ui.selected_index = 0;
        self.adjust_selected_index();
        
        // Auto-select the first item if available
        // If no items exist, select_current_item() won't update selected_item,
        // so we need to clear it to avoid showing items from other tabs
        let items = self.get_current_items();
        if items.is_empty() {
            self.ui.selected_item = None;
        } else {
            self.select_current_item();
        }
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status.message = Some(message);
        self.status.message_time = Some(Instant::now());
    }

    pub fn clear_status_message(&mut self) {
        self.status.message = None;
        self.status.message_time = None;
    }

    /// Check if status message should be auto-cleared (after 3 seconds)
    pub fn check_status_message_timeout(&mut self) {
        const STATUS_MESSAGE_TIMEOUT_SECS: u64 = 3;
        if let Some(time) = self.status.message_time {
            if time.elapsed().as_secs() >= STATUS_MESSAGE_TIMEOUT_SECS {
                self.clear_status_message();
            }
        }
    }

    pub fn enter_search_mode(&mut self) {
        self.ui.mode = Mode::Search;
        self.search.query.clear();
    }

    pub fn exit_search_mode(&mut self) {
        // Get selected item ID - prefer using selected_item if available (from navigation),
        // otherwise use selected_index to look it up from filtered list
        let selected_item_id = if let Some(ref selected) = self.ui.selected_item {
            // Use already-selected item if available
            match selected {
                SelectedItem::Task(t) => t.id.map(|id| ("Task", id)),
                SelectedItem::Note(n) => n.id.map(|id| ("Note", id)),
                SelectedItem::Journal(j) => j.id.map(|id| ("Journal", id)),
            }
        } else {
            // Fall back to looking up by selected_index in filtered list
            let filtered_items = self.get_current_items();
            if !filtered_items.is_empty() && self.ui.selected_index < filtered_items.len() {
                if let Some(item) = filtered_items.get(self.ui.selected_index) {
                    match item {
                        Item::Task(t) => t.id.map(|id| ("Task", id)),
                        Item::Note(n) => n.id.map(|id| ("Note", id)),
                        Item::Journal(j) => j.id.map(|id| ("Journal", id)),
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
        
        self.ui.mode = Mode::View;
        self.search.query.clear();
        
        // Map selected_index from filtered list to full list
        if let Some((item_type, item_id)) = selected_item_id {
            let full_items = self.get_current_items();
            let new_item_index = full_items.iter().position(|item| {
                match (item, item_type) {
                    (Item::Task(t), "Task") => t.id == Some(item_id),
                    (Item::Note(n), "Note") => n.id == Some(item_id),
                    (Item::Journal(j), "Journal") => j.id == Some(item_id),
                    _ => false,
                }
            });
            
            if let Some(item_idx) = new_item_index {
                if self.ui.list_view_mode == ListViewMode::GroupedByTags {
                    // In GroupedByTags mode, we need to find the display index that corresponds to this item index
                    let (_, item_indices) = self.get_display_index_mapping();
                    if let Some(display_idx) = item_indices.iter().position(|&idx_opt| idx_opt == Some(item_idx)) {
                        self.ui.selected_index = display_idx;
                    } else {
                        // Fallback: set to 0 if we can't find the display index
                        self.ui.selected_index = 0;
                    }
                } else {
                    // For non-grouped modes, item index is the same as display index
                    self.ui.selected_index = item_idx;
                }
            }
        }
        
        // Don't call adjust_selected_index() here - we've already calculated the correct index
        // adjust_selected_index() might reset it incorrectly if called from elsewhere
        self.sync_list_state();
        // Auto-select the current item after exiting search
        self.select_current_item();
    }

    pub fn enter_filter_mode(&mut self) {
        self.ui.mode = Mode::Filter;
        // Initialize filter form state with current filter values
        let tags_str = self.filter.tags.clone().unwrap_or_default();
        let archived_index = match self.filter.archived {
            Some(FilterArchivedStatus::Active) => 0,
            Some(FilterArchivedStatus::Archived) => 1,
            Some(FilterArchivedStatus::All) => 2,
            None => 0,
        };
        let status_index = match self.filter.task_status {
            Some(FilterTaskStatus::Todo) => 0,
            Some(FilterTaskStatus::Done) => 1,
            Some(FilterTaskStatus::All) => 2,
            None => 2, // Default to All if not set
        };
        let tag_logic_index = match self.filter.tag_logic {
            FilterTagLogic::And => 0,
            FilterTagLogic::Or => 1,
        };
        self.filter.form_state = Some(FilterFormState {
            current_field: FilterFormField::Tags,
            tags: Editor::from_string(tags_str),
            archived_index,
            status_index,
            tag_logic_index,
        });
    }

    pub fn exit_filter_mode(&mut self) {
        self.ui.mode = Mode::View;
        self.filter.form_state = None;
    }

    pub fn apply_filters(&mut self) {
        if let Some(ref state) = self.filter.form_state {
            // Apply tags filter
            let tags_content = if state.tags.lines.is_empty() {
                String::new()
            } else {
                state.tags.lines[0].clone()
            };
            self.filter.tags = if tags_content.trim().is_empty() {
                None
            } else {
                Some(tags_content.trim().to_string())
            };

            // Apply archived filter
            self.filter.archived = match state.archived_index {
                0 => Some(FilterArchivedStatus::Active),
                1 => Some(FilterArchivedStatus::Archived),
                2 => Some(FilterArchivedStatus::All),
                _ => None,
            };

            // Apply tag logic
            self.filter.tag_logic = match state.tag_logic_index {
                0 => FilterTagLogic::And,
                1 => FilterTagLogic::Or,
                _ => FilterTagLogic::And,
            };

            // Apply task status filter (only relevant for Tasks tab)
            if self.ui.current_tab == Tab::Tasks {
                self.filter.task_status = match state.status_index {
                    0 => Some(FilterTaskStatus::Todo),
                    1 => Some(FilterTaskStatus::Done),
                    2 => Some(FilterTaskStatus::All),
                    _ => Some(FilterTaskStatus::All),
                };
            } else {
                // Clear task status filter when not on Tasks tab
                self.filter.task_status = None;
            }

            // Reload data if needed (to get archived items)
            if let Some(FilterArchivedStatus::Archived) | Some(FilterArchivedStatus::All) = self.filter.archived {
                if let Err(e) = self.load_data() {
                    self.set_status_message(format!("Failed to reload data: {}", e));
                }
            }

            // Reset selection and update display
            self.ui.selected_index = 0;
            self.adjust_selected_index();
            self.select_current_item();
            self.set_status_message("Filters applied".to_string());
        }
        self.exit_filter_mode();
    }

    pub fn clear_filters(&mut self) {
        self.filter.tags = None;
        self.filter.archived = Some(FilterArchivedStatus::Active);
        self.filter.task_status = None;
        // Reload data to get only active items
        if let Err(e) = self.load_data() {
            self.set_status_message(format!("Failed to reload data: {}", e));
        } else {
            self.ui.selected_index = 0;
            self.adjust_selected_index();
            self.select_current_item();
            self.set_status_message("Filters cleared".to_string());
        }
    }

    pub fn get_filter_summary(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref tags) = self.filter.tags {
            if !tags.trim().is_empty() {
                parts.push(format!("Tags: {}", tags));
            }
        }
        
        if let Some(archived) = self.filter.archived {
            let archived_str = match archived {
                FilterArchivedStatus::Active => "Active",
                FilterArchivedStatus::Archived => "Archived",
                FilterArchivedStatus::All => "All",
            };
            parts.push(format!("Archived: {}", archived_str));
        }
        
        if let Some(status) = self.filter.task_status {
            let status_str = match status {
                FilterTaskStatus::Todo => "Todo",
                FilterTaskStatus::Done => "Done",
                FilterTaskStatus::All => "All",
            };
            parts.push(format!("Status: {}", status_str));
        }
        
        let logic_str = match self.filter.tag_logic {
            FilterTagLogic::And => "AND",
            FilterTagLogic::Or => "OR",
        };
        if self.filter.tags.is_some() {
            parts.push(format!("Logic: {}", logic_str));
        }
        
        if parts.is_empty() {
            "No filters".to_string()
        } else {
            parts.join(" | ")
        }
    }

    pub fn navigate_filter_field(&mut self, forward: bool) {
        if let Some(ref mut state) = self.filter.form_state {
            // Build fields list - include Status only when on Tasks tab
            let mut fields = vec![
                FilterFormField::Tags,
                FilterFormField::Archived,
            ];
            
            // Add Status field only for Tasks tab
            if self.ui.current_tab == Tab::Tasks {
                fields.push(FilterFormField::Status);
            }
            
            fields.extend(vec![
                FilterFormField::TagLogic,
                FilterFormField::Apply,
                FilterFormField::Clear,
                FilterFormField::Cancel,
            ]);
            
            let current_idx = fields.iter()
                .position(|f| std::mem::discriminant(f) == std::mem::discriminant(&state.current_field))
                .unwrap_or(0);
            
            let new_idx = if forward {
                (current_idx + 1) % fields.len()
            } else {
                (current_idx + fields.len() - 1) % fields.len()
            };
            
            state.current_field = fields[new_idx].clone();
        }
    }

    pub fn get_current_filter_editor(&mut self) -> Option<&mut Editor> {
        if let Some(ref mut state) = self.filter.form_state {
            if matches!(state.current_field, FilterFormField::Tags) {
                Some(&mut state.tags)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_filter_tags_field_active(&self) -> bool {
        if let Some(ref state) = self.filter.form_state {
            matches!(state.current_field, FilterFormField::Tags)
        } else {
            false
        }
    }

    pub fn move_filter_archived_up(&mut self) {
        if let Some(ref mut state) = self.filter.form_state {
            if state.archived_index > 0 {
                state.archived_index -= 1;
            }
        }
    }

    pub fn move_filter_archived_down(&mut self) {
        if let Some(ref mut state) = self.filter.form_state {
            if state.archived_index < 2 {
                state.archived_index += 1;
            }
        }
    }

    pub fn move_filter_tag_logic_up(&mut self) {
        if let Some(ref mut state) = self.filter.form_state {
            if state.tag_logic_index > 0 {
                state.tag_logic_index -= 1;
            }
        }
    }

    pub fn move_filter_tag_logic_down(&mut self) {
        if let Some(ref mut state) = self.filter.form_state {
            if state.tag_logic_index < 1 {
                state.tag_logic_index += 1;
            }
        }
    }

    pub fn move_filter_status_up(&mut self) {
        if let Some(ref mut state) = self.filter.form_state {
            if state.status_index > 0 {
                state.status_index -= 1;
            }
        }
    }

    pub fn move_filter_status_down(&mut self) {
        if let Some(ref mut state) = self.filter.form_state {
            if state.status_index < 2 {
                state.status_index += 1;
            }
        }
    }

    pub fn enter_help_mode(&mut self) {
        self.ui.mode = Mode::Help;
    }

    pub fn exit_help_mode(&mut self) {
        self.ui.mode = Mode::View;
    }

    pub fn enter_markdown_help_mode(&mut self) {
        self.ui.mode = Mode::MarkdownHelp;
        // Reset scroll positions when entering markdown help
        self.ui.markdown_help_example_scroll = 0;
        self.ui.markdown_help_rendered_scroll = 0;
    }

    pub fn exit_markdown_help_mode(&mut self) {
        // Return to Create mode when exiting markdown help
        self.ui.mode = Mode::Create;
    }

    pub fn enter_settings_mode(&mut self) {
        self.ui.mode = Mode::Settings;
        self.init_settings_state();
    }

    /// Move settings category selection up
    pub fn move_settings_category_up(&mut self) {
        if self.settings.category_index > 0 {
            self.settings.category_index -= 1;
            self.settings.list_state.select(Some(self.settings.category_index));
        }
    }

    /// Move settings category selection down
    pub fn move_settings_category_down(&mut self) {
        let categories = self.get_settings_categories();
        if self.settings.category_index < categories.len().saturating_sub(1) {
            self.settings.category_index += 1;
            self.settings.list_state.select(Some(self.settings.category_index));
        }
    }

    /// Get available sidebar width options
    pub fn get_sidebar_width_options(&self) -> Vec<u16> {
        vec![20, 25, 30, 35, 40]
    }

    /// Move sidebar width selection up
    pub fn move_settings_sidebar_width_up(&mut self) {
        if self.settings.sidebar_width_index > 0 {
            self.settings.sidebar_width_index -= 1;
        }
    }

    /// Move sidebar width selection down
    pub fn move_settings_sidebar_width_down(&mut self) {
        let options = self.get_sidebar_width_options();
        if self.settings.sidebar_width_index < options.len().saturating_sub(1) {
            self.settings.sidebar_width_index += 1;
        }
    }

    /// Apply selected sidebar width
    pub fn apply_sidebar_width(&mut self) -> Result<(), crate::config::ConfigError> {
        let options = self.get_sidebar_width_options();
        if let Some(&width) = options.get(self.settings.sidebar_width_index) {
            self.config.sidebar_width_percent = width;
            self.config.save()?;
            self.set_status_message(format!("Sidebar width set to {}%", width));
        }
        Ok(())
    }

    /// Get display mode options
    pub fn get_display_mode_options(&self) -> Vec<&'static str> {
        vec!["Simple", "TwoLine", "GroupedByTags"]
    }

    /// Move display mode selection up
    pub fn move_settings_display_mode_up(&mut self) {
        if self.settings.display_mode_index > 0 {
            self.settings.display_mode_index -= 1;
        }
    }

    /// Move display mode selection down
    pub fn move_settings_display_mode_down(&mut self) {
        let options = self.get_display_mode_options();
        if self.settings.display_mode_index < options.len().saturating_sub(1) {
            self.settings.display_mode_index += 1;
        }
    }

    /// Apply selected display mode
    pub fn apply_display_mode(&mut self) -> Result<(), crate::config::ConfigError> {
        let options = self.get_display_mode_options();
        if let Some(&mode_str) = options.get(self.settings.display_mode_index) {
            let new_mode = match mode_str {
                "Simple" => ListViewMode::Simple,
                "TwoLine" => ListViewMode::TwoLine,
                "GroupedByTags" => ListViewMode::GroupedByTags,
                _ => return Ok(()), // Invalid mode, do nothing
            };
            
            self.ui.list_view_mode = new_mode;
            self.config.list_view_mode = mode_str.to_string();
            // Determine profile based on database path (same logic as get_config_file_path)
            let db_path = self.config.get_database_path();
            let db_path_str = db_path.to_string_lossy();
            let profile = if db_path_str.contains("tnj-dev") {
                crate::Profile::Dev
            } else {
                crate::Profile::Prod
            };
            self.config.save_with_profile(profile)?;
            
            // Adjust selection to ensure it's valid for the new mode
            self.adjust_selected_index();
            self.select_current_item();
            
            self.set_status_message(format!("Display mode set to: {}", mode_str));
        }
        Ok(())
    }

    pub fn exit_settings_mode(&mut self) {
        self.ui.mode = Mode::View;
    }

    pub fn add_to_search(&mut self, ch: char) {
        self.search.query.push(ch);
        self.ui.selected_index = 0; // Reset to top when searching
        self.sync_list_state();
    }

    pub fn remove_from_search(&mut self) {
        self.search.query.pop();
        self.ui.selected_index = 0; // Reset to top when searching
        self.sync_list_state();
    }

    pub fn enter_edit_mode(&mut self) {
        // Use form-based editing instead of single-field editing
        // Populate form with existing item data
        if let Some(ref item) = self.ui.selected_item {
            let form = match item {
                SelectedItem::Task(task) => {
                    let notebook_id = task.notebook_id;
                    let notebook_selected_index = self.get_notebook_index_for_id(notebook_id);
                    CreateForm::Task(TaskForm {
                        current_field: TaskField::Title,
                        title: Editor::from_string(task.title.clone()),
                        description: Editor::from_string(task.description.clone().unwrap_or_default()),
                        due_date: Editor::from_string(task.due_date.clone().unwrap_or_default()),
                        tags: Editor::from_string(task.tags.clone().unwrap_or_default()),
                        notebook_id,
                        notebook_selected_index,
                        editing_item_id: task.id,
                    })
                }
                SelectedItem::Note(note) => {
                    let notebook_id = note.notebook_id;
                    let notebook_selected_index = self.get_notebook_index_for_id(notebook_id);
                    CreateForm::Note(NoteForm {
                        current_field: NoteField::Title,
                        title: Editor::from_string(note.title.clone()),
                        tags: Editor::from_string(note.tags.clone().unwrap_or_default()),
                        content: Editor::from_string(note.content.clone().unwrap_or_default()),
                        notebook_id,
                        notebook_selected_index,
                        editing_item_id: note.id,
                    })
                }
                SelectedItem::Journal(journal) => {
                    let notebook_id = journal.notebook_id;
                    let notebook_selected_index = self.get_notebook_index_for_id(notebook_id);
                    CreateForm::Journal(JournalForm {
                        current_field: JournalField::Date,
                        date: Editor::from_string(journal.date.clone()),
                        title: Editor::from_string(journal.title.clone().unwrap_or_default()),
                        content: Editor::from_string(journal.content.clone().unwrap_or_default()),
                        tags: Editor::from_string(journal.tags.clone().unwrap_or_default()),
                        notebook_id,
                        notebook_selected_index,
                        editing_item_id: journal.id,
                    })
                }
            };
            self.form.create_form = Some(form);
            self.ui.mode = Mode::Create; // Use Create mode for form-based editing
        } else {
            self.set_status_message("No item selected".to_string());
        }
    }


    pub fn enter_create_mode(&mut self) {
        let notebook_id = self.notebooks.current_notebook_id;
        let notebook_selected_index = self.get_notebook_index_for_id(notebook_id);
        let form = match self.ui.current_tab {
            Tab::Tasks => {
                CreateForm::Task(TaskForm {
                    current_field: TaskField::Title,
                    title: Editor::new(),
                    description: Editor::new(),
                    due_date: Editor::new(),
                    tags: Editor::new(),
                    notebook_id,
                    notebook_selected_index,
                    editing_item_id: None,
                })
            }
            Tab::Notes => {
                CreateForm::Note(NoteForm {
                    current_field: NoteField::Title,
                    title: Editor::new(),
                    tags: Editor::new(),
                    content: Editor::new(),
                    notebook_id,
                    notebook_selected_index,
                    editing_item_id: None,
                })
            }
            Tab::Journal => {
                // Default date to today
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                CreateForm::Journal(JournalForm {
                    current_field: JournalField::Date,
                    date: Editor::from_string(today),
                    title: Editor::new(),
                    content: Editor::new(),
                    tags: Editor::new(),
                    notebook_id,
                    notebook_selected_index,
                    editing_item_id: None,
                })
            }
        };
        self.form.create_form = Some(form);
        self.ui.mode = Mode::Create;
    }

    /// Get notebook index for a given notebook_id (0 = "[None]", 1+ = actual notebooks)
    fn get_notebook_index_for_id(&self, notebook_id: Option<i64>) -> usize {
        if let Some(nb_id) = notebook_id {
            self.notebooks.notebooks
                .iter()
                .position(|n| n.id == Some(nb_id))
                .map(|idx| idx + 1) // +1 because "[None]" is at index 0
                .unwrap_or(0)
        } else {
            0 // "[None]" is selected
        }
    }

    pub fn exit_create_mode(&mut self) {
        self.form.create_form = None;
        self.ui.mode = Mode::View;
    }

    pub fn navigate_form_field(&mut self, forward: bool) {
        if let Some(ref mut form) = self.form.create_form {
            match form {
                CreateForm::Task(task_form) => {
                    let current = task_form.current_field;
                    task_form.current_field = match (current, forward) {
                        (TaskField::Title, true) => TaskField::Description,
                        (TaskField::Description, true) => TaskField::DueDate,
                        (TaskField::DueDate, true) => TaskField::Tags,
                        (TaskField::Tags, true) => TaskField::Notebook,
                        (TaskField::Notebook, true) => TaskField::Title, // Wrap around
                        (TaskField::Title, false) => TaskField::Notebook, // Wrap around
                        (TaskField::Description, false) => TaskField::Title,
                        (TaskField::DueDate, false) => TaskField::Description,
                        (TaskField::Tags, false) => TaskField::DueDate,
                        (TaskField::Notebook, false) => TaskField::Tags,
                    };
                }
                CreateForm::Note(note_form) => {
                    let current = note_form.current_field;
                    note_form.current_field = match (current, forward) {
                        (NoteField::Title, true) => NoteField::Tags,
                        (NoteField::Tags, true) => NoteField::Notebook,
                        (NoteField::Notebook, true) => NoteField::Content,
                        (NoteField::Content, true) => NoteField::Title, // Wrap around
                        (NoteField::Title, false) => NoteField::Content, // Wrap around
                        (NoteField::Tags, false) => NoteField::Title,
                        (NoteField::Notebook, false) => NoteField::Tags,
                        (NoteField::Content, false) => NoteField::Notebook,
                    };
                }
                CreateForm::Journal(journal_form) => {
                    let current = journal_form.current_field;
                    journal_form.current_field = match (current, forward) {
                        (JournalField::Date, true) => JournalField::Title,
                        (JournalField::Title, true) => JournalField::Tags,
                        (JournalField::Tags, true) => JournalField::Notebook,
                        (JournalField::Notebook, true) => JournalField::Content,
                        (JournalField::Content, true) => JournalField::Date, // Wrap around
                        (JournalField::Date, false) => JournalField::Content, // Wrap around
                        (JournalField::Title, false) => JournalField::Date,
                        (JournalField::Tags, false) => JournalField::Title,
                        (JournalField::Notebook, false) => JournalField::Tags,
                        (JournalField::Content, false) => JournalField::Notebook,
                    };
                }
            }
        }
    }

    pub fn get_current_form_editor(&mut self) -> Option<&mut Editor> {
        if let Some(ref mut form) = self.form.create_form {
            match form {
                CreateForm::Task(task_form) => {
                    match task_form.current_field {
                        TaskField::Title => Some(&mut task_form.title),
                        TaskField::Description => Some(&mut task_form.description),
                        TaskField::DueDate => Some(&mut task_form.due_date),
                        TaskField::Tags => Some(&mut task_form.tags),
                        TaskField::Notebook => None, // Notebook field doesn't use Editor
                    }
                }
                CreateForm::Note(note_form) => {
                    match note_form.current_field {
                        NoteField::Title => Some(&mut note_form.title),
                        NoteField::Tags => Some(&mut note_form.tags),
                        NoteField::Notebook => None, // Notebook field doesn't use Editor
                        NoteField::Content => Some(&mut note_form.content),
                    }
                }
                CreateForm::Journal(journal_form) => {
                    match journal_form.current_field {
                        JournalField::Date => Some(&mut journal_form.date),
                        JournalField::Title => Some(&mut journal_form.title),
                        JournalField::Tags => Some(&mut journal_form.tags),
                        JournalField::Notebook => None, // Notebook field doesn't use Editor
                        JournalField::Content => Some(&mut journal_form.content),
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn is_content_field_active(&self) -> bool {
        if let Some(ref form) = self.form.create_form {
            match form {
                CreateForm::Note(note_form) => note_form.current_field == NoteField::Content,
                CreateForm::Journal(journal_form) => journal_form.current_field == JournalField::Content,
                CreateForm::Task(task_form) => task_form.current_field == TaskField::Description,
            }
        } else {
            false
        }
    }

    pub fn is_notebook_field_active(&self) -> bool {
        if let Some(ref form) = self.form.create_form {
            match form {
                CreateForm::Task(task_form) => task_form.current_field == TaskField::Notebook,
                CreateForm::Note(note_form) => note_form.current_field == NoteField::Notebook,
                CreateForm::Journal(journal_form) => journal_form.current_field == JournalField::Notebook,
            }
        } else {
            false
        }
    }

    fn validate_task_form(&self, form: &TaskForm) -> Result<(), String> {
        let title = form.title.to_string().trim().to_string();
        if title.is_empty() {
            return Err("Title is required".to_string());
        }

        // Validate due_date format if provided
        let due_date = form.due_date.to_string().trim().to_string();
        if !due_date.is_empty() {
            if !chrono::NaiveDate::parse_from_str(&due_date, "%Y-%m-%d").is_ok() {
                return Err("Due date must be in YYYY-MM-DD format".to_string());
            }
        }

        Ok(())
    }

    fn validate_note_form(&self, form: &NoteForm) -> Result<(), String> {
        let title = form.title.to_string().trim().to_string();
        if title.is_empty() {
            return Err("Title is required".to_string());
        }
        Ok(())
    }

    fn validate_journal_form(&self, form: &JournalForm) -> Result<(), String> {
        let date = form.date.to_string().trim().to_string();
        if date.is_empty() {
            return Err("Date is required".to_string());
        }
        if !chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_ok() {
            return Err("Date must be in YYYY-MM-DD format".to_string());
        }
        Ok(())
    }

    pub fn save_create_form(&mut self) -> Result<(), DatabaseError> {
        if let Some(ref form) = self.form.create_form {
            match form {
                CreateForm::Task(task_form) => {
                    // Validate
                    if let Err(err) = self.validate_task_form(task_form) {
                        self.set_status_message(format!("Validation error: {}", err));
                        return Ok(());
                    }

                    // Extract values
                    let title = task_form.title.to_string().trim().to_string();
                    let description = task_form.description.to_string().trim().to_string();
                    let due_date = task_form.due_date.to_string().trim().to_string();
                    let tags = task_form.tags.to_string().trim().to_string();

                    if let Some(item_id) = task_form.editing_item_id {
                        // Update existing task
                        if let Some(ref mut task) = self.tasks.iter_mut().find(|t| t.id == Some(item_id)) {
                            task.title = title;
                            task.description = if description.is_empty() { None } else { Some(description) };
                            task.due_date = if due_date.is_empty() { None } else { Some(due_date) };
                            task.tags = if tags.is_empty() { None } else { Some(tags) };
                            task.notebook_id = task_form.notebook_id;
                            task.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            
                            // Update with error handling
                            if let Err(e) = self.database.update_task(task) {
                                self.set_status_message(format!("Failed to update task: {}", e));
                                return Ok(());
                            }
                            
                            if let Err(e) = self.load_data() {
                                self.set_status_message(format!("Failed to reload data: {}", e));
                                return Ok(());
                            }
                            
                            // Refresh selected item
                            if let Some(updated_task) = self.tasks.iter().find(|t| t.id == Some(item_id)) {
                                self.ui.selected_item = Some(SelectedItem::Task(updated_task.clone()));
                            }
                            self.set_status_message("Task updated".to_string());
                        } else {
                            self.set_status_message("Task not found".to_string());
                            return Ok(());
                        }
                    } else {
                        // Create new task
                        let mut task = Task::new(title);
                        task.description = if description.is_empty() { None } else { Some(description) };
                        task.due_date = if due_date.is_empty() { None } else { Some(due_date) };
                        task.tags = if tags.is_empty() { None } else { Some(tags) };
                        task.notebook_id = task_form.notebook_id;
                        
                        // Assign order value (max + 1)
                        let max_order = self.database.get_max_task_order()
                            .unwrap_or(-1);
                        task.order = max_order + 1;

                        // Insert into database with error handling
                        let task_id = match self.database.insert_task(&task) {
                            Ok(id) => id,
                            Err(e) => {
                                self.set_status_message(format!("Failed to create task: {}", e));
                                return Ok(());
                            }
                        };
                        
                        if let Err(e) = self.load_data() {
                            self.set_status_message(format!("Failed to reload data: {}", e));
                            return Ok(());
                        }
                        
                        // Find and select the newly created task
                        if let Some(new_task_index) = self.tasks.iter().position(|t| t.id == Some(task_id)) {
                            self.ui.selected_index = new_task_index;
                            self.sync_list_state();
                            if let Some(new_task) = self.tasks.get(new_task_index) {
                                self.ui.selected_item = Some(SelectedItem::Task(new_task.clone()));
                            }
                        } else {
                            // If we can't find it, just ensure something is selected
                            self.adjust_selected_index();
                            self.select_current_item();
                        }
                        
                        self.set_status_message("Task created".to_string());
                    }
                    self.exit_create_mode();
                }
                CreateForm::Note(note_form) => {
                    // Validate
                    if let Err(err) = self.validate_note_form(note_form) {
                        self.set_status_message(format!("Validation error: {}", err));
                        return Ok(());
                    }

                    // Extract values
                    let title = note_form.title.to_string().trim().to_string();
                    let content = note_form.content.to_string().trim().to_string();
                    let tags = note_form.tags.to_string().trim().to_string();

                    if let Some(item_id) = note_form.editing_item_id {
                        // Update existing note
                        if let Some(ref mut note) = self.notes.iter_mut().find(|n| n.id == Some(item_id)) {
                            note.title = title;
                            note.content = if content.is_empty() { None } else { Some(content) };
                            note.tags = if tags.is_empty() { None } else { Some(tags) };
                            note.notebook_id = note_form.notebook_id;
                            note.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            
                            // Update with error handling
                            if let Err(e) = self.database.update_note(note) {
                                self.set_status_message(format!("Failed to update note: {}", e));
                                return Ok(());
                            }
                            
                            if let Err(e) = self.load_data() {
                                self.set_status_message(format!("Failed to reload data: {}", e));
                                return Ok(());
                            }
                            
                            // Refresh selected item
                            if let Some(updated_note) = self.notes.iter().find(|n| n.id == Some(item_id)) {
                                self.ui.selected_item = Some(SelectedItem::Note(updated_note.clone()));
                            }
                            self.set_status_message("Note updated".to_string());
                        } else {
                            self.set_status_message("Note not found".to_string());
                            return Ok(());
                        }
                    } else {
                        // Create new note
                        let mut note = Note::new(title);
                        note.content = if content.is_empty() { None } else { Some(content) };
                        note.tags = if tags.is_empty() { None } else { Some(tags) };
                        note.notebook_id = note_form.notebook_id;

                        // Insert into database with error handling
                        if let Err(e) = self.database.insert_note(&note) {
                            self.set_status_message(format!("Failed to create note: {}", e));
                            return Ok(());
                        }
                        
                        if let Err(e) = self.load_data() {
                            self.set_status_message(format!("Failed to reload data: {}", e));
                            return Ok(());
                        }
                        
                        self.set_status_message("Note created".to_string());
                    }
                    self.exit_create_mode();
                }
                CreateForm::Journal(journal_form) => {
                    // Validate
                    if let Err(err) = self.validate_journal_form(journal_form) {
                        self.set_status_message(format!("Validation error: {}", err));
                        return Ok(());
                    }

                    // Extract values
                    let date = journal_form.date.to_string().trim().to_string();
                    let title = journal_form.title.to_string().trim().to_string();
                    let content = journal_form.content.to_string().trim().to_string();
                    let tags = journal_form.tags.to_string().trim().to_string();

                    if let Some(item_id) = journal_form.editing_item_id {
                        // Update existing journal entry
                        if let Some(ref mut journal) = self.journals.iter_mut().find(|j| j.id == Some(item_id)) {
                            journal.date = date;
                            journal.title = if title.is_empty() { None } else { Some(title) };
                            journal.content = if content.is_empty() { None } else { Some(content) };
                            journal.tags = if tags.is_empty() { None } else { Some(tags) };
                            journal.notebook_id = journal_form.notebook_id;
                            journal.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            
                            // Update with error handling
                            if let Err(e) = self.database.update_journal(journal) {
                                self.set_status_message(format!("Failed to update journal entry: {}", e));
                                return Ok(());
                            }
                            
                            if let Err(e) = self.load_data() {
                                self.set_status_message(format!("Failed to reload data: {}", e));
                                return Ok(());
                            }
                            
                            // Refresh selected item
                            if let Some(updated_journal) = self.journals.iter().find(|j| j.id == Some(item_id)) {
                                self.ui.selected_item = Some(SelectedItem::Journal(updated_journal.clone()));
                            }
                            self.set_status_message("Journal entry updated".to_string());
                        } else {
                            self.set_status_message("Journal entry not found".to_string());
                            return Ok(());
                        }
                    } else {
                        // Create new journal entry
                        let mut journal = JournalEntry::new(date);
                        journal.title = if title.is_empty() { None } else { Some(title) };
                        journal.content = if content.is_empty() { None } else { Some(content) };
                        journal.tags = if tags.is_empty() { None } else { Some(tags) };
                        journal.notebook_id = journal_form.notebook_id;

                        // Insert into database with error handling
                        if let Err(e) = self.database.insert_journal(&journal) {
                            self.set_status_message(format!("Failed to create journal entry: {}", e));
                            return Ok(());
                        }
                        
                        if let Err(e) = self.load_data() {
                            self.set_status_message(format!("Failed to reload data: {}", e));
                            return Ok(());
                        }
                        
                        self.set_status_message("Journal entry created".to_string());
                    }
                    self.exit_create_mode();
                }
            }
        }
        Ok(())
    }

    /// Get total number of lines for the current selected item (for scrolling calculations)
    /// This is a simplified version that doesn't account for wrapping - actual wrapping
    /// happens in render. This is just for approximate scroll calculations.
    fn get_item_view_total_lines(&self) -> usize {
        if let Some(ref item) = self.ui.selected_item {
            match item {
                SelectedItem::Task(task) => {
                    let mut count = 2; // Title, Status
                    if task.due_date.is_some() {
                        count += 1;
                    }
                    if let Some(ref description) = task.description {
                        count += 2; // Empty line + "Description:" label
                        count += description.lines().count();
                    }
                    if task.tags.is_some() {
                        count += 2; // Empty line + Tags line
                    }
                    count
                }
                SelectedItem::Note(note) => {
                    let mut count = 1; // Title
                    if let Some(ref content) = note.content {
                        count += 2; // Empty line + "Content:" label
                        count += content.lines().count();
                    }
                    if note.tags.is_some() {
                        count += 2; // Empty line + Tags line
                    }
                    count
                }
                SelectedItem::Journal(journal) => {
                    let mut count = 1; // Date
                    if journal.title.is_some() {
                        count += 1;
                    }
                    if let Some(ref content) = journal.content {
                        count += 2; // Empty line + "Content:" label
                        count += content.lines().count();
                    }
                    if journal.tags.is_some() {
                        count += 2; // Empty line + Tags line
                    }
                    count
                }
            }
        } else {
            0
        }
    }

    /// Scroll item view content up by one line
    pub fn scroll_item_view_up(&mut self) {
        if self.ui.item_view_scroll > 0 {
            self.ui.item_view_scroll -= 1;
        }
    }

    /// Scroll item view content down by one line
    pub fn scroll_item_view_down(&mut self) {
        // Just increment - render will clamp it appropriately
        self.ui.item_view_scroll += 1;
    }

    /// Scroll item view content up by one page (viewport height)
    pub fn scroll_item_view_page_up(&mut self, viewport_height: usize) {
        if self.ui.item_view_scroll >= viewport_height {
            self.ui.item_view_scroll -= viewport_height;
        } else {
            self.ui.item_view_scroll = 0;
        }
    }

    /// Scroll item view content down by one page (viewport height)
    pub fn scroll_item_view_page_down(&mut self, viewport_height: usize) {
        let total_lines = self.get_item_view_total_lines();
        let max_scroll = total_lines.saturating_sub(viewport_height);
        if self.ui.item_view_scroll + viewport_height <= max_scroll {
            self.ui.item_view_scroll += viewport_height;
        } else {
            self.ui.item_view_scroll = max_scroll;
        }
    }

    /// Scroll item view content to top
    pub fn scroll_item_view_to_top(&mut self) {
        self.ui.item_view_scroll = 0;
    }

    /// Scroll item view content to bottom
    pub fn scroll_item_view_to_bottom(&mut self, viewport_height: usize) {
        let total_lines = self.get_item_view_total_lines();
        self.ui.item_view_scroll = total_lines.saturating_sub(viewport_height);
    }

    /// Scroll markdown help example panel up by one line
    pub fn scroll_markdown_help_example_up(&mut self) {
        if self.ui.markdown_help_example_scroll > 0 {
            self.ui.markdown_help_example_scroll -= 1;
        }
    }

    /// Scroll markdown help example panel down by one line
    pub fn scroll_markdown_help_example_down(&mut self) {
        self.ui.markdown_help_example_scroll += 1;
    }

    /// Scroll markdown help rendered panel up by one line
    pub fn scroll_markdown_help_rendered_up(&mut self) {
        if self.ui.markdown_help_rendered_scroll > 0 {
            self.ui.markdown_help_rendered_scroll -= 1;
        }
    }

    /// Scroll markdown help rendered panel down by one line
    pub fn scroll_markdown_help_rendered_down(&mut self) {
        self.ui.markdown_help_rendered_scroll += 1;
    }

    /// Scroll markdown help example panel up by one page
    pub fn scroll_markdown_help_example_page_up(&mut self, viewport_height: usize) {
        if self.ui.markdown_help_example_scroll >= viewport_height {
            self.ui.markdown_help_example_scroll -= viewport_height;
        } else {
            self.ui.markdown_help_example_scroll = 0;
        }
    }

    /// Scroll markdown help example panel down by one page
    pub fn scroll_markdown_help_example_page_down(&mut self, viewport_height: usize, total_lines: usize) {
        let max_scroll = total_lines.saturating_sub(viewport_height);
        if self.ui.markdown_help_example_scroll + viewport_height <= max_scroll {
            self.ui.markdown_help_example_scroll += viewport_height;
        } else {
            self.ui.markdown_help_example_scroll = max_scroll;
        }
    }

    /// Scroll markdown help rendered panel up by one page
    pub fn scroll_markdown_help_rendered_page_up(&mut self, viewport_height: usize) {
        if self.ui.markdown_help_rendered_scroll >= viewport_height {
            self.ui.markdown_help_rendered_scroll -= viewport_height;
        } else {
            self.ui.markdown_help_rendered_scroll = 0;
        }
    }

    /// Scroll markdown help rendered panel down by one page
    pub fn scroll_markdown_help_rendered_page_down(&mut self, viewport_height: usize, total_lines: usize) {
        let max_scroll = total_lines.saturating_sub(viewport_height);
        if self.ui.markdown_help_rendered_scroll + viewport_height <= max_scroll {
            self.ui.markdown_help_rendered_scroll += viewport_height;
        } else {
            self.ui.markdown_help_rendered_scroll = max_scroll;
        }
    }

    /// Get settings categories
    pub fn get_settings_categories(&self) -> Vec<String> {
        vec!["Theme Settings".to_string(), "Appearance Settings".to_string(), "Display Settings".to_string(), "System Settings".to_string()]
    }
    
    /// Get config file path
    pub fn get_config_file_path(&self) -> String {
        // Determine profile based on database path
        let db_path = self.config.get_database_path();
        let db_path_str = db_path.to_string_lossy();
        let profile = if db_path_str.contains("tnj-dev") {
            crate::Profile::Dev
        } else {
            crate::Profile::Prod
        };
        
        match Config::get_config_path(profile) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => "Unknown".to_string(),
        }
    }
    
    /// Get database file path
    pub fn get_database_file_path(&self) -> String {
        self.config.get_database_path().to_string_lossy().to_string()
    }

    /// Get available themes
    pub fn get_available_themes(&self) -> Vec<String> {
        self.config.get_available_themes()
    }

    /// Move settings theme selection up
    pub fn move_settings_theme_selection_up(&mut self) {
        if self.settings.theme_index > 0 {
            self.settings.theme_index -= 1;
            self.settings.theme_list_state.select(Some(self.settings.theme_index));
        }
    }

    /// Move settings theme selection down
    pub fn move_settings_theme_selection_down(&mut self) {
        let themes = self.get_available_themes();
        if self.settings.theme_index < themes.len().saturating_sub(1) {
            self.settings.theme_index += 1;
            self.settings.theme_list_state.select(Some(self.settings.theme_index));
        }
    }

    /// Select and apply a theme
    pub fn select_theme(&mut self, theme_name: &str) -> Result<(), crate::config::ConfigError> {
        self.config.set_theme(theme_name)?;
        self.config.save()?;
        
        // Update theme index to match current theme
        let themes = self.get_available_themes();
        if let Some(index) = themes.iter().position(|t| t == theme_name) {
            self.settings.theme_index = index;
            self.settings.theme_list_state.select(Some(index));
        }
        
        self.set_status_message(format!("Theme changed to: {}", theme_name));
        Ok(())
    }

    /// Toggle task status between "todo" and "done"
    /// Only works when on Tasks tab with a task selected
    pub fn toggle_task_status(&mut self) -> Result<(), DatabaseError> {
        // Only work on Tasks tab
        if self.ui.current_tab != Tab::Tasks {
            return Ok(());
        }

        // Check if a task is selected
        if let Some(SelectedItem::Task(task)) = &self.ui.selected_item {
            if let Some(task_id) = task.id {
                // Find the task in the list
                if let Some(ref mut task) = self.tasks.iter_mut().find(|t| t.id == Some(task_id)) {
                    // Get current status before toggling
                    let was_done = task.status == "done";
                    
                    // Toggle status
                    task.status = if was_done {
                        "todo".to_string()
                    } else {
                        "done".to_string()
                    };
                    task.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    
                    // Update in database
                    self.database.update_task(task)?;
                    
                    // Reload data
                    self.load_data()?;
                    
                    // Refresh selected item
                    if let Some(updated_task) = self.tasks.iter().find(|t| t.id == Some(task_id)) {
                        self.ui.selected_item = Some(SelectedItem::Task(updated_task.clone()));
                    }
                    
                    let status_msg = if !was_done {
                        "Task marked as done"
                    } else {
                        "Task marked as todo"
                    };
                    self.set_status_message(status_msg.to_string());
                }
            }
        }
        
        Ok(())
    }

    /// Reorder task up (swap with task above)
    /// Only works when on Tasks tab with a task selected
    pub fn reorder_task_up(&mut self) -> Result<(), DatabaseError> {
        // Only work on Tasks tab
        if self.ui.current_tab != Tab::Tasks {
            return Ok(());
        }

        // Check if we have a valid selection
        if self.ui.selected_index == 0 {
            // Already at the top
            return Ok(());
        }

        let items = self.get_current_items();
        if items.is_empty() || self.ui.selected_index >= items.len() {
            return Ok(());
        }

        // Get the selected task
        if let Some(SelectedItem::Task(selected_task)) = &self.ui.selected_item {
            if let Some(selected_task_id) = selected_task.id {
                // Find the task above
                let above_index = self.ui.selected_index - 1;
                if let Some(Item::Task(above_task)) = items.get(above_index) {
                    if let Some(above_task_id) = above_task.id {
                        // Get the actual tasks from the list
                        if let (Some(selected_task), Some(above_task)) = (
                            self.tasks.iter().find(|t| t.id == Some(selected_task_id)),
                            self.tasks.iter().find(|t| t.id == Some(above_task_id)),
                        ) {
                            // Swap order values
                            let selected_order = selected_task.order;
                            let above_order = above_task.order;

                            // Update both tasks in database
                            self.database.update_task_order(selected_task_id, above_order)?;
                            self.database.update_task_order(above_task_id, selected_order)?;

                            // Reload data
                            self.load_data()?;

                            // Find the new index of the selected task (it moved up one position)
                            if let Some(new_index) = self.tasks.iter().position(|t| t.id == Some(selected_task_id)) {
                                self.ui.selected_index = new_index;
                                self.sync_list_state();
                                
                                // Refresh selected item
                                if let Some(updated_task) = self.tasks.get(new_index) {
                                    self.ui.selected_item = Some(SelectedItem::Task(updated_task.clone()));
                                }
                            }

                            self.set_status_message("Task moved up".to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Reorder task down (swap with task below)
    /// Only works when on Tasks tab with a task selected
    pub fn reorder_task_down(&mut self) -> Result<(), DatabaseError> {
        // Only work on Tasks tab
        if self.ui.current_tab != Tab::Tasks {
            return Ok(());
        }

        let items = self.get_current_items();
        if items.is_empty() {
            return Ok(());
        }

        // Check if we have a valid selection
        if self.ui.selected_index >= items.len().saturating_sub(1) {
            // Already at the bottom
            return Ok(());
        }

        // Get the selected task
        if let Some(SelectedItem::Task(selected_task)) = &self.ui.selected_item {
            if let Some(selected_task_id) = selected_task.id {
                // Find the task below
                let below_index = self.ui.selected_index + 1;
                if let Some(Item::Task(below_task)) = items.get(below_index) {
                    if let Some(below_task_id) = below_task.id {
                        // Get the actual tasks from the list
                        if let (Some(selected_task), Some(below_task)) = (
                            self.tasks.iter().find(|t| t.id == Some(selected_task_id)),
                            self.tasks.iter().find(|t| t.id == Some(below_task_id)),
                        ) {
                            // Swap order values
                            let selected_order = selected_task.order;
                            let below_order = below_task.order;

                            // Update both tasks in database
                            self.database.update_task_order(selected_task_id, below_order)?;
                            self.database.update_task_order(below_task_id, selected_order)?;

                            // Reload data
                            self.load_data()?;

                            // Find the new index of the selected task (it moved down one position)
                            if let Some(new_index) = self.tasks.iter().position(|t| t.id == Some(selected_task_id)) {
                                self.ui.selected_index = new_index;
                                self.sync_list_state();
                                
                                // Refresh selected item
                                if let Some(updated_task) = self.tasks.get(new_index) {
                                    self.ui.selected_item = Some(SelectedItem::Task(updated_task.clone()));
                                }
                            }

                            self.set_status_message("Task moved down".to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Initialize settings state when entering Settings mode
    pub fn init_settings_state(&mut self) {
        // Set theme index to current theme
        let themes = self.get_available_themes();
        if let Some(index) = themes.iter().position(|t| t == &self.config.current_theme) {
            self.settings.theme_index = index;
            self.settings.theme_list_state.select(Some(index));
        } else {
            self.settings.theme_index = 0;
            self.settings.theme_list_state.select(Some(0));
        }
        
        // Initialize category list state
        self.settings.category_index = 0;
        self.settings.list_state.select(Some(0));
        
        // Initialize sidebar width index to current value
        let width_options = self.get_sidebar_width_options();
        if let Some(index) = width_options.iter().position(|&w| w == self.config.sidebar_width_percent) {
            self.settings.sidebar_width_index = index;
        } else {
            // Find closest value
            let current = self.config.sidebar_width_percent;
            if let Some(index) = width_options.iter().position(|&w| w >= current) {
                self.settings.sidebar_width_index = index;
            } else {
                self.settings.sidebar_width_index = width_options.len().saturating_sub(1);
            }
        }
        
        // Initialize display mode index to current value
        let mode_options = self.get_display_mode_options();
        let current_mode_str = match self.ui.list_view_mode {
            ListViewMode::Simple => "Simple",
            ListViewMode::TwoLine => "TwoLine",
            ListViewMode::GroupedByTags => "GroupedByTags",
        };
        if let Some(index) = mode_options.iter().position(|&m| m == current_mode_str) {
            self.settings.display_mode_index = index;
        } else {
            self.settings.display_mode_index = 0;
        }
    }

    /// Get display name for a notebook (returns "[None]" if None)
    pub fn get_notebook_display_name(&self, id: Option<i64>) -> String {
        if let Some(notebook_id) = id {
            self.notebooks.notebooks
                .iter()
                .find(|n| n.id == Some(notebook_id))
                .map(|n| n.name.clone())
                .unwrap_or_else(|| "[None]".to_string())
        } else {
            "[None]".to_string()
        }
    }

    /// Get notebook list with "[None]" first
    pub fn get_notebook_list_with_none(&self) -> Vec<(Option<i64>, String)> {
        let mut list = vec![(None, "[None]".to_string())];
        for notebook in &self.notebooks.notebooks {
            if let Some(id) = notebook.id {
                list.push((Some(id), notebook.name.clone()));
            }
        }
        list
    }

    /// Enter notebook modal mode
    pub fn enter_notebook_modal_mode(&mut self) {
        // Find the index of the current notebook in the list (0 = "[None]", 1+ = actual notebooks)
        let selected_index = if let Some(current_id) = self.notebooks.current_notebook_id {
            self.notebooks.notebooks
                .iter()
                .position(|n| n.id == Some(current_id))
                .map(|idx| idx + 1) // +1 because "[None]" is at index 0
                .unwrap_or(0)
        } else {
            0 // "[None]" is selected
        };

        self.notebooks.modal_state = Some(NotebookModalState {
            mode: NotebookModalMode::View,
            selected_index,
            actions_selected_index: 0,
            name_editor: Editor::new(),
            list_state: ListState::default(),
            current_field: NotebookModalField::NotebookList,
        });
        self.notebooks.modal_state.as_mut().unwrap().list_state.select(Some(selected_index));
        self.ui.mode = Mode::NotebookModal;
    }

    /// Exit notebook modal mode
    pub fn exit_notebook_modal_mode(&mut self) {
        self.ui.mode = Mode::View;
        self.notebooks.modal_state = None;
    }

    /// Switch to a different notebook
    pub fn switch_notebook(&mut self, id: Option<i64>) -> Result<(), DatabaseError> {
        self.notebooks.current_notebook_id = id;
        // Reload data to filter by new notebook
        self.load_data()?;
        self.set_status_message(format!("Switched to notebook: {}", self.get_notebook_display_name(id)));
        Ok(())
    }

    /// Add a new notebook
    pub fn add_notebook(&mut self, name: String) -> Result<(), DatabaseError> {
        if name.trim().is_empty() {
            self.set_status_message("Notebook name cannot be empty".to_string());
            return Ok(());
        }

        // Check for duplicate names
        if self.notebooks.notebooks.iter().any(|n| n.name == name.trim()) {
            self.set_status_message("A notebook with this name already exists".to_string());
            return Ok(());
        }

        let mut notebook = Notebook::new(name.trim().to_string());
        let notebook_id = self.database.insert_notebook(&notebook)?;
        notebook.id = Some(notebook_id);
        self.notebooks.notebooks.push(notebook);
        self.notebooks.notebooks.sort_by(|a, b| a.name.cmp(&b.name));
        
        // Reload to ensure consistency
        self.notebooks.notebooks = self.database.get_all_notebooks()?;
        
        self.set_status_message("Notebook created".to_string());
        Ok(())
    }

    /// Rename a notebook
    pub fn rename_notebook(&mut self, id: i64, new_name: String) -> Result<(), DatabaseError> {
        if new_name.trim().is_empty() {
            self.set_status_message("Notebook name cannot be empty".to_string());
            return Ok(());
        }

        // Check for duplicate names (excluding the current notebook)
        if self.notebooks.notebooks.iter().any(|n| n.id != Some(id) && n.name == new_name.trim()) {
            self.set_status_message("A notebook with this name already exists".to_string());
            return Ok(());
        }

        if let Some(notebook) = self.notebooks.notebooks.iter_mut().find(|n| n.id == Some(id)) {
            notebook.name = new_name.trim().to_string();
            notebook.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            self.database.update_notebook(notebook)?;
            
            // Reload to ensure consistency
            self.notebooks.notebooks = self.database.get_all_notebooks()?;
            
            self.set_status_message("Notebook renamed".to_string());
        } else {
            self.set_status_message("Notebook not found".to_string());
        }
        Ok(())
    }

    /// Delete a notebook
    /// Items that belonged to this notebook will be moved to "[None]"
    pub fn delete_notebook(&mut self, id: i64) -> Result<(), DatabaseError> {
        // Check if this is the current notebook
        if self.notebooks.current_notebook_id == Some(id) {
            // Switch to "[None]" before deleting
            self.notebooks.current_notebook_id = None;
        }

        self.database.delete_notebook(id)?;
        
        // Remove from local list
        self.notebooks.notebooks.retain(|n| n.id != Some(id));
        
        // Reload to ensure consistency
        self.notebooks.notebooks = self.database.get_all_notebooks()?;
        
        // Reload data to show items that were moved to "[None]"
        self.load_data()?;
        
        self.set_status_message("Notebook deleted (items moved to [None])".to_string());
        Ok(())
    }

    /// Navigate notebook modal fields
    pub fn navigate_notebook_modal(&mut self) {
        if let Some(ref mut state) = self.notebooks.modal_state {
            // Tab only switches between NotebookList and ActionsList
            state.current_field = match state.current_field {
                NotebookModalField::NotebookList => NotebookModalField::ActionsList,
                NotebookModalField::ActionsList => NotebookModalField::NotebookList,
            };
        }
    }

    /// Move notebook selection up
    pub fn move_notebook_selection_up(&mut self) {
        if let Some(ref mut state) = self.notebooks.modal_state {
            if state.selected_index > 0 {
                state.selected_index -= 1;
                state.list_state.select(Some(state.selected_index));
            }
        }
    }

    /// Move notebook selection down
    pub fn move_notebook_selection_down(&mut self) {
        if let Some(ref mut state) = self.notebooks.modal_state {
            let max_index = self.notebooks.notebooks.len(); // "[None]" + notebooks
            // Allow incrementing up to and including max_index (the last notebook)
            // Since index 0 is "[None]" and indices 1+ are notebooks, max_index is the last valid index
            if state.selected_index <= max_index {
                state.selected_index = (state.selected_index + 1).min(max_index);
                state.list_state.select(Some(state.selected_index));
            }
        }
    }

    /// Move actions selection up
    pub fn move_actions_selection_up(&mut self) {
        if let Some(ref mut state) = self.notebooks.modal_state {
            if state.actions_selected_index > 0 {
                state.actions_selected_index -= 1;
            }
        }
    }

    /// Move actions selection down
    pub fn move_actions_selection_down(&mut self) {
        if let Some(ref mut state) = self.notebooks.modal_state {
            // Actions: Add (0), Rename (1), Delete (2), Switch (3)
            let max_index = 3;
            if state.actions_selected_index < max_index {
                state.actions_selected_index += 1;
            }
        }
    }

    /// Get current notebook modal editor (for add/rename)
    pub fn get_notebook_modal_editor(&mut self) -> Option<&mut Editor> {
        if let Some(ref mut state) = self.notebooks.modal_state {
            if matches!(state.mode, NotebookModalMode::Add | NotebookModalMode::Rename) {
                Some(&mut state.name_editor)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Task(Task),
    Note(Note),
    Journal(JournalEntry),
}

impl Item {
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        match self {
            Item::Task(task) => {
                task.title.to_lowercase().contains(&query_lower) ||
                task.description.as_ref().map(|d| d.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                task.tags.as_ref().map(|t| t.to_lowercase().contains(&query_lower)).unwrap_or(false)
            }
            Item::Note(note) => {
                note.title.to_lowercase().contains(&query_lower) ||
                note.content.as_ref().map(|c| c.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                note.tags.as_ref().map(|t| t.to_lowercase().contains(&query_lower)).unwrap_or(false)
            }
            Item::Journal(journal) => {
                journal.date.to_lowercase().contains(&query_lower) ||
                journal.title.as_ref().map(|t| t.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                journal.content.as_ref().map(|c| c.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                journal.tags.as_ref().map(|t| t.to_lowercase().contains(&query_lower)).unwrap_or(false)
            }
        }
    }

    pub fn matches_tag_filter(&self, filter_tags: &str, logic: FilterTagLogic) -> bool {
        use crate::tui::widgets::tags::parse_tags;
        
        let filter_tag_list: Vec<String> = filter_tags
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        
        if filter_tag_list.is_empty() {
            return true; // No filter tags means match all
        }
        
        let item_tags = match self {
            Item::Task(task) => parse_tags(task.tags.as_ref()),
            Item::Note(note) => parse_tags(note.tags.as_ref()),
            Item::Journal(journal) => parse_tags(journal.tags.as_ref()),
        };
        
        let is_untagged = item_tags.is_empty();
        
        // Check if "[Untagged]" is in the filter (case-insensitive)
        let has_untagged_filter = filter_tag_list.iter().any(|tag| tag == "[untagged]");
        
        // Remove "[Untagged]" from filter list for normal tag matching
        let regular_filter_tags: Vec<String> = filter_tag_list.iter()
            .filter(|tag| tag != &"[untagged]")
            .cloned()
            .collect();
        
        let item_tags_lower: Vec<String> = item_tags.iter()
            .map(|t| t.to_lowercase())
            .collect();
        
        match logic {
            FilterTagLogic::And => {
                // For AND logic:
                // - If "[Untagged]" is the only filter: match only untagged items
                // - If "[Untagged]" is combined with other tags: impossible (can't be untagged and have tags), so no match
                // - If only regular tags: item must have ALL regular tags
                if has_untagged_filter {
                    if regular_filter_tags.is_empty() {
                        // Only "[Untagged]" filter: match untagged items
                        return is_untagged;
                    } else {
                        // "[Untagged]" + other tags with AND: impossible, no match
                        return false;
                    }
                } else {
                    // Only regular tags: item must have ALL filter tags
                    regular_filter_tags.iter().all(|filter_tag| {
                        item_tags_lower.contains(filter_tag)
                    })
                }
            }
            FilterTagLogic::Or => {
                // For OR logic:
                // - Item matches if it's untagged (when "[Untagged]" is in filter) OR
                // - Item matches if it has ANY of the regular filter tags
                let matches_untagged = has_untagged_filter && is_untagged;
                let matches_regular_tags = if regular_filter_tags.is_empty() {
                    false
                } else {
                    regular_filter_tags.iter().any(|filter_tag| {
                        item_tags_lower.contains(filter_tag)
                    })
                };
                matches_untagged || matches_regular_tags
            }
        }
    }
}

