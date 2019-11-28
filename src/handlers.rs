use diesel::prelude::*;
use diesel::pg::PgConnection;

use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::result::{DatabaseErrorKind, Error as DBError};

use futures::Future;
use futures::stream::Stream;

use url::Url;

use actix_web::{web, HttpResponse, HttpRequest, Error};
use actix_web::client::Client;
//use futures::Future;
//u

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




#[derive(Serialize, Deserialize)]
pub struct Marrier {
    pub groomid: i32,
    pub brideid: i32,
}
pub fn marry(data: web::Json<Marrier>, session: Session, pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Response> {
    let user_id = session.get("id")
        .map_err(|_| Response::unauthorized(""))
        .and_then(|val| val.ok_or(Response::unauthorized("")));
    web::block(move || {
        use crate::schema::characters::dsl::*;
        let data = data.into_inner();
        if data.groomid == data.brideid {
            return Err(Response::bad_request("you cannot marry with yourself"));
        }
        let user_id: i32 = user_id?;
        let conn: &PgConnection = &pool.get().unwrap();
		let marriers = {
			characters
				.filter(id.eq(data.groomid).or(id.eq(data.brideid)))
				.load::<Character>(conn)
				.map_err(|_| Response::internal_server_error("no such characters"))?
		};
        if marriers.len() < 2 {
            return Err(Response::bad_request("you don't have such character"));
        }
        if marriers[0].partnerid.is_some() || marriers[1].partnerid.is_some() {
            return Err(Response::bad_request("character already has a partner"));
        }
        if marriers[0].ownerid.is_none() || marriers[1].ownerid.is_none() || marriers[0].ownerid.unwrap() != user_id || marriers[1].ownerid.unwrap() != user_id {
            return Err(Response::bad_request("you donot have the characters"));
        }
        diesel::update(characters.filter(id.eq(data.groomid))).set(partnerid.eq(data.brideid))
            .execute(conn)?;
        diesel::update(characters.filter(id.eq(data.brideid))).set(partnerid.eq(data.groomid))
            .execute(conn)?;
		//Ok(Response::login{ user: user_with_password.without_password(), characters })
        Ok(Response::event{ event: Event::married{groomid: data.groomid, brideid: data.brideid } })
    })
	.from_err::<Response>()
    .and_then(move |result| {
		Ok(HttpResponse::Ok().json(result))
	})
}

pub fn forward(req: HttpRequest, body: web::Bytes, url: web::Data<Url>, client: web::Data<Client>) -> impl Future<Item = HttpResponse, Error = Error> {
	// Copied from https://github.com/actix/examples/tree/master/http-proxy/src
    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());
    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };
    forwarded_req
        .send_body(body)
        .map_err(Error::from)
        .map(|mut res| {
            let mut client_resp = HttpResponse::build(res.status());
            // Remove `Connection` as per
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
            for (header_name, header_value) in
                res.headers().iter().filter(|(h, _)| *h != "connection")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }
            res.body()
                .into_stream()
                .concat2()
                .map(move |b| client_resp.body(b))
                .map_err(|e| e.into())
        })
        .flatten()
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
            let events:Vec<Event> = events.into_iter().map(|e| e.apply(&conn, user_id).unwrap()).collect();
            Ok(Response::events{ events })
        })
    })
	.from_err::<Response>()
	.and_then(|res| Ok(HttpResponse::Ok().json(res)))
}
