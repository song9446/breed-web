use r2d2_beanstalkd::BeanstalkdConnectionManager;
use r2d2_beanstalkd::{ Client, Error };
use serde_json::json;
pub const SEED_TUBE:&str = "seed";
pub const GEN_IMAGE_REQ_TUBE:&str = "gen_image";
fn init(conn: Beanstalkd, seed: Vec<f64>, id: i32) -> Result<(), Error> {
}
fn gen_image(conn: Beanstalkd, seed: Vec<f64>, id: i32) -> Result<(), Error>{
    conn.tube(GEN_IMAGE_REQ_TUBE)?;
    let seed_json = json!({
        "seed": &inserted_character.seed,
        "id": &(inserted_character.id.to_string() + ".png"),
    });
    conn.put_default(&seed.to_vec())?;
    Ok(())
}
fn new_seed(seed) -> Result<(), Error>{
    conn.("seeds")?;
    conn.
}
