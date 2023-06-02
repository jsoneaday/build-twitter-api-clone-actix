use serde::Deserialize;
use sqlx::{FromRow, Postgres, Pool};
use std::env;
use dotenv::dotenv;
use sqlx::migrate;

#[allow(unused)]
#[derive(FromRow, Deserialize)]
pub struct EntityId {
    pub id: i64
}

pub trait DbConnGetter {
    type Output;
    fn get_conn(&self) -> &Self::Output;
}

#[derive(Clone)]
pub struct DbRepo {
    conn: Pool<Postgres>
}

impl DbRepo {
    pub async fn init() -> Self {
        Self { conn: get_db_conn().await }
    }
}

impl DbConnGetter for DbRepo {
    type Output = Pool<Postgres>;

    fn get_conn(&self) -> &Self::Output {
        &self.conn    
    }
}

pub async fn get_db_conn() -> Pool<Postgres> {
    dotenv().ok();
    let postgres_host = env::var("POSTGRES_HOST").unwrap();
    let postgres_port = env::var("POSTGRES_PORT").unwrap().parse::<u16>().unwrap();
    let postgres_password = env::var("POSTGRES_PASSWORD").unwrap();
    let postgres_user = env::var("POSTGRES_USER").unwrap();
    let postgres_db = env::var("POSTGRES_DB").unwrap();

    let postgres_url = format!(
        "postgres://{postgres_user}:{postgres_password}@{postgres_host}:{postgres_port}/{postgres_db}"
    );

    let conn = sqlx::postgres::PgPool::connect(&postgres_url).await.unwrap();
    let migrate = migrate!("./migrations").run(&conn).await;
    match migrate {
        Ok(()) => println!("sqlx migration success"),
        Err(e) => println!("sqlx migration error: {:?}", e),
    }
    conn
}
