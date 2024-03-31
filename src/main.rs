use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt as TokioStreamExt;
// For using next() on streams
use csv_async::AsyncDeserializer;
use spider::website::Website;
use tokio::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteData {
    rank: i32,
    root_domain: String,
}

#[tokio::main]
async fn main() {
    // Open the CSV file
    let file = File::open("assets/websites-10.csv").await.unwrap();

    // Create an AsyncDeserializer from the file
    let mut rdr = AsyncDeserializer::from_reader(file);

    // Create a stream from the deserializer
    let mut records = rdr.deserialize::<WebsiteData>();
    
    // Crawler vector
    let mut crawlers = Vec::new();
    
    // Use StreamExt to asynchronously process each record
    while let Some(result) = records.next().await {
        match result {
            Ok(data) => {
                 crawlers.push( tokio::spawn(async move {
                    // Initialize Website instance with Amazon's URL
                    let mut website: Website = Website::new(&data.root_domain);

                    // Subscribe to receive pages. Adjust the channel size as needed.
                    let mut rx = website.subscribe(1).unwrap();
                    let mut rx_guard = website.subscribe_guard().unwrap();

                    // Spawn a task to handle received pages
                    tokio::spawn(async move {
                        while let Ok(page) = rx.recv().await {
                            println!("Page URL: {:?}", page.get_url());
                            println!("Page HTML: {:?}", page.get_html());
                            // Here you can process the page further, e.g., take a screenshot or scrape specific data
                            rx_guard.inc();
                        }
                    });

                    // Start crawling the website
                    website.crawl().await;
                }));
            }
            Err(e) => eprintln!("Error reading CSV: {}", e),
        }
    }
    // Wait for all crawlers to finish
    for crawler in crawlers {
        crawler.await.unwrap();
    }

}
