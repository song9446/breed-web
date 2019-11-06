use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

use diesel::RunQueryDsl;
use diesel::QueryDsl;

use futures::Future;

use actix_web::{error::BlockingError, web, HttpResponse};
//use futures::Future;

use crate::models::{NewUser, User, Pool, Character, NewCharacter};
use crate::errors::{ServiceError};

use actix_session::Session;

use rand::prelude::*;
use rand::distributions::StandardNormal;

use serde_json::json;

use r2d2_beanstalkd::BeanstalkdConnectionManager;

use chrono::prelude::*;

pub type MqPool = r2d2::Pool<BeanstalkdConnectionManager>;

#[derive(Serialize, Deserialize)]
pub struct GameData {
	pub user: User,
	pub characters: Vec<Character>,
}
#[derive(Serialize, Deserialize)]
pub struct JoinForm {
    pub id: String,
    pub password: String,
    pub email: String,
    pub nickname: String,
}
pub fn join(join_form: web::Json<JoinForm>, session: Session, pool: web::Data<Pool>)  -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || {
        use crate::schema::users::dsl::*;
        let conn: &PgConnection = &pool.get().unwrap();
        let join_form = join_form.into_inner();
        let new_user = NewUser::new(join_form.id, join_form.password, join_form.email, join_form.nickname);
        let query = diesel::insert_into(users).values(&new_user);
        let debug = diesel::debug_query::<diesel::pg::Pg, _>(&query);
        println!("The insert query: {:?}", debug);
        diesel::insert_into(users)
            .values(&new_user)
            .execute(conn)
            .map_err(|_db_error| ServiceError::BadRequest("user does not exists".into()))
    })
	.from_err::<ServiceError>()
	.and_then(move |_| {
		Ok(HttpResponse::Ok().finish())
	})
}



#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub userid: String,
    pub password: String,
}
pub fn login(data: web::Json<LoginData>, session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || {
        let conn: &PgConnection = &pool.get().unwrap();
        let data = data.into_inner();
		let user = {
			use crate::schema::users::dsl::*;
			users
				.select((id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at)) 
				.filter(userid.eq(data.userid).and(password.eq(data.password)))
				.load::<User>(conn)
				.map_err(|_db_error| ServiceError::InternalServerError)
				.and_then(|mut result| result.pop().ok_or(ServiceError::BadRequest("user does not exists".into())))?
		};
		let characters = {
			use crate::schema::characters::dsl::*;
			characters
				.filter(ownerid.eq(user.id))
				.load::<Character>(conn)
				.map_err(|_db_error| ServiceError::InternalServerError)?
		};
		Ok(GameData{user, characters})
    })
	.from_err::<ServiceError>()
    .and_then(move |gamedata| {
		Ok(HttpResponse::Ok().json(gamedata))
	})
}

pub fn reload_session(session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    let user_id = session.get("id")
        .map_err(|err| ServiceError::Unauthorized)
        .and_then(|val| val.ok_or(ServiceError::Unauthorized));
    web::block(move || {
        let user_id: i32 = user_id?;
        let conn: &PgConnection = &pool.get().unwrap();
		let user = {
			use crate::schema::users::dsl::*;
			users
				.select((id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at)) 
				.filter(id.eq(user_id))
				.load::<User>(conn)
				.map_err(|_db_error| ServiceError::InternalServerError)
				.and_then(|mut result| result.pop().ok_or(ServiceError::BadRequest("user does not exists".into())))?
		};
		let characters = {
			use crate::schema::characters::dsl::*;
			characters
				.filter(ownerid.eq(user.id))
				.load::<Character>(conn)
				.map_err(|_db_error| ServiceError::InternalServerError)?
		};
		Ok(GameData{user, characters})
    })
	.from_err::<ServiceError>()
    .and_then(move |gamedata| {
		Ok(HttpResponse::Ok().json(gamedata))
	})
}

pub fn logout(session: Session) -> Result<HttpResponse, ServiceError>{
    session.clear();
    Ok(HttpResponse::Ok().finish())
}

pub fn summon_character(session: Session, dbpool: web::Data<Pool>, mqpool: web::Data<MqPool>) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    let user_id = session.get("id")
        .map_err(|err| ServiceError::Unauthorized)
        .and_then(|val| val.ok_or(ServiceError::Unauthorized));
    web::block(move || {
        let user_id: i32 = user_id?;
        let dbconn: &PgConnection = &dbpool.get().unwrap();
        let mqconn = &mut mqpool.get().unwrap();
        let character = NewCharacter::random(user_id);
        dbconn.transaction(|| {
            use crate::schema::users::dsl::*;
            let user = users.select((crate::schema::users::id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at)).find(1).first::<User>(dbconn)
                .map_err(|_db_error| ServiceError::InternalServerError)?;
            let now = Utc::now().naive_utc();
            let charged_mana = (user.mana_charge_per_day as f64 * ((now - user.mana_updated_at).num_milliseconds() as f64 / (1000*3600*24) as f64)) as i32;
            let updated_mana = user.mana + charged_mana - user.summon_mana_cost;
            if updated_mana < 0 {
                return Err(ServiceError::NotEnoughMana);
            }
            diesel::update(&user).set((mana.eq(updated_mana), mana_updated_at.eq(now))).execute(dbconn)
                .map_err(|_db_error| ServiceError::InternalServerError)?;
            let seed_json = json!({
                "seed": &character.seed,
                "url": &character.url,
            });
            mqconn
                .put(&seed_json.to_string(), 0, 0, 10)
                .map_err(|_db_error| ServiceError::InternalServerError)?;
            use crate::schema::characters::dsl::*;
            diesel::insert_into(characters)
                .values(&character)
                .get_result::<Character>(dbconn)
                .map_err(|_db_error| ServiceError::InternalServerError)
        })
    })
	.from_err::<ServiceError>()
	.and_then(move |inserted_character| Ok(HttpResponse::Ok().json(inserted_character)))
}
