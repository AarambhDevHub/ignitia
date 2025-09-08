use crate::middleware::Middleware;
use crate::{Response, Result};
use http::StatusCode;
use std::collections::HashMap;
use tracing::{debug, error, warn};

pub struct ErrorHandlerMiddleware {
    /// Whether to include detailed error information in responses
    include_details: bool,
    /// Whether to include stack traces in debug mode
    include_stack_trace: bool,
    /// Custom error pages for different status codes
    custom_error_pages: HashMap<StatusCode, String>,
    /// Whether to log errors
    log_errors: bool,
    /// Minimum status code to log as error (vs warning)
    error_log_threshold: u16,
}

impl ErrorHandlerMiddleware {
    pub fn new() -> Self {
        Self {
            include_details: cfg!(debug_assertions),
            include_stack_trace: cfg!(debug_assertions),
            custom_error_pages: HashMap::new(),
            log_errors: true,
            error_log_threshold: 500, // 5xx errors logged as errors, 4xx as warnings
        }
    }

    /// Enable/disable detailed error information in responses
    pub fn with_details(mut self, include: bool) -> Self {
        self.include_details = include;
        self
    }

    /// Enable/disable stack traces in error responses
    pub fn with_stack_trace(mut self, include: bool) -> Self {
        self.include_stack_trace = include;
        self
    }

    /// Add a custom error page for a specific status code
    pub fn with_custom_error_page(mut self, status: StatusCode, html: String) -> Self {
        self.custom_error_pages.insert(status, html);
        self
    }

    /// Enable/disable error logging
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_errors = enabled;
        self
    }

    /// Set the threshold for error vs warning logs (default: 500)
    pub fn with_error_log_threshold(mut self, threshold: u16) -> Self {
        self.error_log_threshold = threshold;
        self
    }
}

#[async_trait::async_trait]
impl Middleware for ErrorHandlerMiddleware {
    async fn after(&self, res: &mut Response) -> Result<()> {
        // Only process error responses
        if res.status.is_success() {
            return Ok(());
        }

        // Log the error if logging is enabled
        if self.log_errors {
            let status_code = res.status.as_u16();
            let log_message = format!(
                "HTTP {} - {} (Body length: {} bytes)",
                status_code,
                res.status.canonical_reason().unwrap_or("Unknown"),
                res.body.len()
            );

            if status_code >= self.error_log_threshold {
                error!("{}", log_message);
            } else if status_code >= 400 {
                warn!("{}", log_message);
            } else {
                debug!("{}", log_message);
            }
        }

        Ok(())
    }
}

impl Default for ErrorHandlerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

// fn _get_user_friendly_message(&self, status: StatusCode) -> &'static str {
//     match status {
//         StatusCode::BAD_REQUEST => "The request could not be understood by the server.",
//         StatusCode::UNAUTHORIZED => "Authentication is required to access this resource.",
//         StatusCode::FORBIDDEN => "You don't have permission to access this resource.",
//         StatusCode::NOT_FOUND => "The requested resource could not be found.",
//         StatusCode::METHOD_NOT_ALLOWED => {
//             "The request method is not allowed for this resource."
//         }
//         StatusCode::CONFLICT => "The request conflicts with the current state of the resource.",
//         StatusCode::UNPROCESSABLE_ENTITY => {
//             "The request was well-formed but contains semantic errors."
//         }
//         StatusCode::TOO_MANY_REQUESTS => "Too many requests. Please try again later.",
//         StatusCode::INTERNAL_SERVER_ERROR => "An internal server error occurred.",
//         StatusCode::NOT_IMPLEMENTED => "This feature is not yet implemented.",
//         StatusCode::BAD_GATEWAY => {
//             "The server received an invalid response from an upstream server."
//         }
//         StatusCode::SERVICE_UNAVAILABLE => "The service is temporarily unavailable.",
//         StatusCode::GATEWAY_TIMEOUT => {
//             "The server did not receive a timely response from an upstream server."
//         }
//         _ => "An error occurred while processing your request.",
//     }
// }
