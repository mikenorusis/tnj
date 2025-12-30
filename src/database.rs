use rusqlite::Connection;
use std::path::PathBuf;
use thiserror::Error;

use crate::models::{Task, Note, JournalEntry, Notebook};

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("Failed to create database directory: {0}")]
    DirectoryError(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize the schema
    pub fn new(path: &str) -> Result<Self, DatabaseError> {
        let db_path = PathBuf::from(path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DatabaseError::DirectoryError(e.to_string()))?;
            }
        }

        // Open or create the database
        let conn = Connection::open(&db_path)?;

        let db = Database { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Initialize the database schema (tables and indexes)
    fn initialize_schema(&self) -> Result<(), DatabaseError> {
        // Create tasks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                title           TEXT NOT NULL,
                description     TEXT,
                due_date        TEXT,
                status          TEXT DEFAULT 'todo',
                tags            TEXT,
                \"order\"        INTEGER DEFAULT 0,
                archived        INTEGER DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            )",
            [],
        )?;

        // Create notes table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                title           TEXT NOT NULL,
                content         TEXT,
                tags            TEXT,
                archived        INTEGER DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            )",
            [],
        )?;

        // Create journals table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS journals (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                date            TEXT NOT NULL,
                title           TEXT,
                content         TEXT,
                tags            TEXT,
                archived        INTEGER DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            )",
            [],
        )?;

        // Create notebooks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notebooks (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_journals_date ON journals(date)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_title ON tasks(title)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_journals_title ON journals(title)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notebooks_name ON notebooks(name)",
            [],
        )?;

        // Migrate existing tables to add notebook_id column if it doesn't exist
        self.migrate_add_notebook_id()?;

        Ok(())
    }

    /// Migrate existing tables to add notebook_id column
    fn migrate_add_notebook_id(&self) -> Result<(), DatabaseError> {
        // Helper to check if a column exists
        fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, DatabaseError> {
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2"
            )?;
            let count: i64 = stmt.query_row(rusqlite::params![table, column], |row| row.get(0))?;
            Ok(count > 0)
        }

        // Add notebook_id to tasks table if it doesn't exist
        if !column_exists(&self.conn, "tasks", "notebook_id")? {
            self.conn.execute(
                "ALTER TABLE tasks ADD COLUMN notebook_id INTEGER",
                [],
            )?;
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_tasks_notebook_id ON tasks(notebook_id)",
                [],
            )?;
        }

        // Add notebook_id to notes table if it doesn't exist
        if !column_exists(&self.conn, "notes", "notebook_id")? {
            self.conn.execute(
                "ALTER TABLE notes ADD COLUMN notebook_id INTEGER",
                [],
            )?;
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_notes_notebook_id ON notes(notebook_id)",
                [],
            )?;
        }

        // Add notebook_id to journals table if it doesn't exist
        if !column_exists(&self.conn, "journals", "notebook_id")? {
            self.conn.execute(
                "ALTER TABLE journals ADD COLUMN notebook_id INTEGER",
                [],
            )?;
            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_journals_notebook_id ON journals(notebook_id)",
                [],
            )?;
        }

        Ok(())
    }

    /// Get a reference to the underlying connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying connection
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Insert a task into the database and return its ID
    pub fn insert_task(&self, task: &Task) -> Result<i64, DatabaseError> {
        self.conn.execute(
            "INSERT INTO tasks (title, description, due_date, status, tags, \"order\", archived, notebook_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                task.title,
                task.description,
                task.due_date,
                task.status,
                task.tags,
                task.order,
                if task.archived { 1 } else { 0 },
                task.notebook_id,
                task.created_at,
                task.updated_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert a note into the database and return its ID
    pub fn insert_note(&self, note: &Note) -> Result<i64, DatabaseError> {
        self.conn.execute(
            "INSERT INTO notes (title, content, tags, archived, notebook_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                note.title,
                note.content,
                note.tags,
                if note.archived { 1 } else { 0 },
                note.notebook_id,
                note.created_at,
                note.updated_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert a journal entry into the database and return its ID
    pub fn insert_journal(&self, journal: &JournalEntry) -> Result<i64, DatabaseError> {
        self.conn.execute(
            "INSERT INTO journals (date, title, content, tags, archived, notebook_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                journal.date,
                journal.title,
                journal.content,
                journal.tags,
                if journal.archived { 1 } else { 0 },
                journal.notebook_id,
                journal.created_at,
                journal.updated_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Helper function to map a row to a Task
    fn row_to_task(row: &rusqlite::Row) -> Result<Task, rusqlite::Error> {
        Ok(Task {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            description: row.get(2)?,
            due_date: row.get(3)?,
            status: row.get(4)?,
            tags: row.get(5)?,
            order: row.get(6)?,
            archived: row.get::<_, i64>(7)? != 0,
            notebook_id: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }

    /// Get all tasks ordered by order ASC, optionally filtered by notebook_id
    pub fn get_all_tasks(&self, notebook_id: Option<i64>) -> Result<Vec<Task>, DatabaseError> {
        if let Some(nb_id) = notebook_id {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, description, due_date, status, tags, \"order\", archived, notebook_id, created_at, updated_at
                 FROM tasks WHERE archived = 0 AND notebook_id = ?1 ORDER BY \"order\" ASC"
            )?;
            let tasks = stmt.query_map(rusqlite::params![nb_id], Self::row_to_task)?
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(tasks);
        }
        
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, due_date, status, tags, \"order\", archived, notebook_id, created_at, updated_at
             FROM tasks WHERE archived = 0 AND notebook_id IS NULL ORDER BY \"order\" ASC"
        )?;
        let tasks = stmt.query_map([], Self::row_to_task)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(tasks)
    }

    /// Get all tasks including archived, ordered by order ASC, optionally filtered by notebook_id
    pub fn get_all_tasks_including_archived(&self, notebook_id: Option<i64>) -> Result<Vec<Task>, DatabaseError> {
        if let Some(nb_id) = notebook_id {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, description, due_date, status, tags, \"order\", archived, notebook_id, created_at, updated_at
                 FROM tasks WHERE notebook_id = ?1 ORDER BY \"order\" ASC"
            )?;
            let tasks = stmt.query_map(rusqlite::params![nb_id], Self::row_to_task)?
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(tasks);
        }
        
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, due_date, status, tags, \"order\", archived, notebook_id, created_at, updated_at
             FROM tasks WHERE notebook_id IS NULL ORDER BY \"order\" ASC"
        )?;
        let tasks = stmt.query_map([], Self::row_to_task)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(tasks)
    }

    /// Get a single task by ID
    pub fn get_task(&self, id: i64) -> Result<Task, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, due_date, status, tags, \"order\", archived, notebook_id, created_at, updated_at
             FROM tasks WHERE id = ?1"
        )?;
        
        stmt.query_row(rusqlite::params![id], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                due_date: row.get(3)?,
                status: row.get(4)?,
                tags: row.get(5)?,
                order: row.get(6)?,
                archived: row.get::<_, i64>(7)? != 0,
                notebook_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(DatabaseError::from)
    }

    /// Update an existing task
    pub fn update_task(&self, task: &Task) -> Result<(), DatabaseError> {
        let id = task.id.ok_or_else(|| DatabaseError::SqliteError(
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Null)
        ))?;
        
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE tasks SET title = ?1, description = ?2, due_date = ?3, 
             status = ?4, tags = ?5, \"order\" = ?6, archived = ?7, notebook_id = ?8, updated_at = ?9 WHERE id = ?10",
            rusqlite::params![
                task.title,
                task.description,
                task.due_date,
                task.status,
                task.tags,
                task.order,
                if task.archived { 1 } else { 0 },
                task.notebook_id,
                task.updated_at,
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Get the maximum order value from all tasks
    pub fn get_max_task_order(&self) -> Result<i64, DatabaseError> {
        let max_order: Option<i64> = self.conn.query_row(
            "SELECT MAX(\"order\") FROM tasks",
            [],
            |row| row.get(0),
        )?;
        Ok(max_order.unwrap_or(-1))
    }

    /// Update a task's order value
    pub fn update_task_order(&self, task_id: i64, new_order: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE tasks SET \"order\" = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![
                new_order,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                task_id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Delete a task by ID
    pub fn delete_task(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])?;
        tx.commit()?;
        Ok(())
    }

    /// Archive a task by ID
    pub fn archive_task(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE tasks SET archived = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Helper function to map a row to a Note
    fn row_to_note(row: &rusqlite::Row) -> Result<Note, rusqlite::Error> {
        Ok(Note {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            content: row.get(2)?,
            tags: row.get(3)?,
            archived: row.get::<_, i64>(4)? != 0,
            notebook_id: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    /// Get all notes ordered by created_at DESC, optionally filtered by notebook_id
    pub fn get_all_notes(&self, notebook_id: Option<i64>) -> Result<Vec<Note>, DatabaseError> {
        if let Some(nb_id) = notebook_id {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, content, tags, archived, notebook_id, created_at, updated_at
                 FROM notes WHERE archived = 0 AND notebook_id = ?1 ORDER BY created_at DESC"
            )?;
            let notes = stmt.query_map(rusqlite::params![nb_id], Self::row_to_note)?
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(notes);
        }
        
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, archived, notebook_id, created_at, updated_at
             FROM notes WHERE archived = 0 AND notebook_id IS NULL ORDER BY created_at DESC"
        )?;
        let notes = stmt.query_map([], Self::row_to_note)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(notes)
    }

    /// Get all notes including archived, ordered by created_at DESC, optionally filtered by notebook_id
    pub fn get_all_notes_including_archived(&self, notebook_id: Option<i64>) -> Result<Vec<Note>, DatabaseError> {
        if let Some(nb_id) = notebook_id {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, content, tags, archived, notebook_id, created_at, updated_at
                 FROM notes WHERE notebook_id = ?1 ORDER BY created_at DESC"
            )?;
            let notes = stmt.query_map(rusqlite::params![nb_id], Self::row_to_note)?
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(notes);
        }
        
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, archived, notebook_id, created_at, updated_at
             FROM notes WHERE notebook_id IS NULL ORDER BY created_at DESC"
        )?;
        let notes = stmt.query_map([], Self::row_to_note)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(notes)
    }

    /// Get a single note by ID
    pub fn get_note(&self, id: i64) -> Result<Note, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, archived, notebook_id, created_at, updated_at
             FROM notes WHERE id = ?1"
        )?;
        
        stmt.query_row(rusqlite::params![id], |row| {
            Ok(Note {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                tags: row.get(3)?,
                archived: row.get::<_, i64>(4)? != 0,
                notebook_id: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(DatabaseError::from)
    }

    /// Update an existing note
    pub fn update_note(&self, note: &Note) -> Result<(), DatabaseError> {
        let id = note.id.ok_or_else(|| DatabaseError::SqliteError(
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Null)
        ))?;
        
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE notes SET title = ?1, content = ?2, tags = ?3, archived = ?4, notebook_id = ?5, updated_at = ?6 WHERE id = ?7",
            rusqlite::params![
                note.title,
                note.content,
                note.tags,
                if note.archived { 1 } else { 0 },
                note.notebook_id,
                note.updated_at,
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Delete a note by ID
    pub fn delete_note(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM notes WHERE id = ?1", rusqlite::params![id])?;
        tx.commit()?;
        Ok(())
    }

    /// Archive a note by ID
    pub fn archive_note(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE notes SET archived = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Helper function to map a row to a JournalEntry
    fn row_to_journal(row: &rusqlite::Row) -> Result<JournalEntry, rusqlite::Error> {
        Ok(JournalEntry {
            id: Some(row.get(0)?),
            date: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            tags: row.get(4)?,
            archived: row.get::<_, i64>(5)? != 0,
            notebook_id: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }

    /// Get all journal entries ordered by date DESC (newest first), optionally filtered by notebook_id
    pub fn get_all_journals(&self, notebook_id: Option<i64>) -> Result<Vec<JournalEntry>, DatabaseError> {
        if let Some(nb_id) = notebook_id {
            let mut stmt = self.conn.prepare(
                "SELECT id, date, title, content, tags, archived, notebook_id, created_at, updated_at
                 FROM journals WHERE archived = 0 AND notebook_id = ?1 ORDER BY date DESC, created_at DESC"
            )?;
            let journals = stmt.query_map(rusqlite::params![nb_id], Self::row_to_journal)?
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(journals);
        }
        
        let mut stmt = self.conn.prepare(
            "SELECT id, date, title, content, tags, archived, notebook_id, created_at, updated_at
             FROM journals WHERE archived = 0 AND notebook_id IS NULL ORDER BY date DESC, created_at DESC"
        )?;
        let journals = stmt.query_map([], Self::row_to_journal)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(journals)
    }

    /// Get all journal entries including archived, ordered by date DESC, created_at DESC, optionally filtered by notebook_id
    pub fn get_all_journals_including_archived(&self, notebook_id: Option<i64>) -> Result<Vec<JournalEntry>, DatabaseError> {
        if let Some(nb_id) = notebook_id {
            let mut stmt = self.conn.prepare(
                "SELECT id, date, title, content, tags, archived, notebook_id, created_at, updated_at
                 FROM journals WHERE notebook_id = ?1 ORDER BY date DESC, created_at DESC"
            )?;
            let journals = stmt.query_map(rusqlite::params![nb_id], Self::row_to_journal)?
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(journals);
        }
        
        let mut stmt = self.conn.prepare(
            "SELECT id, date, title, content, tags, archived, notebook_id, created_at, updated_at
             FROM journals WHERE notebook_id IS NULL ORDER BY date DESC, created_at DESC"
        )?;
        let journals = stmt.query_map([], Self::row_to_journal)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(journals)
    }

    /// Get a single journal entry by ID
    pub fn get_journal(&self, id: i64) -> Result<JournalEntry, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, title, content, tags, archived, notebook_id, created_at, updated_at
             FROM journals WHERE id = ?1"
        )?;
        
        stmt.query_row(rusqlite::params![id], |row| {
            Ok(JournalEntry {
                id: Some(row.get(0)?),
                date: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                tags: row.get(4)?,
                archived: row.get::<_, i64>(5)? != 0,
                notebook_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(DatabaseError::from)
    }

    /// Update an existing journal entry
    pub fn update_journal(&self, journal: &JournalEntry) -> Result<(), DatabaseError> {
        let id = journal.id.ok_or_else(|| DatabaseError::SqliteError(
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Null)
        ))?;
        
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE journals SET date = ?1, title = ?2, content = ?3, tags = ?4, archived = ?5, notebook_id = ?6, updated_at = ?7 WHERE id = ?8",
            rusqlite::params![
                journal.date,
                journal.title,
                journal.content,
                journal.tags,
                if journal.archived { 1 } else { 0 },
                journal.notebook_id,
                journal.updated_at,
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Delete a journal entry by ID
    pub fn delete_journal(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM journals WHERE id = ?1", rusqlite::params![id])?;
        tx.commit()?;
        Ok(())
    }

    /// Archive a journal entry by ID
    pub fn archive_journal(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE journals SET archived = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Get all notebooks ordered by name ASC
    pub fn get_all_notebooks(&self) -> Result<Vec<Notebook>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at
             FROM notebooks ORDER BY name ASC"
        )?;
        
        let notebooks = stmt.query_map([], |row| {
            Ok(Notebook {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(notebooks)
    }

    /// Get a single notebook by ID
    pub fn get_notebook(&self, id: i64) -> Result<Notebook, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at
             FROM notebooks WHERE id = ?1"
        )?;
        
        stmt.query_row(rusqlite::params![id], |row| {
            Ok(Notebook {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(DatabaseError::from)
    }

    /// Insert a notebook into the database and return its ID
    pub fn insert_notebook(&self, notebook: &Notebook) -> Result<i64, DatabaseError> {
        self.conn.execute(
            "INSERT INTO notebooks (name, created_at, updated_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![
                notebook.name,
                notebook.created_at,
                notebook.updated_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Update an existing notebook
    pub fn update_notebook(&self, notebook: &Notebook) -> Result<(), DatabaseError> {
        let id = notebook.id.ok_or_else(|| DatabaseError::SqliteError(
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Null)
        ))?;
        
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE notebooks SET name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![
                notebook.name,
                notebook.updated_at,
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Delete a notebook by ID
    /// Sets notebook_id to NULL for all items (tasks, notes, journals) that belonged to this notebook
    pub fn delete_notebook(&self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        
        // Set notebook_id to NULL for all tasks that belonged to this notebook
        tx.execute(
            "UPDATE tasks SET notebook_id = NULL, updated_at = ?1 WHERE notebook_id = ?2",
            rusqlite::params![
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                id
            ],
        )?;
        
        // Set notebook_id to NULL for all notes that belonged to this notebook
        tx.execute(
            "UPDATE notes SET notebook_id = NULL, updated_at = ?1 WHERE notebook_id = ?2",
            rusqlite::params![
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                id
            ],
        )?;
        
        // Set notebook_id to NULL for all journals that belonged to this notebook
        tx.execute(
            "UPDATE journals SET notebook_id = NULL, updated_at = ?1 WHERE notebook_id = ?2",
            rusqlite::params![
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                id
            ],
        )?;
        
        // Delete the notebook
        tx.execute("DELETE FROM notebooks WHERE id = ?1", rusqlite::params![id])?;
        
        tx.commit()?;
        Ok(())
    }

    /// Get the first notebook (for default)
    pub fn get_default_notebook(&self) -> Result<Option<Notebook>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at
             FROM notebooks ORDER BY name ASC LIMIT 1"
        )?;
        
        let result = stmt.query_row([], |row| {
            Ok(Notebook {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        });
        
        match result {
            Ok(notebook) => Ok(Some(notebook)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::from(e)),
        }
    }
}

