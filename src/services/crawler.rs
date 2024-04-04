use spider::page::Page;
use spider::website::Website;

pub struct Crawler {
    // `page_sender` is a mpsc channel sender that sends a page to the page pool.
    page_sender: crossbeam_channel::Sender<Page>,

    // `url_reader` is a reader that reads the URL to crawl.
    url_reader: crossbeam_channel::Receiver<url::Url>,
}

impl Crawler {
    /// Create a new Crawler instance.
    pub fn new(page_sender: crossbeam_channel::Sender<Page>, url_reader:crossbeam_channel::Receiver<url::Url>) -> Self {
        Self {
            page_sender,
            url_reader,
        }
    }

    /// Start the crawler in background.
    pub async fn start(self) {
        while let Ok(url) = self.url_reader.recv() {

            // Start the crawler in background.
            // Initialize Website instance with Amazon's URL
            let mut website: Website = Website::new(&url.to_string());

            // Subscribe to receive pages. Adjust the channel size as needed.
            let mut rx = website.subscribe(3).unwrap();
            let mut rx_guard = website.subscribe_guard().unwrap();
            let page_sender = self.page_sender.clone();
            // Spawn a task to handle received pages
            tokio::spawn(async move {
                while let Ok(page) = rx.recv().await {
                    println!("Page URL: {:?}", page.get_url());
                    // Send the page to the page pool.
                    match page_sender.send(page) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error sending page to page pool: {:?}", e);
                        }
                    }
                    // Here you can process the page further, e.g., take a screenshot or scrape specific data
                    rx_guard.inc();
                }
            });

            // Start crawling the website
            website.crawl().await;
        }
    }
}