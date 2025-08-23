use rocket_db_pools::{mongodb::Client, Database};

#[derive(Database)]
#[database("bearodata")]
pub struct BearoData(Client);
