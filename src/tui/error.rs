use thiserror::Error;
use crate::database::DatabaseError;

#[derive(Debug, Error)]
pub enum TuiError {
    #[error("IO/Terminal error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    
    #[error("Key binding error: {0}")]
    KeyBindingError(String),
    
    #[error("Render error: {0}")]
    RenderError(String),
}

