use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Keyword {
    pub(crate) id: uuid::Uuid,
    pub(crate) keyword: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct InsertKeywordDao {
    pub keyword: String,
}

impl Keyword {
    pub async fn insert(pool: &sqlx::PgPool, insert_keyword: InsertKeywordDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO keywords ( keyword)
            VALUES ($1)
            RETURNING id, keyword, created_at, updated_at
            "#,
            insert_keyword.keyword,
        )
            .fetch_one(pool)
            .await?;

        Ok(Self {
            id: row.id,
            keyword: row.keyword,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn find_by_word(pool: &sqlx::PgPool, keyword: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Keyword,
            r#"
            SELECT id, keyword, created_at, updated_at
            FROM keywords
            WHERE keyword = $1
            "#,
            keyword
        ).fetch_one(pool).await
    }
    
    pub async fn find_or_create(pool: &sqlx::PgPool, keyword: &str) -> Result<Self, sqlx::Error> {
        match Self::find_by_word(pool, keyword).await{
            Ok(keyword) => Ok(keyword),
            Err(sqlx::Error::RowNotFound) => {
                let insert_keyword = InsertKeywordDao{
                    keyword: keyword.to_string(),
                };
                Self::insert(pool, insert_keyword).await
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}
