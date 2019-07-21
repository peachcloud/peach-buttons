use std::{error, str};

use jsonrpc_core::{types::error::Error, ErrorCode};
use snafu::Snafu;

pub type BoxError = Box<dyn error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ButtonError {
    #[snafu(display("Invalid parameters. Subscription rejected"))]
    RejectSubscription,
}

impl From<ButtonError> for Error {
    fn from(err: ButtonError) -> Self {
        match &err {
            ButtonError::RejectSubscription => Error {
                code: ErrorCode::ParseError,
                message: "Invalid parameters. Subscription request rejected".to_string(),
                data: None,
            },
        }
    }
}
