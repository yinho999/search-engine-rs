/// SitePool is a pool of sites that are to be crawled.
pub struct SitePool {
    // `site_receiver` is a mpsc channel receiver that receives a URL to crawl.
    site_receiver: crossbeam_channel::Receiver<url::Url>,
    // `crawler_sender` is a mpsc channel sender that sends a URL to the crawler.
    crawler_sender: crossbeam_channel::Sender<url::Url>,
}

impl SitePool {
    /// Create a new SitePool instance.
    pub fn new(site_receiver: crossbeam_channel::Receiver<url::Url>, crawler_sender: crossbeam_channel::Sender<url::Url>) -> Self {
        Self {
            site_receiver,
            crawler_sender,
        }
    }

    /// Start the site pool in background.
    pub async fn start(self) {
        // Loop to receive URLs from the site receiver.
        while let Ok(url) = self.site_receiver.recv() {
            
            // Send the URL to the crawler.
            match self.crawler_sender.send(url){
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error sending URL to crawler: {:?}", e);
                }
            }
        }
    }
}