use std::path::PathBuf;
use crossbeam_channel::unbounded;
use sqlx::PgPool;
use search_engine::services::{Crawler, FileReader, PageParser, SitePool, TextPool};

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the CSV file - https://tranco-list.eu/list/XJJZN/1000000, need to add "rank,root_domain" as the first line
    let sites_path = dotenv!("SITES_PATH");
    let sites_path_buf = PathBuf::from(sites_path);
    // Open the lemmatizer JSON file - https://github.com/conaticus/search-engine-crawler/blob/dev/lemmatizedMap.json - credit to conaticus
    let lemmatizer_json_path = dotenv!("LEMMATIZER_JSON_PATH");
    let lemmatizer_json_path_buf = PathBuf::from(lemmatizer_json_path);
    let host = dotenv!("DB_HOST");
    let port = dotenv!("DB_PORT").parse().expect("DB_PORT must be a number");
    let username = dotenv!("DB_USERNAME");
    let password = dotenv!("DB_PASSWORD");
    let database = dotenv!("DB_DATABASE");
    let db_options = sqlx::postgres::PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(username)
        .password(password)
        .database(database);
    let db = PgPool::connect_with(db_options).await.map_err(|e| {println!("Error connecting to database: {:?}", e);e})?;

    // url channel
    let (url_sender, url_receiver) = unbounded();
    // crawler channel
    let (crawler_sender, crawler_receiver) = unbounded();
    // page channel
    let (page_sender, page_receiver) = unbounded();
    // text channel
    let (text_sender, text_receiver) = unbounded();

    // Create a new FileReader
    let file_reader = FileReader::new(sites_path_buf, url_sender).await.map_err(|e| {
        println!("Error creating file reader: {:?}", e);
        e
    })?;
    // Create a site pool
    let site_pool = SitePool::new(url_receiver, crawler_sender);
    // Create multiple crawlers
    let mut crawlers = Vec::new();
    for _ in 0..10 {
        let crawler = Crawler::new(page_sender.clone(), crawler_receiver.clone());
        crawlers.push(crawler);
    }
    // Create Page Parser
    let page_parser = PageParser::new(page_receiver, text_sender, lemmatizer_json_path_buf).map_err(|e| {
        format!("Error creating page parser: {:?}", e);
        e
    })?;

    // Create a text pool
    let text_pool = TextPool::new(text_receiver, db);

    // Start all services
    tokio::spawn(async move {
        file_reader.start().await;
    });
    tokio::spawn(async move {
        site_pool.start().await;
    });
    for crawler in crawlers {
        tokio::spawn(async move {
            crawler.start().await;
        });
    }
    tokio::spawn(async move {
        page_parser.start().await;
    });
    tokio::spawn(async move {
        text_pool.start().await;
    }).await.map_err(|e| {
        println!("Error starting text pool: {:?}", e);
        e
    })?;
    Ok(())
}
