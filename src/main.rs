#![allow(dead_code)] // usful in dev mode
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate r2d2_beanstalkd;
extern crate base64;
extern crate byteorder;


use actix_session::{CookieSession};
use actix_web::{middleware, web, App, HttpServer,};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use r2d2_beanstalkd::BeanstalkdConnectionManager;


mod schema;
mod models;
mod names;
mod handlers;
mod response;

pub type MqPool = r2d2::Pool<BeanstalkdConnectionManager>;


fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
	let secret_key = std::env::var("SECRET_KEY").unwrap_or("0123".repeat(8));
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pgpool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pg pool.");
    let mq_url = std::env::var("MESSAGE_QUEUE_URL").expect("MESSAGE_QUEUE_URL must be set");
    let mut it = mq_url.split(":");
    let manager = BeanstalkdConnectionManager::new(it.next().unwrap().to_string(), it.next().unwrap().parse::<u16>().unwrap());
    let mqpool: MqPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create mq pool.");
    let domain = std::env::var("DOMAIN").expect("DOMAIN must be set");
    let domain_for_cookiesession = domain.clone();

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(pgpool.clone())
            .data(mqpool.clone())
            .wrap(actix_cors::Cors::new()
                  .allowed_origin("http://127.0.0.1:5000")
            )
            .wrap(middleware::Logger::default())
            .wrap(CookieSession::signed(secret_key.as_bytes())
                 .domain(domain_for_cookiesession.as_str())
                 .max_age_time(chrono::Duration::days(1))
                 .secure(false)
            )
            .data(web::JsonConfig::default().limit(4096))
            .service(
                web::resource("/session")
                .route(web::get().to_async(handlers::reload_session))
                //.route(web::put().to_async(handlers::login))
                .route(web::post().to_async(handlers::login))
                .route(web::delete().to(handlers::logout))
                )
            .service(
                web::resource("/accounts")
                .route(web::post().to_async(handlers::join))
                //.route(web::delete().to_async(handlers::remove_account))
                )
            //.service(
                //web::resource("/update")
                //.route(web::get().to_async())
                //.route(web::post().to_async(handlers::update))
            //    )
    })
    .bind(domain)?
    .run()
}
