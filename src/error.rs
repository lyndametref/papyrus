use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Runtime(String),
    NotFound(String),
    Validation(String),
}

impl AppError {
    pub fn code(&self) -> i32 {
        match self {
            AppError::Runtime(_) => 1,
            AppError::NotFound(_) => 3,
            AppError::Validation(_) => 4,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            AppError::Runtime(m) => m,
            AppError::NotFound(m) => m,
            AppError::Validation(m) => m,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for AppError {}
