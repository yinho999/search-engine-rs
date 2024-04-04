use std::error::Error;
use std::path::PathBuf;
use csv_async::AsyncDeserializer;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_stream::StreamExt as TokioStreamExt;
use url::ParseError;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteData {
    rank: i32,
    root_domain: String,
}

pub struct FileReader {
    // Path to the file to read
    pub file: File,

    // `url_sender` is a mpsc channel sender that sends a URL to the site pool.
    pub url_sender: crossbeam_channel::Sender<url::Url>,
}

impl FileReader {
    pub async fn new(path_buf: PathBuf, url_sender: crossbeam_channel::Sender<url::Url>) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path_buf).await?;
        Ok(Self {
            file,
            url_sender,
        })
    }

    pub async fn start(self) {
        // Create an AsyncDeserializer from the file
        let mut rdr = AsyncDeserializer::from_reader(self.file);

        // Create a stream from the deserializer
        let mut records = rdr.deserialize::<WebsiteData>();
        // Use StreamExt to asynchronously process each record
        while let Some(result) = records.next().await {
            match result {
                Ok(data) => {
                   let url =  match url::Url::parse(&data.root_domain) {
                        Ok(url) => {
                            url
                        }
                       Err(ParseError::RelativeUrlWithoutBase) => {
                           let url = format!("https://www.{}", data.root_domain);
                            match url::Url::parse(&url) {
                                 Ok(url) => url,
                                 Err(e) => {
                                      eprintln!("Error parsing URL: {:?}", e);
                                      continue;
                                 }
                            }
                       }
                        Err(e) => {
                            eprintln!("Error parsing URL: {:?}", e);
                            continue;
                        }
                    };
                    // Send the URL to the site pool.
                    match self.url_sender.send(url) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error sending URL to site pool: {:?}", e);
                        }
                    }
                }
                Err(e) => eprintln!("Error reading CSV: {}", e),
            }
        };
    }
}