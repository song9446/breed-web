use diesel::prelude::*;
use serde::ser::{Serialize};
use crate::models::{Character, NewCharacter};
use crate::response::Response;
use diesel::pg::PgConnection;
use rand::distributions::{Poisson, Distribution};

use rand::{thread_rng, Rng};
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Debug, Serialize)]
#[serde(tag = "name")]
pub enum Event{
    born{ matherid: i32, fatherid: i32, baby: Character },
    pregnant{ matherid: i32, fatherid: i32, fetus: Option<Character> },
    summon_start{ summon: Character },
    summon_finish{ summon: Character},
    married{ groomid: i32, brideid: i32 }
}


impl Event {
    pub fn push_events(events: &mut Vec<Event>, ch: &Character, chs: &[Character]) {
        if !ch.born && ch.image_server_domain.is_some() {
            if ch.matherid.is_some() || ch.fatherid.is_some() {
                events.push(Event::born{ matherid: ch.matherid.unwrap(), fatherid: ch.fatherid.unwrap(), baby: (*ch).clone() });
            } else {
                events.push(Event::summon_finish{ summon: (*ch).clone() });
            }
            return;
        }
        let mut childrens_num = 0;
        for c in chs {
            if let Some(matherid) = c.matherid {
                if matherid == ch.id {
                    childrens_num += 1;
                }
            }
            if let Some(fatherid) = c.fatherid {
                if fatherid == ch.id {
                    childrens_num += 1;
                }
            }
        }
        if ch.partnerid.is_some() && childrens_num == 0 && ch.gender == 0 {
            for i in 0..rand::thread_rng().gen_range(1,3){
                events.push(Event::pregnant{ matherid: ch.id, fatherid: ch.partnerid.unwrap(), fetus: None});
            }
        }

    }
    pub fn apply(self, conn: &PgConnection, user_id: i32) -> Result<Event, Response> {
        match &self {
            Event::born{baby, fatherid, matherid} => {
                use crate::schema::characters::dsl::*;
                diesel::update(characters.filter(id.eq(baby.id)))
                    .set(born.eq(true))
                    .execute(conn)?;
            },
            Event::summon_finish{summon} => {
                use crate::schema::characters::dsl::*;
                diesel::update(characters.filter(id.eq(summon.id)))
                    .set(born.eq(true))
                    .execute(conn)?;
            },
            Event::pregnant{matherid: mid, fatherid: fid, fetus} => {
                use crate::schema::characters::dsl::*;
                let character = NewCharacter::random().with_owner(user_id).with_parents(*mid, *fid);
                let fetus = diesel::insert_into(characters)
                    .values(&character)
                    .get_result::<Character>(conn)
                    .map_err(|dberr| Response::internal_server_error(&format!("{:?}", dberr)))?;
                return Ok(Event::pregnant{matherid: *mid, fatherid: *fid, fetus: Some(fetus)});
            },
            Event::summon_start{summon} => {
            },
            _ => {
            },
        };
        Ok(self)
    }
}
