use diesel::prelude::*;
use diesel::pg::PgConnection;

use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::result::{DatabaseErrorKind, Error as DBError};

use futures::Future;

use actix_web::{web, HttpResponse};
//use futures::Future;

use crate::models::{NewUser, User, UserWithPassword, UserManaUpdated, Pool, Character, NewCharacter};

use actix_session::Session;

use serde_json::json;

use r2d2_beanstalkd::BeanstalkdConnectionManager;

use chrono::prelude::*;

use crate::response::{Response, ErrorResponse};

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
pub fn join(join_form: web::Json<JoinForm>, _session: Session, pool: web::Data<Pool>)  -> impl Future<Item = HttpResponse, Error = ErrorResponse> {
    web::block(move || {
        use crate::schema::users::dsl::*;
        let conn: &PgConnection = &pool.get().unwrap();
        let join_form = join_form.into_inner();
        let new_user = NewUser::new(join_form.id, join_form.password, join_form.email, join_form.nickname)
			.map_err(|_| Response::bad_request("invalid password"))?;
        diesel::insert_into(users)
            .values(&new_user)
            .execute(conn)
            .map_err(|err| match err {
                DBError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => Response::bad_request("id or email already exist"),
                _ => Response::internal_server_error(""),
            })
    })
	.from_err::<ErrorResponse>()
	.and_then(|_| {
		Ok(HttpResponse::Ok().json(Response::data(())))
	})
}



#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub id: String,
    pub password: String,
}
pub fn login(data: web::Json<LoginData>, session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = ErrorResponse> {
    web::block(move || {
        let conn: &PgConnection = &pool.get().unwrap();
        let data = data.into_inner();
		let user_with_password = {
			use crate::schema::users::dsl::*;
			users
				.select((password, id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at)) 
				.filter(userid.eq(data.id))
				.first::<UserWithPassword>(conn)
				.map_err(|err| match err {
					DBError::NotFound => Response::bad_request("user not exist"),
					_ => Response::internal_server_error("user select error"),
				})?
		};
		bcrypt::verify(data.password, &user_with_password.password)
			.map_err(|_| Response::internal_server_error("hash error"))
			.and_then(|res| match res {
				true => Ok(()),
				false => Err(Response::bad_request("incorrect password")),
			})?;
		let characters = {
			use crate::schema::characters::dsl::*;
			characters
				.filter(ownerid.eq(user_with_password.id))
				.load::<Character>(conn)
				.map_err(|_| Response::internal_server_error("no characters"))?
		};
		Ok(GameData{user: user_with_password.without_password(), characters})
    })
	.from_err::<ErrorResponse>()
    .and_then(move |gamedata| {
		session.set("id", gamedata.user.id)
			.map_err(|_| Response::internal_server_error("session set"))?;
		Ok(Response::ok(gamedata))
	})
}

pub fn reload_session(session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = ErrorResponse> {
    let user_id = session.get("id")
        .map_err(|_| Response::unauthorized(""))
        .and_then(|val| val.ok_or(Response::unauthorized("")));
    web::block(move || {
        let user_id: i32 = user_id?;
        let conn: &PgConnection = &pool.get().unwrap();
		let user = {
			use crate::schema::users::dsl::*;
            users
				.select((id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at))
				.find(user_id)
				.first::<User>(conn)
				.map_err(|err| match err {
					DBError::NotFound => Response::bad_request("id or password is incorrect"),
					_ => Response::internal_server_error(""),
				})?
		};
		let characters = {
			use crate::schema::characters::dsl::*;
			characters
				.filter(ownerid.eq(user.id))
				.load::<Character>(conn)
				.map_err(|_| Response::internal_server_error(""))?
		};
		Ok(GameData{user, characters})
    })
	.from_err::<ErrorResponse>()
    .and_then(|gamedata| Ok(Response::ok(gamedata)))
}

pub fn logout(session: Session) -> Result<HttpResponse, ErrorResponse>{
    session.clear();
	Ok(Response::ok(()))
}

#[derive(Serialize, Deserialize)]
pub struct SummonData {
	pub user: UserManaUpdated,
	pub character: Character,
}
pub fn summon_character(session: Session, dbpool: web::Data<Pool>, mqpool: web::Data<MqPool>) -> impl Future<Item = HttpResponse, Error = ErrorResponse> {
    let user_id = session.get("id")
        .map_err(|_| Response::unauthorized(""))
        .and_then(|val| val.ok_or(Response::unauthorized("")));
    web::block(move || {
        let user_id: i32 = user_id?;
        let dbconn: &PgConnection = &dbpool.get().unwrap();
        let mqconn = &mut mqpool.get().unwrap();
        let character = NewCharacter::random(user_id);
        dbconn.transaction(|| {
            use crate::schema::users::dsl::*;
            let user = users
				.select((crate::schema::users::id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at))
				.find(user_id)
				.first::<User>(dbconn)
				.map_err(|err| match err {
					DBError::NotFound => Response::bad_request("user not exist"),
					_ => Response::internal_server_error(""),
				})?;
            let now = Utc::now().naive_utc();
            let charged_mana = (user.mana_charge_per_day as f64 * ((now - user.mana_updated_at).num_milliseconds() as f64 / (1000*3600*24) as f64)) as i32;
            let updated_mana = core::cmp::min(user.mana + charged_mana, user.max_mana) - user.summon_mana_cost;
            if updated_mana < 0 {
                return Err(Response::bad_request("not enough mana"));
            }
            let user_mana_updated = UserManaUpdated{ mana: updated_mana, mana_updated_at: now };
            diesel::update(&user).set(&user_mana_updated).execute(dbconn)
				.map_err(|_| Response::internal_server_error(""))?;
            let seed_json = json!({
                "seed": &character.seed,
                "url": &character.url,
            });
            mqconn
                .put(&seed_json.to_string(), 0, 0, 10)
				.map_err(|_| Response::internal_server_error(""))?;
            use crate::schema::characters::dsl::*;
            let inserted_character = diesel::insert_into(characters)
                .values(&character)
                .get_result::<Character>(dbconn)
				.map_err(|_| Response::internal_server_error(""))?;
            Ok(SummonData {
                user: user_mana_updated,
                character: inserted_character,
            })
        })
    })
	.from_err::<ErrorResponse>()
	.and_then(|summon_data| Ok(Response::ok(summon_data)))

}
