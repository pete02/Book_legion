use crate::core::types::{ErrorCode, Severity, ValidationError};

/// Result of a complete EPUB validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub passed: bool,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            passed: true,
        }
    }
    
    pub fn add_error(&mut self, error: ValidationError) {
        if error.severity == Severity::Error {
            self.errors.push(error);
        } else {
            self.warnings.push(error);
        }
        self.passed = false;
    }
    
    pub fn add_warning(&mut self, error: ValidationError) {
        self.warnings.push(error);
        self.passed = false;
    }
    
    pub fn add(&mut self, error: ValidationError) {
        match error.severity {
            Severity::Error => self.add_error(error),
            Severity::Warning => self.add_warning(error),
        }
    }
    
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
    
    pub fn extend(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.passed = self.passed && other.passed;
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing validation results
pub struct ValidationResultBuilder {
    result: ValidationResult,
}

impl ValidationResultBuilder {
    pub fn new() -> Self {
        Self {
            result: ValidationResult::new(),
        }
    }
    
    pub fn with_error(mut self, code: ErrorCode, location: crate::core::types::ValidationLocation) -> Self {
        self.result.add_error(ValidationError::new(code, location));
        self
    }
    
    pub fn with_warning(mut self, code: ErrorCode, location: crate::core::types::ValidationLocation) -> Self {
        self.result.add_warning(ValidationError::warning(code, location));
        self
    }
    
    pub fn with_errors(mut self, errors: Vec<ValidationError>) -> Self {
        for error in errors {
            self.result.add(error);
        }
        self
    }
    
    pub fn build(self) -> ValidationResult {
        self.result
    }
}

impl Default for ValidationResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}
