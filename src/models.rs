use serde::{Deserialize, Serialize, Serializer};
use sqlx::{FromRow, PgPool};
use chrono::NaiveDateTime;

fn serialize_naive_datetime_as_rfc3339<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let utc_dt = dt.and_utc();
    serializer.serialize_str(&utc_dt.to_rfc3339())
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Ref {
    pub code: String,
    pub path: String,
    pub point: i32,
    #[serde(serialize_with = "serialize_naive_datetime_as_rfc3339")]
    pub created_at: NaiveDateTime,
    #[serde(serialize_with = "serialize_naive_datetime_as_rfc3339")]
    pub updated_at: NaiveDateTime,
}

impl Ref {
    pub async fn insert(
        pool: &PgPool,
        code: &str,
        path: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Ref>(
            "INSERT INTO refs (code, path) VALUES ($1, $2::ltree) \
             RETURNING code, path::text as path, created_at, updated_at",
        )
        .bind(code)
        .bind(path)
        .fetch_one(pool)
        .await
    }

    pub async fn select_by_code(pool: &PgPool, code: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Ref>(
            "SELECT code, path::text as path, created_at, updated_at FROM refs WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_point(pool: &PgPool, code: &str, increment: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE refs SET point = point + $1, updated_at = NOW() WHERE code = $2",
        )
        .bind(increment)
        .bind(code)
        .execute(pool)
        .await
        .map(|_| ())
    }
}
