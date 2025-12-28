use clap::{Parser, Subcommand};
use thiserror::Error;

use crate::database::Database;
use crate::database::DatabaseError;
use crate::models::{Task, Note, JournalEntry};
use crate::utils::{parse_date, get_current_date_string};

#[derive(Parser)]
#[command(name = "tnj")]
#[command(about = "Tasks, Notes, Journal - A lightweight terminal application")]
#[command(version)]
pub struct Cli {
    /// Custom config file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Use development mode (uses separate dev config/database)
    #[arg(long)]
    pub dev: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive TUI (default if no subcommand)
    Tui,
    /// Quickly add a new task
    AddTask {
        /// Task title
        title: String,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
    },
    /// Quickly add a new note
    AddNote {
        /// Note title
        title: String,
        /// Note content
        #[arg(long)]
        content: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
    },
    /// Quickly add a new journal entry
    AddJournal {
        /// Journal content
        content: String,
        /// Journal title
        #[arg(long)]
        title: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
    },
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Failed to parse date: {0}")]
    DateParseError(String),
}

/// Handle the add-task command
pub fn handle_add_task(
    title: String,
    due: Option<String>,
    tags: Option<String>,
    db: &Database,
) -> Result<(), CliError> {
    // Parse due date if provided
    let due_date = if let Some(due_str) = due {
        parse_date(&due_str)
            .map_err(|e| CliError::DateParseError(format!("Invalid date format '{}': {}", due_str, e)))?;
        Some(due_str)
    } else {
        None
    };

    // Create task
    let mut task = Task::new(title);
    task.due_date = due_date;
    task.tags = tags;
    
    // Assign order value (max + 1)
    let max_order = db.get_max_task_order().unwrap_or(-1);
    task.order = max_order + 1;

    // Insert into database
    let id = db.insert_task(&task)?;
    println!("Task created successfully (ID: {})", id);

    Ok(())
}

/// Handle the add-note command
pub fn handle_add_note(
    title: String,
    content: Option<String>,
    tags: Option<String>,
    db: &Database,
) -> Result<(), CliError> {
    // Create note
    let mut note = Note::new(title);
    note.content = content;
    note.tags = tags;

    // Insert into database
    let id = db.insert_note(&note)?;
    println!("Note created successfully (ID: {})", id);

    Ok(())
}

/// Handle the add-journal command
pub fn handle_add_journal(
    content: String,
    title: Option<String>,
    tags: Option<String>,
    db: &Database,
) -> Result<(), CliError> {
    // Use current date for journal entry
    let date = get_current_date_string();

    // Create journal entry
    let mut journal = JournalEntry::new(date);
    journal.content = Some(content);
    journal.title = title;
    journal.tags = tags;

    // Insert into database
    let id = db.insert_journal(&journal)?;
    println!("Journal entry created successfully (ID: {})", id);

    Ok(())
}

