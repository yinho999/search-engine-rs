use std::collections::HashMap;
use rust_numerals::number_to_cardinal;
use scraper::Selector;
use spider::page::Page;

pub struct PageParser {
    // `page_rx` is a mpsc channel receiver that receives a page from the page pool.
    page_rx: tokio::sync::mpsc::Receiver<Page>,
    // `site_sender` is a mpsc channel sender that sends a URL to the site pool.
    site_sender: tokio::sync::mpsc::Sender<url::Url>,
    // `db` is a postgres connection pool.
    db: sqlx::PgPool,
    // `lemmatizer_map` is a hashmap that stores the lemmatized words.
    lemmatizer_map: HashMap<String, String>,
    // `stemmer` is a stemmer instance.
    stemmer: rust_stemmers::Stemmer,
    // `stop_words` is a list of stopwords.
    stop_words: Vec<String>,
}

impl PageParser {
    /// Create a new PageParser instance.
    pub fn new(page_rx: tokio::sync::mpsc::Receiver<Page>, site_sender: tokio::sync::mpsc::Sender<url::Url>, db: sqlx::PgPool, lemmatizer_json_path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let lemmatizer_json = std::fs::read_to_string(lemmatizer_json_path)?;
        let mut lemmatizer_json: HashMap<String, String> = serde_json::from_str(&lemmatizer_json)?;
        let mut map = HashMap::new();
        let keys: Vec<String> = lemmatizer_json.keys().cloned().collect();
        for key in keys.iter() {
            let (k, v) = match lemmatizer_json.remove_entry(key) {
                Some((k, v)) => (k, v),
                None => continue,
            };
            map.insert(k, v);
        }
        Ok(Self {
            page_rx,
            site_sender,
            db,
            lemmatizer_map: map,
            stemmer: rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English),
            stop_words: stop_words::get(stop_words::LANGUAGE::English),
        })
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
            let texts = self.preprocess_text(texts);
        }
    }

    /// Preprocess the text.
    fn preprocess_text(&self, texts: Vec<String>) -> Vec<String> {
        // lower
        let texts = Self::parse_lower(texts);
        let texts = texts.iter()
            // remove punctuation
            .map(|text| Self::remove_punctuation(text))
            // remove apostrophes
            .map(|text| Self::remove_apostrophes(&text))
            // remove single characters
            .map(|text| Self::remove_single_chars(&text))
            // convert numbers to words
            .map(|text| Self::convert_numbers_to_words(&text))
            .collect();
        // Remove stopwords
        let texts = self.remove_stopwords(texts);
        let texts = texts.iter()
            // Stem the words
            .map(|text| self.stem_word(text))
            // Lemmatize the words
            .map(|text| self.lemmatize_word(&text))
            // remove punctuation again
            .map(|text| Self::remove_punctuation(&text))
            // convert numbers to words again
            .map(|text| Self::convert_numbers_to_words(&text))
            .collect();
        texts
    }


    /// Parse all the words in the text and cast them to lowercase.
    fn parse_lower(texts: Vec<String>) -> Vec<String> {
        texts.iter()
            .map(|text| text.to_lowercase())
            .collect()
    }

    /// Punctuation marks to be removed from the text.
    fn remove_punctuation(text: &str) -> String {
        text.chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect()
    }

    /// Remove apostrophes from the text.
    fn remove_apostrophes(text: &str) -> String {
        text.chars()
            .filter(|c| *c != '\'')
            .collect()
    }

    /// Remove single characters from the text.
    fn remove_single_chars(text: &str) -> String {
        text.chars()
            .filter(|c| c.len_utf8() > 1)
            .collect()
    }

    /// Convert Numbers to Words
    fn convert_numbers_to_words(text: &str) -> String {
        // try to convert the number to a word
        let num = text.parse::<i64>();
        match num {
            Ok(num) => {
                number_to_cardinal(num)
            }
            Err(_) => text.to_string(),
        }
    }

    /// Remove stopwords from the text.
    fn remove_stopwords(&self, texts: Vec<String>) -> Vec<String> {
        texts.iter()
            .filter(|text| !self.stop_words.contains(text))
            .map(|text| text.clone())
            .collect()
    }

    /// Stem the word in the text.
    fn stem_word(&self, word: &str) -> String {
        self.stemmer.stem(word).to_string()
    }

    /// Lemmatize the word in the text.
    fn lemmatize_word(&self, word: &str) -> String {
        match self.lemmatizer_map.get(word) {
            Some(lemma) => lemma.clone(),
            None => word.to_string(),
        }
    }
}