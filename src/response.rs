use serde::ser::{Serialize};
use actix_web::{error::{ResponseError, BlockingError}, HttpResponse};
use std::fmt;

#[allow(non_snake_case, non_camel_case_types)]
#[derive(Debug, Serialize)]
pub enum Response<T: Serialize>{
    //#[display(fmt = "Error {}: {}", code, message)]
    error{ code: i32, message: String, },
    data(T),
}

pub type ErrorResponse = Response<()>;

impl<T: Serialize> fmt::Display for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Response::error{code, message} => write!(f, "Error {}: {}", code, message),
			_ => write!(f, "Data"),
		}
    }
}

impl<T: Serialize> Response <T> {
    pub fn bad_request(msg: &str) -> Response<T> {
        Response::error{code: 400, message: msg.into()}
    }
    pub fn unauthorized(msg: &str) -> Response<T> {
        Response::error{code: 401, message: msg.into()}
    }
    pub fn not_found(msg: &str) -> Response<T> {
        Response::error{code: 404, message: msg.into()}
    }
    pub fn internal_server_error(msg: &str) -> Response<T> {
        Response::error{code: 500, message: msg.into()}
    }
    pub fn ok(data: T) -> HttpResponse {
        HttpResponse::Ok().json(Response::data(data))
    }
}

impl ResponseError for ErrorResponse {
    fn error_response(&self) -> HttpResponse {
        match self {
            Response::error{code: 400, message: _} => HttpResponse::BadRequest().json(self),
            Response::error{code: 401, message: _} => HttpResponse::Unauthorized().json(self),
            Response::error{code: 404, message: _} => HttpResponse::NotFound().json(self),
            Response::error{code: 500, message: _} => HttpResponse::InternalServerError().json(self),
            _ => HttpResponse::InternalServerError().json(self),
        }
    }
    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}
/*
impl<T: Serialize + fmt::Display + fmt::Debug> ResponseError for Response<T> {
    fn error_response(&self) -> HttpResponse {
        match self {
            Response::error{code: 400, message: _} => HttpResponse::BadRequest().json(self),
            Response::error{code: 401, message: _} => HttpResponse::Unauthorized().json(self),
            Response::error{code: 404, message: _} => HttpResponse::NotFound().json(self),
            Response::error{code: 500, message: _} => HttpResponse::InternalServerError().json(self),
            _ => HttpResponse::InternalServerError().json(self),
        }
    }
}*/

impl From<BlockingError<ErrorResponse>> for ErrorResponse {
    fn from(error: BlockingError<ErrorResponse>) -> ErrorResponse {
        match error {
            BlockingError::Error(err) => err,
            BlockingError::Canceled => ErrorResponse::internal_server_error(""),
        }
    }
}

impl<T: Serialize> From<diesel::result::Error> for Response<T> {
    fn from(_: diesel::result::Error) -> Response<T> {
		Response::internal_server_error("")
	}
}
