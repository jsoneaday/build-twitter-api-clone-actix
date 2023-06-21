use chrono::{DateTime, Utc};
use fake::{faker::{internet::en::Username, name::en::{FirstName, LastName}}, Fake};
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = sqlx::postgres::PgPool::connect("postgres://tester:tester@localhost:5432/tester")
        .await
        .unwrap();

    let mut tx = conn.begin().await.unwrap();

    let insert_result = sqlx::query_as::<_, EntityId>("insert into profile (user_name, full_name) values ($1, $2) returning id")
        .bind(Username().fake::<String>())
        .bind(format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>()))
        .fetch_one(&mut tx)
        .await;

    let query_result = sqlx::query_as::<_, Profile>("select * from profile where id = $1")
        //.bind(243245645)
        .bind(insert_result.unwrap().id)
        .fetch_one(&mut tx)
        .await;

    match query_result {
        Ok(profile) => {
            println!("Profile: {:?}", profile);
            _ = tx.commit().await;
        },
        Err(_) => {
            println!("Error: profile {} not found", 24324);
            let rollback_result = tx.rollback().await;
            println!("Rollback result: {:?}", rollback_result);
        }
    };

    Ok(())
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
struct EntityId {
    pub id: i64
}

#[allow(unused)]
#[derive(FromRow, Deserialize, Serialize, Debug)]
struct Profile {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_name: String,
    pub full_name: String
}