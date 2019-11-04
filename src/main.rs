#![allow(dead_code)] // usful in dev mode
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate r2d2_beanstalkd;
extern crate base_62;
extern crate byteorder;


use actix_session::{Session, CookieSession};
use actix_web::{middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use r2d2_beanstalkd::BeanstalkdConnectionManager;


mod schema;
mod models;
mod errors;
mod utils;
mod names;
mod handlers;

pub type MqPool = r2d2::Pool<BeanstalkdConnectionManager>;

fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info");
    env_logger::init();
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
    let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(pgpool.clone())
            .data(mqpool.clone())
            .wrap(middleware::Logger::default())
            .wrap(CookieSession::signed(utils::SECRET_KEY.as_bytes())
                    .domain(domain.as_str())
                    .max_age_time(chrono::Duration::days(1))
                    .secure(false)
            )
            .data(web::JsonConfig::default().limit(4096))
            .service(
                web::resource("/session")
                .route(web::get().to_async(handlers::reload_session))
                .route(web::put().to_async(handlers::login))
                .route(web::post().to_async(handlers::join))
                .route(web::delete().to(handlers::logout))
                )
            //.service(
                //web::resource("/update")
                //.route(web::get().to_async())
                //.route(web::post().to_async(handlers::update))
            //    )
    })
    .bind("127.0.0.1:3000")?
    .run()
}
