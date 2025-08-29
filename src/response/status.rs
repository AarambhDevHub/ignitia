use http::StatusCode;

pub trait StatusCodeExt {
    fn is_informational(&self) -> bool;
    fn is_success(&self) -> bool;
    fn is_redirection(&self) -> bool;
    fn is_client_error(&self) -> bool;
    fn is_server_error(&self) -> bool;
}

impl StatusCodeExt for StatusCode {
    fn is_informational(&self) -> bool {
        self.as_u16() >= 100 && self.as_u16() < 200
    }

    fn is_success(&self) -> bool {
        self.as_u16() >= 200 && self.as_u16() < 300
    }

    fn is_redirection(&self) -> bool {
        self.as_u16() >= 300 && self.as_u16() < 400
    }

    fn is_client_error(&self) -> bool {
        self.as_u16() >= 400 && self.as_u16() < 500
    }

    fn is_server_error(&self) -> bool {
        self.as_u16() >= 500 && self.as_u16() < 600
    }
}
