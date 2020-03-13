use crate::{Loc, Sentence, Token};
use anyhow::Error;
use rust_stemmers::{Algorithm, Stemmer};
use std::sync::Arc;

#[derive(Clone)]
pub struct Service {
    stemmer: Arc<Stemmer>,
    pipeline: Arc<Vec<Box<dyn Enricher>>>,
}

impl Service {
    pub fn new(stemmer_algo: Algorithm) -> Service {
        Service {
            stemmer: Arc::new(Stemmer::create(stemmer_algo)),
            pipeline: Arc::new(vec![]),
        }
    }

    pub fn add_enricher(&mut self, enricher: Box<dyn Enricher>) {
        if let Some(pipe) = Arc::get_mut(&mut self.pipeline) {
            pipe.push(enricher);
        }
    }

    pub fn sentence(&self, msg: String) -> Result<Sentence, Error> {
        let tokens = self.tokenize(&msg);
        let mut sentence = Sentence::new(msg, tokens);

        for enricher in self.pipeline.as_ref() {
            enricher.enrich(&mut sentence)?
        }

        Ok(sentence)
    }

    pub fn update_enricher(&self, sentence: Sentence) -> Result<(), Error> {
        for enricher in self.pipeline.as_ref() {
            enricher.update(&sentence)?;
        }

        Ok(())
    }

    fn tokenize(&self, msg: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut builder: Option<TokenBuilder> = None;
        let number = None;

        for (index, symbol) in msg.to_lowercase().chars().enumerate() {
            if symbol.is_ascii_whitespace() {
                if let Some(builder) = builder.take() {
                    tokens.push(builder.done(&self.stemmer));
                }
            } else if symbol.is_ascii_punctuation() {
                if let Some(builder) = builder.take() {
                    tokens.push(builder.done(&self.stemmer));
                }
                let mut builder = TokenBuilder::new(index);
                builder.add_symbol(symbol);
                tokens.push(builder.done(&self.stemmer));
            } else {
                if builder.is_none() {
                    builder = Some(TokenBuilder::new(index));
                }
                let builder = builder.as_mut().unwrap();
                builder.add_symbol(symbol);
            }
        }

        if let Some(builder) = builder.take() {
            tokens.push(builder.done(&self.stemmer));
        }

        tokens
    }
}

pub trait Enricher {
    fn enrich(&self, sentence: &mut Sentence) -> Result<(), Error>;
    fn update(&self, _sentence: &Sentence) -> Result<(), Error> {
        Ok(())
    }
}

struct TokenBuilder {
    pub token: String,
    pub start_index: usize,
    pub len: usize,
}

impl TokenBuilder {
    pub fn new(start_index: usize) -> TokenBuilder {
        TokenBuilder {
            token: "".to_string(),
            start_index,
            len: 0,
        }
    }

    pub fn add_symbol(&mut self, symbol: char) {
        self.token.push(symbol);
        self.len += 1;
    }

    pub fn done(self, stemmer: &Stemmer) -> Token {
        Token::new(
            stemmer.stem(&self.token).to_string(),
            Loc::new(self.start_index, self.len),
        )
    }
}
