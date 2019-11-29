#![recursion_limit="256"]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use anyhow::{Result, Error, anyhow};
use std::time::Duration;

use futures::{pin_mut, SinkExt, StreamExt, TryFutureExt, FutureExt, future, select};
use futures::stream::{FuturesUnordered, SelectAll, SplitStream, SplitSink};
use tokio::net::{TcpListener, TcpStream};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_hdr_async, WebSocketStream, accept_async};
use tokio::{runtime, task};
use tokio::time::timeout;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use bytes::{BufMut, Buf};
use prost::Message as ProstMessage;

mod schema;
#[allow(non_snake_case)]
mod model;
mod session;
mod name;

mod event {
    include!(concat!(env!("OUT_DIR"), "/event.rs"));
}
mod action {
    include!(concat!(env!("OUT_DIR"), "/action.rs"));
}

use model::{User, Character};
use session::{Session};

pub type WebSocket = WebSocketStream<TcpStream>;
pub type DbPool = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;

const LOGIN_TIMEOUT :Duration = Duration::from_secs(5);
const REACTION_TIMEOUT :Duration = Duration::from_secs(5);

struct GlobalSession {
    _drop: oneshot,

}

async fn run(addr: &str, dbpool: DbPool) {
    let mut listener = TcpListener::bind(addr).await.unwrap();
    let socket_global_sessions = slab::Slab::new();
    while let Ok((sock, _)) = listener.accept().await {
        let dbpool = dbpool.clone();
        tokio::spawn(async move {
            let mut ws = match accept_async(sock).await {
                Ok(ws) => ws,
                Err(err) => {
                    println!("{:?}", err);
                    return;
                }
            }; 
            evloop(ws, dbpool).await;
        });
    }
}

async fn recv_action(ws: &mut WebSocket, timeout_du: Duration) -> Result<action::Action> {
    let binary = match timeout(timeout_du, ws.next()).await? {
        Some(Ok(Message::Binary(msg))) => Ok(msg),
        _ => Err(anyhow!("fail to recv binary message")),
    }?;
    Ok(action::Action::decode(binary)?)
}

async fn evloop(mut ws: WebSocket, dbpool: DbPool) -> Result<()> {
    let msg = recv_action(&mut ws, LOGIN_TIMEOUT).await?;
    let login = match msg.action {
        Some(action::action::Action::Login(login)) => login,
        _ => return Err(anyhow!("must login before do something")),
    };
    let mut session = Session::login(dbpool, login.id, login.pw).await?;
    loop {
        select! {
            msg = recv_action(&mut ws, REACTION_TIMEOUT).fuse() => {
                let msg = msg?;
                let res = match msg.action {
                    Some(action::action::Action::Login(login)) => Err(anyhow!("already logined!")),
                    Some(action::action::Action::Marry(marry)) => session.marry(marry.groomid, marry.brideid).await,
                    Some(action::action::Action::Summon(summon)) => session.summon().await,
                    _ => {Ok(())}
                };
                if let Err(err) = res {
                    session.event_queue.push(event::Event{
                        event: Some(event::event::Event::Error ( err.to_string() ))
                    });
                }
            },
            default => {
                // update
                // send events
            },
            complete => {
                break;
            },
        }
    }
    Ok(())
}


#[tokio::main(basic_scheduler)]
async fn main() {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //let timeout = std::env::var("TIMEOUT_DURATION").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pgpool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pg pool.");

    run("127.0.0.1:3012", pgpool).await;
}
