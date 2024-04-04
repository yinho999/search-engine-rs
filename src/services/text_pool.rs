use std::collections::HashMap;
use spider::page::Page;
use sqlx::types::BigDecimal;
use crate::models;
use crate::models::website::{InsertWebsiteDao, Website};

pub struct TextPool {
    // `text_sender` is a mpsc channel sender that sends a vector of processed texts to the text pool.
    text_rx: crossbeam_channel:: Receiver<(Page, Vec<String>)>,
    // `db` is a postgres connection pool.
    db: sqlx::PgPool,
}

impl TextPool {
    /// Create a new TextPool instance.
    pub fn new(text_rx: crossbeam_channel:: Receiver<(Page, Vec<String>)>, db: sqlx::PgPool) -> Self {
        Self {
            text_rx,
            db,
        }
    }
    /// Start the text pool in background.
    pub async fn start(self) {
        // Loop to receive texts from the text receiver.
        while let Ok((page, texts)) = self.text_rx.recv() {
            let term_frequency = self.tf(texts.clone());
            let total_count = texts.len();
            // Save the texts to the database.
            match self.save_texts(page, total_count as i64, term_frequency).await {
                Ok(_) => {
                    println!("Texts saved successfully.");
                }
                Err(e) => {
                    eprintln!("Error saving texts: {:?}", e);
                    continue;
                }
            }
        }
    }
    /// Save the texts to the database.
    async fn save_texts(&self, page: Page, count: i64, term_frequency: HashMap<String, i64>) -> Result<(), Box<dyn std::error::Error>> {
        let page_url = url::Url::parse(page.get_url())?;
        // Find website by url, create a new website if it doesn't exist.
        let website = models::website::Website::find_by_url(&self.db, page_url.as_str().to_string()).await;
        match website {
            Ok(website) => {
                // Update the word count of the website.
                Self::update_website(&self, website, count, term_frequency).await?;
            }
            Err(sqlx::Error::RowNotFound) => {
                // Insert the website to the database
                Self::insert_website(&self, page_url.as_str(), count as i32, term_frequency).await?;
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
        Ok(())
    }

    async fn insert_website(&self, url: &str, word_count: i32, term_frequency: HashMap<String, i64>) -> Result<(), Box<dyn std::error::Error>> {
        let insert_website = InsertWebsiteDao {
            url: url::Url::parse(url).map_err(|_| "Error parsing URL")?,
            word_count,
        };
        // Insert the website to the database
        let website = models::website::Website::insert(&self.db, insert_website).await.map_err(|e| format!("Error inserting website: {:?}", e))?;
        // Insert the keywords to the database
        for (keyword, frequency) in term_frequency.iter() {
            self.insert_keyword(&website, keyword.clone(), *frequency as i32).await.map_err(|e| format!("Error inserting keyword: {:?}", e))?;
        }
        Ok(())
    }

    async fn update_website(&self, website: models::website::Website, count: i64, term_frequency: HashMap<String, i64>) -> Result<(), Box<dyn std::error::Error>> {
        // Update the word count of the website.
        models::website::Website::update_word_count(&self.db, website.id, count as i32).await.map_err(|e| format!("Error updating word count: {:?}", e))?;
        // Remove all the keywords associated with the website.
        models::website_keywords::WebsiteKeywords::delete_by_website(&self.db, website.id).await.map_err(|e| format!("Error deleting website keywords: {:?}", e))?;
        // Insert the keywords to the database
        for (keyword, frequency) in term_frequency.iter() {
            self.insert_keyword(&website, keyword.clone(), *frequency as i32).await.map_err(|e| format!("Error inserting keyword: {:?}", e))?;
        }
        Ok(())
    }

    async fn insert_keyword(&self, website: &Website, keyword: String, frequency: i32) -> Result<(), Box<dyn std::error::Error>> {
        let keyword = models::keyword::Keyword::find_or_create(&self.db, &keyword).await.map_err(|e| format!("Error finding or creating keyword: {:?}", e))?;
        // Insert the keyword to the database
        let insert_website_keywords = models::website_keywords::InsertWebsiteKeywordsDao {
            keyword_id: keyword.id,
            website_id: website.id,
            frequency,
        };
        // Insert the website keywords to the database
        models::website_keywords::WebsiteKeywords::insert(&self.db, insert_website_keywords).await?;
        let total_docs_with_keyword = models::website_keywords::WebsiteKeywords::count_by_keyword_id(&self.db, keyword.id).await.map_err(|e| format!("Error counting total docs with keyword: {:?}", e))?;
        let total_docs = models::website::Website::count(&self.db).await.map_err(|e| format!("Error counting total docs: {:?}", e))?;
        let idf = self.idf(total_docs_with_keyword, total_docs);
        let normalized_frequency = frequency as f64 / website.word_count as f64;
        let tfidf = self.tfidf(normalized_frequency, idf);
        let insert_website_keyword_tfidf = models::website_keyword_tfidf::InsertWebsiteKeywordTfidfDao {
            website_id: website.id,
            keyword_id: keyword.id,
            tf: BigDecimal::try_from(frequency as f64).map_err(|_| "Error converting to BigDecimal")?,
            idf: BigDecimal::try_from(idf).map_err(|_| "Error converting to BigDecimal")?,
            tfidf: BigDecimal::try_from(tfidf).map_err(|_| "Error converting to BigDecimal")?,
        };
        // Insert the website keyword tfidf to the database
        models::website_keyword_tfidf::WebsiteKeywordTfidf::upsert_by_website_keyword(&self.db, insert_website_keyword_tfidf).await.map_err(|e| format!("Error upserting website keyword tfidf: {:?}", e))?;
        Ok(())
    }

    /// Calculate the term frequency of the text.
    fn tf(&self, texts: Vec<String>) -> HashMap<String, i64> {
        let mut tf: HashMap<String, i64> = HashMap::new();
        for text in texts.iter() {
            let count = tf.entry(text.clone()).or_insert(0);
            *count += 1;
        }
        tf
    }

    /// Calculate the inverse document frequency of the keyword.
    fn idf(&self, total_docs_with_keyword: i64, total_docs: i64) -> f64 {
        // Calculate the idf
        let idf = 1.0 + (total_docs as f64 / total_docs_with_keyword as f64).ln();
        idf
    }

    // Calculate TF-IDF
    fn tfidf(&self, tf: f64, idf: f64) -> f64 {
        tf * idf
    }
}