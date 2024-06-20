use axum::{
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderValue,
    },
    response::{AppendHeaders, IntoResponse},
};
use serde::Serialize;

use crate::err::{ApiError, ModelError, OperationError};

pub mod http;
// pub mod zmq;

pub struct NoDownload<T>(T);
impl<T: IntoResponse> IntoResponse for NoDownload<T> {
    fn into_response(self) -> axum::response::Response {
        (
            AppendHeaders([(CONTENT_DISPOSITION, HeaderValue::from_static("inline"))]),
            self.0,
        )
            .into_response()
    }
}
pub struct MsgPak<T>(T);
impl<T: IntoResponse + Serialize> IntoResponse for MsgPak<T> {
    fn into_response(self) -> axum::response::Response {
        let mut resp = match rmp_serde::encode::to_vec_named(&self.0) {
            Ok(b) => b.into_response(),
            Err(_) => ApiError::Operation(OperationError::Model(ModelError::Parse {
                desc: "msgpak serialized error <- node cannot be serialized".to_owned(),
            }))
            .into_response(),
        };
        resp.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.messagepack"),
        );
        resp
    }
}
