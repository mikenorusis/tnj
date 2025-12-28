use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>, // ISO 8601: YYYY-MM-DD
    pub status: String,           // todo, done
    pub tags: Option<String>,
    pub order: i64,               // Order for sorting tasks
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Option<i64>,
    pub title: String,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Option<i64>,
    pub date: String, // YYYY-MM-DD
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Task {
    pub fn new(title: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: None,
            title,
            description: None,
            due_date: None,
            status: "todo".to_string(),
            tags: None,
            order: 0,
            archived: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl Note {
    pub fn new(title: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: None,
            title,
            content: None,
            tags: None,
            archived: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl JournalEntry {
    pub fn new(date: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: None,
            date,
            title: None,
            content: None,
            tags: None,
            archived: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Notebook {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: None,
            name,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

