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

use crate::byteorder::ByteOrder;

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
struct LoginData {
    userid: String,
    password: String,
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

#[derive(Serialize, Deserialize)]
struct Parents {
    mather: i32,
    father: i32,
}
pub fn create_character(session: Session, pool: web::Data<Pool>, mqpool: web::Data<MqPool>, parents_params: Option<web::Json<Parents>>) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    let userid:Option<i32> = session.get("id").unwrap_or(None);
    web::block(move || {
        use crate::schema::characters::dsl::*;
        if userid.is_none() {
            return Err(ServiceError::Unauthorized);
        };
        let dbconn: &PgConnection = &pool.get().unwrap();
        let mqconn = &mut mqpool.get().unwrap();
        let character = gen_character(dbconn, userid, parents_params.map(|item| item.0))?;
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
pub fn gen_character(dbconn: &PgConnection, owner_id:Option<i32>, parents: Option<Parents>) -> Result<NewCharacter, ServiceError> {
    let mut seed_bytes = [0; SEED_LEN*8];
    let (mather, father) = match parents {
        Some(parents) => (Some(parents.mather), Some(parents.father)),
        None => (None, None)
    };
    let seed = if father.is_some() && mather.is_some() {
        random_character_seed()
    } else {
        use crate::schema::characters::dsl::*;
        let seed1 = characters
            .select(seed)
            .filter(id.eq(mather.unwrap()).and(
                    ownerid.eq(owner_id)))
            .get_result(dbconn)?;
        let seed2 = characters
            .select(seed)
            .filter(id.eq(father.unwrap()).and(
                    ownerid.eq(owner_id)))
            .get_result(dbconn)?;
        blend_character_seeds(seed1, seed2)
    };
    byteorder::LittleEndian::write_f64_into(&seed, &mut seed_bytes);
    Ok(NewCharacter{
        firstname: crate::names::gen().to_string(),
        matherid: mather,
        fatherid: father,
        ownerid: owner_id,
        seed: seed,
        url: base_62::encode(&seed_bytes),
        height: SmallRng::from_entropy().sample(StandardNormal)*HEIGHT_VAR+HEIGHT_MEAN,
        stats: (0..STATS_NUM).map(|_| (SmallRng::from_entropy().sample(StandardNormal)*STATS_VAR + STATS_MEAN) as i32).collect::<Vec<i32>>(),
    })
}
pub fn random_character_seed() -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..SEED_LEN).map(|_| rng.gen::<f64>()).collect()
}
pub fn blend_character_seeds(seed1: Vec<f64>, seed2: Vec<f64>) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..SEED_LEN).map(|i| {
        let r = rng.gen::<f64>(); 
        seed1[i]*r+seed2[i]*(1.0-r)
    }).collect()
}
