use serde::{Deserialize, Serialize};
use time::{OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Website {
    id: uuid::Uuid,
    url: String,
    word_count: i32,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

pub struct InsertWebsiteDao {
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub word_count: i32,
}

impl Website {
    pub async fn insert(pool: &sqlx::PgPool, insert_website: InsertWebsiteDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO websites (id, url, word_count)
            VALUES ($1, $2, $3)
            RETURNING id, url, word_count, created_at, updated_at
            "#,
            insert_website.id,
            insert_website.url.to_string(),
            insert_website.word_count
        )
            .fetch_one(pool)
            .await?;

        Ok(Self {
            id: row.id,
            url: row.url,
            word_count: row.word_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
    
    pub async fn upsert(pool: &sqlx::PgPool, insert_website: InsertWebsiteDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO websites (id, url, word_count)
            VALUES ($1, $2, $3)
            ON CONFLICT (url) DO UPDATE
            SET word_count = $3
            RETURNING id, url, word_count, created_at, updated_at
            "#,
            insert_website.id,
            insert_website.url.to_string(),
            insert_website.word_count
        )
            .fetch_one(pool)
            .await?;

        Ok(Self {
            id: row.id,
            url: row.url,
            word_count: row.word_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn find_by_url(pool: &sqlx::PgPool, url: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Website,
            r#"
            SELECT id, url, word_count, created_at, updated_at
            FROM websites
            WHERE url = $1
            "#,
            url
        ).fetch_one(pool).await
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Website,
            r#"
            SELECT id, url, word_count, created_at, updated_at
            FROM websites
            WHERE id = $1
            "#,
            id
        ).fetch_one(pool).await
    }
}