use scraper::Selector;
use spider::page::Page;

pub struct PageParser {
    // `page_rx` is a mpsc channel receiver that receives a page from the page pool.
    page_rx: tokio::sync::mpsc::Receiver<Page>,
    // `site_sender` is a mpsc channel sender that sends a URL to the site pool.
    site_sender: tokio::sync::mpsc::Sender<url::Url>,
    // `db` is a postgres connection pool.
    db: sqlx::PgPool,
}

impl PageParser {
    /// Create a new PageParser instance.
    pub fn new(page_rx: tokio::sync::mpsc::Receiver<Page>, site_sender: tokio::sync::mpsc::Sender<url::Url>, db: sqlx::PgPool) -> Self {
        Self {
            page_rx,
            site_sender,
            db,
        }
    }

    /// Start the page parser in background.
    pub async fn start(mut self) {
        // Loop to receive pages from the page receiver.
        while let Some(page) = self.page_rx.recv().await {
            let html = page.get_html();
            let document = scraper::Html::parse_document(&html);
            // Create a wildcard selector to select all elements
            let selector = match Selector::parse("*") {
                Ok(selector) => selector,
                Err(e) => {
                    eprintln!("Error parsing selector: {:?}", e);
                    continue;
                }
            };

            // Iterate over all elements and collect their text content
            let texts: Vec<String> = document.select(&selector)
                .flat_map(|el| el.text())
                .flat_map(|text| text.split_whitespace().map(str::to_string))
                .collect();
        }
    }
    
    /// Parse all the words in the text and cast them to lowercase.
    fn parse_words(texts: Vec<String>) -> Vec<String> {
        texts.iter()
            .map(|text| text.to_lowercase())
            .collect()
    }
    
    /// 
}