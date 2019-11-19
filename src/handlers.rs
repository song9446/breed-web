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

//use r2d2_beanstalkd::BeanstalkdConnectionManager;

use chrono::prelude::*;

use crate::response::{Response};
use crate::events::{Event};

//pub type MqPool = r2d2::Pool<BeanstalkdConnectionManager>;


#[derive(Serialize, Deserialize)]
pub struct JoinForm {
    pub id: String,
    pub password: String,
    pub email: String,
    pub nickname: String,
}
pub fn join(join_form: web::Json<JoinForm>, _session: Session, pool: web::Data<Pool>)  -> impl Future<Item = HttpResponse, Error = Response> {
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
	.from_err::<Response>()
	.and_then(|_| {
		Ok(HttpResponse::Ok().json(Response::ok)) })
}



#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub id: String,
    pub password: String,
}
pub fn login(data: web::Json<LoginData>, session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Response> {
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
		//Ok(Response::login{ user: user_with_password.without_password(), characters })
        Ok((user_with_password.without_password(), characters))
    })
	.from_err::<Response>()
    .and_then(move |result| {
        let (user, characters) = result;
		session.set("id", user.id)
			.map_err(|_| Response::internal_server_error("session set"))?;
        let response = Response::login{ user, characters };
		Ok(HttpResponse::Ok().json(response))
	})
}

pub fn reload_session(session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Response> {
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
		Ok(Response::login{ user, characters })
    })
	.from_err::<Response>()
    .and_then(|gamedata| Ok(HttpResponse::Ok().json(gamedata)))
		
}

pub fn logout(session: Session) -> Result<HttpResponse, Response>{
    session.clear();
    Ok(HttpResponse::Ok().json(Response::ok))
}

pub fn summon_character(session: Session, dbpool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Response> {
    let user_id = session.get("id")
        .map_err(|_| Response::unauthorized(""))
        .and_then(|val| val.ok_or(Response::unauthorized("")));
    web::block(move || {
        let user_id: i32 = user_id?;
        let dbconn: &PgConnection = &dbpool.get().unwrap();
        //let mqconn = &mut mqpool.get().unwrap();
        let character = NewCharacter::random().with_owner(user_id);
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
            use crate::schema::characters::dsl::*;
            let summon = diesel::insert_into(characters)
                .values(&character)
                .get_result::<Character>(dbconn)
				.map_err(|dberr| Response::internal_server_error(&format!("{:?}", dberr)))?;
            /*let seed_json = json!({
                "seed": &inserted_character.seed,
                "url": &(inserted_character.id.to_string() + ".png"),
            });
            mqconn
                .put(&seed_json.to_string(), 0, 0, 10)
				.map_err(|_| Response::internal_server_error(""))?;*/
            Ok(Response::event{ event: Event::summon_start{ summon } })
        })
    })
	.from_err::<Response>()
	.and_then(|summon_event| Ok(HttpResponse::Ok().json(summon_event)))
}


pub fn update(session: Session, dbpool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Response> {
    let user_id = session.get("id")
        .map_err(|_| Response::unauthorized(""))
        .and_then(|val| val.ok_or(Response::unauthorized("")));
    web::block(move || {
        let user_id: i32 = user_id?;
        let conn: &PgConnection = &dbpool.get().unwrap();
        conn.transaction(|| {
            use crate::schema::characters::dsl::*;
            let chs = characters
                    .filter(ownerid.eq(user_id))
                    .load::<Character>(conn)
                    .map_err(|_| Response::internal_server_error(""))?;
            let mut events: Vec<Event> = Vec::new();
            for ch in &chs {
                Event::push_events(&mut events, &ch, &chs);
            }
            for event in &events {
                event.apply(&conn)?;
            }
            Ok(Response::events{ events })
        })
    })
	.from_err::<Response>()
	.and_then(|res| Ok(HttpResponse::Ok().json(res)))
}
