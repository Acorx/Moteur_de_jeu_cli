use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub exit_code: i32,
    pub json: Option<String>,
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
            json: None,
        }
    }

    pub fn outcome(message: impl Into<String>, exit_code: i32, json: String) -> Self {
        Self {
            message: message.into(),
            exit_code,
            json: Some(json),
        }
    }

    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = exit_code;
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
