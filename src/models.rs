use rand::prelude::*;
use rand::distributions::StandardNormal;

use crate::response::ErrorResponse;

use crate::byteorder::ByteOrder;

use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::{r2d2::ConnectionManager, PgConnection};
use super::schema::*;
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub const HEIGHT_MEAN:f64 = 160.0;
pub const HEIGHT_VAR:f64 = 10.0;
pub const STATS_NUM:i32 = 7;
pub const STATS_MEAN:f64 = 5.0;
pub const STATS_VAR:f64 = 5.0;
pub const SEED_LEN: usize = 512;
pub const SUMMON_MANA_COST:i32 = 3333;
pub const MAX_MANA:i32 = 10000;
pub const MANA_CHARGE_PER_DAY:i32 = 10000;
//pub const RANDOM_CHARACTER_GEN_ACTION_ID: i32 = 1;

#[derive(Serialize, Deserialize, Queryable)]
pub struct UserWithPassword {
    pub password: String,
    pub id: i32,
    pub nickname: String,
	pub mana: i32,
    pub mana_charge_per_day: i32,
    pub max_mana: i32,
    pub summon_mana_cost: i32,
    pub mana_updated_at: chrono::NaiveDateTime,
}
impl UserWithPassword {
	pub fn without_password(self) -> User {
		User{
			id: self.id,
			nickname: self.nickname,
			mana: self.mana,
			mana_charge_per_day: self.mana_charge_per_day,
			max_mana: self.max_mana,
			summon_mana_cost: self.summon_mana_cost,
			mana_updated_at: self.mana_updated_at,
		}
	}
}
#[derive(Serialize, Deserialize, Queryable, Identifiable, Associations)]
pub struct User {
    pub id: i32,
    pub nickname: String,
	pub mana: i32,
    pub mana_charge_per_day: i32,
    pub max_mana: i32,
    pub summon_mana_cost: i32,
    pub mana_updated_at: chrono::NaiveDateTime,
}
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub userid: String,
    pub password: String,
    pub email: String,
    pub nickname: String,
    pub mana_charge_per_day: i32,
    pub max_mana: i32,
    pub summon_mana_cost: i32,
}
impl NewUser {
    pub fn new(userid: String, plain_password: String, email: String, nickname: String) -> Result<Self, bcrypt::BcryptError> {
        Ok(NewUser {
            userid: userid,
            password: bcrypt::hash(&plain_password, 6)?,//bcrypt::DEFAULT_COST)?,
            email: email,
            nickname: nickname,
            mana_charge_per_day: MANA_CHARGE_PER_DAY,
            max_mana: MAX_MANA,
            summon_mana_cost: SUMMON_MANA_COST,
        })
    }
}
#[derive(Serialize, Deserialize, AsChangeset)]
#[table_name = "users"]
pub struct UserManaUpdated {
    pub mana: i32,
    pub mana_updated_at: chrono::NaiveDateTime,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Queryable, Identifiable, Associations, Debug)]
pub struct Character {
    pub id: i32,
    pub firstname: String,
    pub surname: Option<String>,
    pub matherid: Option<i32>,
    pub fatherid: Option<i32>,
    pub ownerid: Option<i32>,
    pub seed: Vec<f64>,
    pub jobid: Option<i32>,
    pub height: f64,
    pub created_at: chrono::NaiveDateTime,
    pub stats: Vec<i32>,
    //pub stateid: i32,
    //pub created_at: chrono::DateTime<chrono::Utc>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Queryable, Identifiable, Associations, Debug)]
#[table_name = "characters"]
pub struct CharacterWithoutSeed {
    pub id: i32,
    pub firstname: String,
    pub surname: Option<String>,
    pub matherid: Option<i32>,
    pub fatherid: Option<i32>,
    pub ownerid: Option<i32>,
    pub jobid: Option<i32>,
    pub height: f64,
    pub created_at: chrono::NaiveDateTime,
    pub stats: Vec<i32>,
    //pub stateid: i32,
    //pub created_at: chrono::DateTime<chrono::Utc>,
}
impl From<Character> for CharacterWithoutSeed {
    fn from(c: Character) -> Self {
        CharacterWithoutSeed {
            id: c.id,
            firstname: c.firstname,
            surname: c.surname,
            matherid: c.matherid,
            fatherid: c.fatherid,
            ownerid: c.ownerid,
            jobid: c.jobid,
            height: c.height,
            created_at: c.created_at,
            stats: c.stats,
        }
    }
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name="characters"]
pub struct NewCharacter {
    pub firstname: String,
    pub matherid: Option<i32>,
    pub fatherid: Option<i32>,
    pub ownerid: Option<i32>,
    pub seed: Vec<f64>,
    pub height: f64,
    pub stats: Vec<i32>,
}
impl NewCharacter {
    fn new(seed: Vec<f64>) -> NewCharacter {
        NewCharacter {
            firstname: crate::names::gen().to_string(),
            matherid:None,
            fatherid:None,
            ownerid:None,
            seed,
            height: SmallRng::from_entropy().sample(StandardNormal)*HEIGHT_VAR+HEIGHT_MEAN,
            stats: (0..STATS_NUM).map(|_| (SmallRng::from_entropy().sample(StandardNormal)*STATS_VAR + STATS_MEAN) as i32).collect::<Vec<i32>>(),
        }
    }
    fn with_owner(mut self, ownerid: i32) -> Self{
        self.ownerid = Some(ownerid);
        self
    }
    fn with_parents(mut self, matherid: i32, fatherid: i32) -> Self{
        self.matherid = Some(matherid);
        self.fatherid = Some(fatherid); 
        self
    }
    pub fn synthesize(dbconn: &PgConnection, owner_id:i32, mather_id: i32, father_id: i32) -> Result<NewCharacter, ErrorResponse>  {
        use crate::schema::characters::dsl::*;
        let mut rng = rand::thread_rng();
        let seed1:Vec<f64> = characters
            .select(seed)
            .filter(id.eq(mather_id).and(
                    ownerid.eq(owner_id)))
            .get_result(dbconn)?;
        let seed2:Vec<f64> = characters
            .select(seed)
            .filter(id.eq(father_id).and(
                    ownerid.eq(owner_id)))
            .get_result(dbconn)?;
        let new_seed = (0..SEED_LEN).map(|i| {
            let r = rng.gen::<f64>(); 
            seed1[i]*r+seed2[i]*(1.0-r)
        }).collect();
        Ok(Self::new(new_seed).with_owner(owner_id).with_parents(mather_id, father_id))
    }
    pub fn random(ownerid:i32) -> NewCharacter {
        let mut rng = rand::thread_rng();
        let seed = (0..SEED_LEN).map(|_| rng.gen::<f64>()).collect();
        Self::new(seed).with_owner(ownerid)
    }
}
