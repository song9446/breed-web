use diesel::prelude::*;
use serde::ser::{Serialize};
use crate::models::Character;
use crate::response::Response;
use diesel::pg::PgConnection;
use rand::distributions::{Poisson, Distribution};
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Debug, Serialize)]
#[serde(tag = "name")]
pub enum Event{
    born{ matherid: i32, fatherid: i32, babyid: i32 },
    pregnant{ matherid: i32, fatherid: i32, fetus: Character },
    summon_start{ summon: Character },
    summon_finish{ summonid: i32 },
}


impl Event {
    pub fn push_events(events: &mut Vec<Event>, ch: &Character, chs: &[Character]) {
        if !ch.born && ch.image_server_domain.is_some() {
            if ch.matherid.is_some() && ch.fatherid.is_some() {
                events.push(Event::born{ matherid: ch.matherid.unwrap(), fatherid: ch.fatherid.unwrap(), babyid: ch.id });
            } else {
                events.push(Event::summon_finish{ summonid: ch.id });
            }
            return;
        }
    }
    pub fn apply(&self, conn: &PgConnection) -> Result<(), Response> {
        match self {
            Event::born{babyid, fatherid, matherid} => {
                use crate::schema::characters::dsl::*;
                diesel::update(characters.filter(id.eq(babyid)))
                    .set(born.eq(true))
                    .execute(conn)?;
            },
            Event::summon_finish{summonid} => {
                use crate::schema::characters::dsl::*;
                diesel::update(characters.filter(id.eq(summonid)))
                    .set(born.eq(true))
                    .execute(conn)?;
            },
            Event::pregnant{matherid, fatherid, fetus} => {
            },
            Event::summon_start{summon} => {
            },
            _ => {
            },
        };
        Ok(())
    }
}
