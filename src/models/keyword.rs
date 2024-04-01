use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Keyword {
    id: uuid::Uuid,
    keyword: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertKeywordDao {
    pub id: uuid::Uuid,
    pub keyword: String,
}

impl Keyword {
    pub async fn insert(pool: &sqlx::PgPool, insert_keyword: InsertKeywordDao) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO keywords (id, keyword)
            VALUES ($1, $2)
            RETURNING id, keyword, created_at, updated_at
            "#,
            insert_keyword.id,
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

    pub async fn find_by_word(pool: &sqlx::PgPool, keyword: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Keyword,
            r#"
            SELECT id, keyword, created_at, updated_at
            FROM keywords
            WHERE keyword = $1
            "#,
            keyword
        ).fetch_all(pool).await
    }
}
