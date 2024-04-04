use std::collections::HashMap;
use rust_numerals::number_to_cardinal;
use scraper::Selector;
use spider::page::Page;

pub struct PageParser {
    // `page_rx` is a mpsc channel receiver that receives a page from the page pool.
    page_rx: crossbeam_channel::Receiver<Page>,
    // `text_sender` is a mpsc channel sender that sends a vector of processed texts to the text pool.
    text_tx: crossbeam_channel::Sender<(Page, Vec<String>)>,
    // `lemmatizer_map` is a hashmap that stores the lemmatized words.
    lemmatizer_map: HashMap<String, String>,
    // `stemmer` is a stemmer instance.
    stemmer: rust_stemmers::Stemmer,
    // `stop_words` is a list of stopwords.
    stop_words: Vec<String>,
}

impl PageParser {
    /// Create a new PageParser instance.
    pub fn new(page_rx: crossbeam_channel::Receiver<Page>, text_tx: crossbeam_channel::Sender<(Page, Vec<String>)>, lemmatizer_json_path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
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
            text_tx,
            lemmatizer_map: map,
            stemmer: rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English),
            stop_words: stop_words::get(stop_words::LANGUAGE::English),
        })
    }

    /// Start the page parser in background.
    pub async fn start(self) {
        // Loop to receive pages from the page receiver.
        while let Ok(page) = self.page_rx.recv() {
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
            match self.text_tx.send((page, texts)) {
                Ok(_) => (),
                Err(e) => eprintln!("Error sending texts to text pool: {:?}", e),
            }
        }
    }

    /// Preprocess the text.
    fn preprocess_text(&self, texts: Vec<String>) -> Vec<String> {
        // lower
        let texts = Self::parse_lower(texts);
        let texts = texts.iter()
            // split by whitespace
            .flat_map(|text| text.split_whitespace().map(str::to_string))
            // remove punctuation
            .map(|text| Self::remove_punctuation(&text))
            // remove apostrophes
            .map(|text| Self::remove_apostrophes(&text))
            // remove single characters
            .filter(|text| text.len() > 1)
            // remove length more than 50
            .filter(|text| text.len() < 50)
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

#[cfg(test)]
mod test{
    use std::path::PathBuf;
    use crossbeam_channel::unbounded;
    use super::*;
    fn get_page_parser() -> Result<PageParser, Box<dyn std::error::Error>> {
        let (_, page_receiver) = unbounded();
        let (text_sender, _) = unbounded();
        let lemmatizer_json_path = PathBuf::from("assets/lemmatizedMap.json");
        PageParser::new(page_receiver, text_sender, lemmatizer_json_path)
    } 
    
    // Remove punctuation
    #[test]
    fn can_remove_punctuation() {
        let text = "Hello, World!";
        let text = PageParser::remove_punctuation(text);
        assert_eq!(text, "Hello World");
    }
    // Remove apostrophes
    #[test]
    fn can_remove_apostrophes() {
        let text = "Hello's World";
        let text = PageParser::remove_apostrophes(text);
        assert_eq!(text, "Hellos World");
    }
    
    // Convert Numbers to Words
    #[test]
    fn can_convert_numbers_to_words() {
        let text = "123";
        let text = PageParser::convert_numbers_to_words(text);
        assert_eq!(text, "one hundred and twenty-three");
    }
    // Remove stopwords
    #[test]
    fn can_remove_stopwords() {
        let page_parser = get_page_parser().unwrap();
        let texts = vec!["the".to_string(), "quick".to_string(), "brown".to_string(), "fox".to_string()];
        let texts = page_parser.remove_stopwords(texts);
        assert_eq!(texts, vec!["quick".to_string(), "brown".to_string(), "fox".to_string()]);
    }
    
    // Stem the word
    #[test]
    fn can_stem_word() {
        let page_parser = get_page_parser().unwrap();
        let word = "running";
        let word = page_parser.stem_word(word);
        assert_eq!(word, "run");
    }
    // Lemmatize the word
    #[test]
    fn can_lemmatize_word() {
        let page_parser = get_page_parser().unwrap();
        let word = "running";
        let word = page_parser.lemmatize_word(word);
        assert_eq!(word, "run");
    }
    
    // Parse lower
    #[test]
    fn can_parse_lower() {
        let texts = vec!["Hello".to_string(), "World".to_string()];
        let texts = PageParser::parse_lower(texts);
        assert_eq!(texts, vec!["hello".to_string(), "world".to_string()]);
    }
    #[tokio::test]
    async fn can_preprocess_text () -> Result<(), Box<dyn std::error::Error>> {
        let texts = vec!["Hello, World!".to_string(),
                            "The quick brown fox jumps over the lazy dog.".to_string(),
                            "123".to_string(),
                            "running".to_string(),
                            "the".to_string(),
                            "quick".to_string(),
                            "brown".to_string(),
                            "fox".to_string(),
                            "jumps".to_string(),
                            "over".to_string()
        ];
        let page_parser = get_page_parser()?;
        let texts = page_parser.preprocess_text(texts);
        assert_eq!(texts, vec!["quick".to_string(), "brown".to_string(), "fox".to_string(), "jump".to_string(), "lazi".to_string(), "dog".to_string(), "one hundred and twentythre".to_string(), "run".to_string(), "quick".to_string(), "brown".to_string(), "fox".to_string(), "jump".to_string()]);
        Ok(())
    }
}