use serde::ser::{Serialize};
use actix_web::{error::{ResponseError, BlockingError}, HttpResponse};
//use r2d2_beanstalkd::Error as BeanstalkdError;
use std::fmt;
use crate::models::{ User, Character };
use crate::events::{ Event };

#[allow(non_snake_case, non_camel_case_types)]
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Response{
    //#[display(fmt = "Error {}: {}", code, message)]
    error{ code: i32, message: String, },
    login{ user:User, characters: Vec<Character> },
    event { event: Event },
    events{ events: Vec<Event> },
    ok,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Response::error{code, message} => write!(f, "Error {}: {}", code, message),
			_ => write!(f, "Ok response"),
		}
    }
}

impl Response {
    pub fn bad_request(msg: &str) -> Response {
        Response::error{code: 400, message: msg.into()}
    }
    pub fn unauthorized(msg: &str) -> Response {
        Response::error{code: 401, message: msg.into()}
    }
    pub fn not_found(msg: &str) -> Response {
        Response::error{code: 404, message: msg.into()}
    }
    pub fn internal_server_error(msg: &str) -> Response {
        Response::error{code: 500, message: msg.into()}
    }
    /*pub fn ok(self) -> HttpResponse {
        HttpResponse::Ok().json(Response::data(data))
    }*/
}

impl ResponseError for Response {
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

impl From<BlockingError<Response>> for Response {
    fn from(error: BlockingError<Response>) -> Response {
        match error {
            BlockingError::Error(err) => err,
            BlockingError::Canceled => Response::internal_server_error(""),
        }
    }
}

impl From<diesel::result::Error> for Response {
    fn from(_: diesel::result::Error) -> Response {
		Response::internal_server_error("")
	}
}
/*
impl<T: Serialize> From<BeanstalkdError> for Response<T> {
    fn from(_: BeanstalkdError) -> Response<T> {
		Response::internal_server_error("")
	}
}
*/
