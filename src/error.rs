use log::error;

/// Common error type for the maps crate.
#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    /// Logs the error message before passing the error on.
    pub fn and_log_it(self) -> Self {
        error!("{}", self.message);
        self
    }
}
