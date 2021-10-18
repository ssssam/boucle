use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<portmidi::Error> for AppError {
    fn from(error: portmidi::Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<cpal::DevicesError> for AppError {
    fn from(error: cpal::DevicesError) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<cpal::DeviceNameError> for AppError {
    fn from(error: cpal::DeviceNameError) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<hound::Error> for AppError {
    fn from(error: hound::Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}
