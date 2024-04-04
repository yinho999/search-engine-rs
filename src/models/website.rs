use serde::{Deserialize, Serialize};
use time::{OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Website {
    pub(crate) id: uuid::Uuid,
    url: String,
    pub(crate) word_count: i32,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

pub struct InsertWebsiteDao {
    pub url: url::Url,
    pub word_count: i32,
}

impl Website {
    pub async fn insert(pool: &sqlx::PgPool, insert_website: InsertWebsiteDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO websites (url, word_count)
            VALUES ($1, $2)
            RETURNING id, url, word_count, created_at, updated_at
            "#,
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
            INSERT INTO websites (url, word_count)
            VALUES ($1, $2)
            ON CONFLICT (url) DO UPDATE
            SET word_count = $2
            RETURNING id, url, word_count, created_at, updated_at
            "#,
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
    
    pub async fn find_or_create(pool: &sqlx::PgPool, url: &str, word_count: i32) -> Result<Self, sqlx::Error> {
        match Self::find_by_url(pool, url.to_string()).await{
            Ok(website) => Ok(website),
            Err(sqlx::Error::RowNotFound) => {
                let insert_website = InsertWebsiteDao{
                    url: url::Url::parse(url).unwrap(),
                    word_count,
                };
                Self::insert(pool, insert_website).await
            },
            Err(e) => {
                Err(e)
            }
        }
    }
    pub async fn update_word_count(pool: &sqlx::PgPool, id: uuid::Uuid, word_count: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE websites
            SET word_count = $1
            WHERE id = $2
            "#,
            word_count,
            id
        )
            .execute(pool)
            .await?;
        Ok(())
    }
    pub async fn count(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM websites
            "#,
        )
            .fetch_one(pool)
            .await?;
        let count = row.count.ok_or(sqlx::Error::RowNotFound)?;
        Ok(count)
    }
}