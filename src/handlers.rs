use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

use diesel::RunQueryDsl;
use diesel::QueryDsl;

use futures::Future;

use actix_web::{error::BlockingError, web, HttpResponse};
//use futures::Future;

use crate::models::{NewUser, User, Pool, NewCharacter};
use crate::errors::{ServiceError};

use actix_session::Session;

use rand::prelude::*;
use rand::distributions::StandardNormal;

use serde_json::json;

use r2d2_beanstalkd::BeanstalkdConnectionManager;

pub type MqPool = r2d2::Pool<BeanstalkdConnectionManager>;

const HEIGHT_MEAN:f64 = 160.0;
const HEIGHT_VAR:f64 = 10.0;
const STATS_NUM:i32 = 7;
const STATS_MEAN:f64 = 5.0;
const STATS_VAR:f64 = 5.0;
const SEED_LEN: usize = 512;

pub fn join(new_user: web::Json<NewUser>, session: Session, pool: web::Data<Pool>)  -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || {
        use crate::schema::users::dsl::*;
        let conn: &PgConnection = &pool.get().unwrap();

        diesel::insert_into(users)
            .values(&new_user.into_inner())
            .returning(id)
            .get_result(conn)
            .map_err(|_db_error| ServiceError::BadRequest("user does not exists".into()))
    })
    .then(move |res: Result<i32, BlockingError<ServiceError>>| match res {
        Ok(inserted_id) => {
            session.set("id", inserted_id).map_err(|_| ServiceError::Unauthorized)?;
            Ok(HttpResponse::Ok().finish())
        },
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}


#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub userid: String,
    pub password: String,
}
pub fn login(data: web::Json<LoginData>, session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || {
        use crate::schema::users::dsl::*;
        let conn: &PgConnection = &pool.get().unwrap();
        let data = data.into_inner();
        users
            .select(id) 
            .filter(userid.eq(data.userid).and(password.eq(data.password)))
            .load::<i32>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("user does not exists".into()))
            .and_then(|mut result| {
                if let Some(user_id) = result.pop() {
                    return Ok(user_id)
                }
                Err(ServiceError::BadRequest("user does not exists".into()))
            })
    })
    .then(move |res| match res {
        Ok(user_id) => {
            session.set("id", user_id).map_err(|_| ServiceError::Unauthorized)?;
            Ok(HttpResponse::Ok().finish())
        },
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn logout(session: Session) -> Result<HttpResponse, ServiceError>{
    session.clear();
    Ok(HttpResponse::Ok().finish())
}

pub fn create_character(session: Session, dbpool: web::Data<Pool>, mqpool: web::Data<MqPool>) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    let user_id = session.get("id").unwrap_or(None);
    let data = sessio.get("").unwrap_or(None);
    web::block(move || {
        use crate::schema::characters::dsl::*;
        let user_id = match user_id {
            Some(x)=>x,
            None=> {
                return Err(ServiceError::Unauthorized);
            }
        };
        let dbconn: &PgConnection = &dbpool.get().unwrap();
        let mqconn = &mut mqpool.get().unwrap();
        let character = NewCharacter::gerneate_random_action_try(dbconn, user_id)?;
        dbconn.transaction(|| {
            diesel::insert_into(characters)
                .values(&character)
                .execute(dbconn)
                .map_err(Into::into)
                //.map_err(|_db_error| ServiceError::InternalServerError)
                .and_then(move |_| {
                    let seed_json = json!({
                        "seed": &character.seed,
                        "url": &character.url,
                    });
                    mqconn
                        .put(&seed_json.to_string(), 0, 0, 10)
                        .map_err(Into::into)
                })
        })
    })
    .then(move |res| match res {
        Ok(_) => {
            Ok(HttpResponse::Ok().finish())
        },
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}
