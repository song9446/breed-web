use rand::prelude::*;
use rand::distributions::StandardNormal;

use chrono::prelude::*;

use crate::errors::ServiceError;

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
pub const RANDOM_CHARACTER_GEN_ACTION_ID: i32 = 1;
lazy_static!{
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
    pub fn new(userid: String, password: String, email: String, nickname: String) -> Self {
        NewUser {
            userid: userid,
            password: password,
            email: email,
            nickname: nickname,
            mana_charge_per_day: MANA_CHARGE_PER_DAY,
            max_mana: MAX_MANA,
            summon_mana_cost: SUMMON_MANA_COST,
        }
    }
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
    pub url: String,
    pub jobid: Option<i32>,
    pub height: f64,
    pub created_at: chrono::NaiveDateTime,
    pub stats: Vec<i32>,
    pub stateid: i32,
    //pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name="characters"]
pub struct NewCharacter {
    pub firstname: String,
    pub matherid: Option<i32>,
    pub fatherid: Option<i32>,
    pub ownerid: Option<i32>,
    pub seed: Vec<f64>,
    pub url: String,
    pub height: f64,
    pub stats: Vec<i32>,
}
impl NewCharacter {
    fn new(seed: Vec<f64>) -> NewCharacter {
        let mut seed_bytes = [0; SEED_LEN*8];
        byteorder::LittleEndian::write_f64_into(&seed, &mut seed_bytes);
        NewCharacter {
            firstname: crate::names::gen().to_string(),
            matherid:None,
            fatherid:None,
            ownerid:None,
            seed,
            url: base64::encode_config(&seed_bytes[..], base64::URL_SAFE),
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
    pub fn synthesize(dbconn: &PgConnection, owner_id:i32, mather_id: i32, father_id: i32) -> Result<NewCharacter, ServiceError>  {
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
