use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, )]
pub struct WebsiteKeywordTfidf {
    pub id: Uuid,
    pub website_id: Uuid,
    pub keyword_id: Uuid,
    pub tf: BigDecimal,
    pub idf: BigDecimal,
    pub tfidf: BigDecimal,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

pub struct InsertWebsiteKeywordTfidfDao {
    pub website_id: Uuid,
    pub keyword_id: Uuid,
    pub tf: BigDecimal,
    pub idf: BigDecimal,
    pub tfidf: BigDecimal,
    
}

impl WebsiteKeywordTfidf {
    pub async fn insert(pool: &sqlx::PgPool, insert_website_keyword_tfidf: InsertWebsiteKeywordTfidfDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO website_keyword_tfidf (website_id, keyword_id, tf, idf, tfidf)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, website_id, keyword_id, tf, idf, tfidf, created_at, updated_at
            "#,
            insert_website_keyword_tfidf.website_id,
            insert_website_keyword_tfidf.keyword_id,
            insert_website_keyword_tfidf.tf,
            insert_website_keyword_tfidf.idf,
            insert_website_keyword_tfidf.tfidf
        )
            .fetch_one(pool)
            .await?;

        Ok(Self {
            id: row.id,
            website_id: row.website_id,
            keyword_id: row.keyword_id,
            tf: row.tf,
            idf: row.idf,
            tfidf: row.tfidf,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn find_by_website_keyword(pool: &sqlx::PgPool, website_id: Uuid, keyword_id: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            WebsiteKeywordTfidf,
            r#"
            SELECT id, website_id, keyword_id, tf, idf, tfidf, created_at, updated_at
            FROM website_keyword_tfidf
            WHERE website_id = $1 AND keyword_id = $2
            "#,
            website_id,
            keyword_id
        ).fetch_one(pool).await
    }

    pub async fn upsert_by_website_keyword(pool: &sqlx::PgPool, insert_website_keyword_tfidf: InsertWebsiteKeywordTfidfDao) -> Result<Self, sqlx::Error> {
        // Check if the website keyword tfidf exists
        match Self::find_by_website_keyword(pool, insert_website_keyword_tfidf.website_id, insert_website_keyword_tfidf.keyword_id).await {
            Ok(website_keyword_tfidf) => {
                // Update the website keyword tfidf
                let row = sqlx::query!(
                    r#"
                    UPDATE website_keyword_tfidf
                    SET tf = $1, idf = $2, tfidf = $3
                    WHERE id = $4
                    RETURNING id, website_id, keyword_id, tf, idf, tfidf, created_at, updated_at
                    "#,
                    insert_website_keyword_tfidf.tf,
                    insert_website_keyword_tfidf.idf,
                    insert_website_keyword_tfidf.tfidf,
                    website_keyword_tfidf.id
                )
                    .fetch_one(pool)
                    .await?;
                Ok(Self {
                    id: row.id,
                    website_id: row.website_id,
                    keyword_id: row.keyword_id,
                    tf: row.tf,
                    idf: row.idf,
                    tfidf: row.tfidf,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            }
            Err(sqlx::Error::RowNotFound) => {
                // Insert the website keyword tfidf to the database
                Self::insert(pool, insert_website_keyword_tfidf).await
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}