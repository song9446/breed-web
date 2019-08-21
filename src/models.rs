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

const HEIGHT_MEAN:f64 = 160.0;
const HEIGHT_VAR:f64 = 10.0;
const STATS_NUM:i32 = 7;
const STATS_MEAN:f64 = 5.0;
const STATS_VAR:f64 = 5.0;
const SEED_LEN: usize = 512;
const RANDOM_CHARACTER_GEN_ACTION_ID: i32 = 1;
lazy_static!{
static ref RANDOM_CHARACTER_GEN_DURATION: chrono::Duration = chrono::Duration::days(1);
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32,
    pub userid: String,
    pub password: String,
    pub email: String,
    pub nickname: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Queryable, Debug)]
pub struct Character<'a> {
    pub id: i32,
    pub firstname: &'a str,
    pub parentid: Option<i32>,
    pub ownerid: Option<i32>,
    pub seed: Vec<f64>,
    pub url: &'a str,
    pub jobid: Option<i32>,
    pub height: f64,
    pub stats: Vec<i32>,
    pub stateid: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
            url: base_62::encode(&seed_bytes),
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
    pub fn gerneate_random_action_try(dbconn: &PgConnection, ownerid:i32) -> Result<NewCharacter, ServiceError> {
        use crate::schema::users_actions::dsl::*;
        let last_create_action_at = users_actions
            .select(diesel::dsl::max(created_at))
            .filter(userid.eq(ownerid).and(
                    actionid.eq(RANDOM_CHARACTER_GEN_ACTION_ID)))
            .first::<Option<chrono::NaiveDateTime>>(dbconn)?
            .unwrap_or(chrono::NaiveDateTime::from_timestamp(0,0));
        if last_create_action_at + *RANDOM_CHARACTER_GEN_DURATION > Utc::now().naive_utc() {
            return Err(ServiceError::Unexpired);
        }
        let mut rng = rand::thread_rng();
        let seed = (0..SEED_LEN).map(|_| rng.gen::<f64>()).collect();
        Ok(Self::new(seed).with_owner(ownerid))
    }
}
