use diesel::prelude::*;
use diesel::pg::PgConnection;
use anyhow::{anyhow, Context, Result};
use super::*;
use model::{User, Character, UserWithPassword, NewUser, NewCharacter};
use chrono::naive::NaiveDateTime;

use actix_threadpool as threadpool;

pub async fn join(dbpool: DbPool, id: String, pw: String, email: String, nickname: String) -> Result<()> {
    threadpool::run(move || {
        let conn: &PgConnection = &dbpool.get().unwrap();
        let new_user = NewUser::new(id, pw, email, nickname).context("invalid password")?;
        {
            use crate::schema::users::dsl::*;
            diesel::insert_into(users)
                .values(&new_user)
                .execute(conn)
                .context("email or id already exist")?;
        }
        Ok(())
    }).await?
}


pub struct Session {
    user: User,
    characters: Vec<Character>,
    dbpool: DbPool,
    pub event_queue: Vec<event::Event>,
}
impl Session {
    pub async fn login(dbpool: DbPool, id_: String, pw: String) -> Result<Session>{
        let dbpool_ = dbpool.clone(); 
        let (user, characters) = threadpool::run(move || {
            let conn: &PgConnection = &dbpool_.get().unwrap();
            let user_with_password = {
                use crate::schema::users::dsl::*;
                users
                    .select((password, id, nickname, mana, mana_charge_per_day, max_mana, summon_mana_cost, mana_updated_at)) 
                    .filter(userid.eq(id_))
                    .first::<UserWithPassword>(conn)
                    .context("user id not found")
            }?;
            if false == bcrypt::verify(pw, &user_with_password.password).context("hash error")? {
                return Err(anyhow!("incorrect password"));
            }
            let characters = {
                use crate::schema::characters::dsl::*;
                characters
                    .filter(ownerid.eq(user_with_password.id))
                    .load::<Character>(conn)
                    .context("no characters")
            }?;
            Ok((user_with_password.without_password(), characters))
        }).await??;
        let mut event_queue = Vec::new();
        event_queue.push(event::Event{
            event: Some(event::event::Event::Logined(event::Logined{
                user: Some(event::User{ 
                    id: user.id,
                    nickname: user.nickname.clone(),
                    mana: user.mana,
                    mana_charge_per_day: user.mana_charge_per_day,
                    max_mana: user.max_mana,
                    summon_mana_cost: user.summon_mana_cost,
                    mana_updated_at: user.mana_updated_at.timestamp(),
                }),
                characters: characters.iter().map(|ch| (*ch).clone().into() ).collect(),
            })),
        });
        Ok(Session{
            user, characters, event_queue, dbpool
        })
    }
    fn character_find_by_id(&mut self, id: i32) -> Option<&mut Character>{
        self.characters.iter_mut().find(|ch| ch.id == id)
    }
    fn mana_update(&mut self){
        let now = chrono::Utc::now().naive_utc();
        let charged_mana = (self.user.mana_charge_per_day as f64 * ((now - self.user.mana_updated_at).num_milliseconds() as f64 / (1000*3600*24) as f64)) as i32;
        let updated_mana = core::cmp::min(self.user.mana + charged_mana, self.user.max_mana);
        self.user.mana = updated_mana;
        self.user.mana_updated_at = now;
    }
    async fn summon(&mut self) -> Result<()> {
        self.mana_update();
        if self.user.mana < self.user.summon_mana_cost {
            return Err(anyhow!("mana not enough for summoning"));
        }
        let user_id: i32 = self.user.id;
        let dbpool = self.dbpool.clone();
        let summon = threadpool::run(move || {
            use crate::schema::characters::dsl::*;
            let dbconn: &PgConnection = &dbpool.get().unwrap();
            //let mqconn = &mut mqpool.get().unwrap();
            let character = NewCharacter::random().with_owner(user_id);
            diesel::insert_into(characters)
                .values(&character)
                .get_result::<Character>(dbconn)
        }).await??;
        self.user.mana -= self.user.summon_mana_cost;
        //self.event_queue.push();
        Ok(())
    }
    async fn marry(&mut self, groomid: i32, brideid: i32) -> Result<()> {
        if groomid == brideid {
            return Err(anyhow!("you cannot marry yourself"));
        }
        let groom = self.character_find_by_id(groomid);
        let bride = self.character_find_by_id(brideid);
        if groom.is_none() || bride.is_none() {
            return Err(anyhow!("you don't have that characters"));
        }
        if groom.partnerid.is_some() || bride.partnerid.is_some() {
            return Err(anyhow!("they already have a partner"));
        }
        self.event_queue.push(event::Event{
            event: Some(event::event::Event::Married(event::Married{
                groomid, 
                brideid,
            })),
        });
        Ok(())
    }
    pub async fn action(&mut self, action: action::Action) -> Result<()> {
    }
}


