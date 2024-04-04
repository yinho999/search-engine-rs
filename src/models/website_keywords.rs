use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteKeywords {
    id: uuid::Uuid,
    keyword_id: uuid::Uuid,
    website_id: uuid::Uuid,
    frequency: i32,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertWebsiteKeywordsDao {
    pub keyword_id: uuid::Uuid,
    pub website_id: uuid::Uuid,
    pub frequency: i32
}

impl WebsiteKeywords {

    pub async fn insert(pool: &PgPool, insert_website_keywords_dao: InsertWebsiteKeywordsDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO website_keywords (keyword_id, website_id, frequency)
            VALUES ($1, $2, $3)
            RETURNING id, keyword_id, website_id, frequency, created_at, updated_at
            "#,
            insert_website_keywords_dao.keyword_id,
            insert_website_keywords_dao.website_id,
            insert_website_keywords_dao.frequency
            
        )
        .fetch_one(pool)
        .await?;

        Ok(Self {
            id: row.id,
            keyword_id: row.keyword_id,
            website_id: row.website_id,
            frequency: row.frequency,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
    

    pub async fn find_by_keyword_id(pool: &PgPool, keyword_id: uuid::Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebsiteKeywords,
            r#"
            SELECT id, keyword_id, website_id, frequency,created_at, updated_at
            FROM website_keywords
            WHERE keyword_id = $1
            "#,
            keyword_id
        ).fetch_all(pool).await
    }

    pub async fn find_by_website_id(pool: &PgPool, website_id: uuid::Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebsiteKeywords,
            r#"
            SELECT id, keyword_id, website_id, frequency, created_at, updated_at
            FROM website_keywords
            WHERE website_id = $1
            "#,
            website_id
        ).fetch_all(pool).await
    }

    pub async fn find_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            WebsiteKeywords,
            r#"
            SELECT id, keyword_id, website_id, frequency, created_at, updated_at
            FROM website_keywords
            WHERE id = $1
            "#,
            id
        ).fetch_optional(pool).await
    }

    pub async fn count_by_keyword_id(pool: &PgPool, keyword_id: uuid::Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT COUNT(*)
            FROM website_keywords
            WHERE keyword_id = $1
            "#,
            keyword_id
        ).fetch_one(pool).await.map(|row| row.count.unwrap())
    }

    pub async fn count_by_website_id(pool: &PgPool, website_id: uuid::Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT COUNT(*)
            FROM website_keywords
            WHERE website_id = $1
            "#,
            website_id
        ).fetch_one(pool).await.map(|row| row.count.unwrap())
    }
    
    pub async fn delete_by_website(pool: &PgPool, website_id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM website_keywords
            WHERE website_id = $1
            "#,
            website_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}