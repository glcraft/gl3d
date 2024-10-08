#[derive(Debug)]
pub struct ApplicationError {
    titre: String,
    description: Option<String>
}

impl ApplicationError {
    pub fn new(titre: String) -> ApplicationError {
        ApplicationError {
            titre,
            description: None
        }
    }

    pub fn with_description(titre: String, description: String) -> ApplicationError {
        ApplicationError {
            titre,
            description: Some(description)
        }
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.titre)?;
        if let Some(description) = self.description.as_ref() {
            write!(f, ": {}", description)?
        }
        Ok(())
    }
}

impl std::error::Error for ApplicationError {}

impl From<ash::LoadingError> for ApplicationError {
    fn from(err: ash::LoadingError) -> ApplicationError {
        ApplicationError::with_description("Unable to load Vulkan".to_string(), err.to_string())
    }
}

impl From<ash::vk::Result> for ApplicationError {
    fn from(err: ash::vk::Result) -> ApplicationError {
        ApplicationError::with_description("Vulkan error".to_string(), err.to_string())
    }
}
impl From<&str> for ApplicationError {
    fn from(err: &str) -> ApplicationError {
        ApplicationError::new(err.to_string())
    }
}
impl From<String> for ApplicationError {
    fn from(err: String) -> ApplicationError {
        ApplicationError::new(err)
    }
}