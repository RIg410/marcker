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

    pub fn add_enricher<E>(&mut self, enricher: E) where E: Enricher + 'static {
        if let Some(pipe) = Arc::get_mut(&mut self.pipeline) {
            pipe.push(Box::new(enricher));
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

    pub fn back_propagation(&self, sentence: Sentence) -> Result<(), Error> {
        for enricher in self.pipeline.as_ref() {
            enricher.update(&sentence)?;
        }

        Ok(())
    }

    fn tokenize(&self, msg: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut builder: Option<(TokenBuilder, bool)> = None;

        for (index, symbol) in msg.to_lowercase().chars().enumerate() {
            if symbol.is_ascii_whitespace() {
                if let Some(builder) = builder.take() {
                    tokens.push(builder.0.done(&self.stemmer));
                }
            } else if symbol.is_ascii_punctuation() {
                if let Some(builder) = builder.take() {
                    tokens.push(builder.0.done(&self.stemmer));
                }
                let mut builder = TokenBuilder::new(index);
                builder.add_symbol(symbol);
                tokens.push(builder.done(&self.stemmer));
            } else {
                let is_numeric = symbol.is_numeric();
                if builder.is_none() {
                    builder = Some((TokenBuilder::new(index), is_numeric));
                }

                if builder.as_ref().unwrap().1 == is_numeric {
                    let builder_val = builder.as_mut().unwrap();
                    builder_val.0.add_symbol(symbol);
                } else {
                    let builder_val = builder.take().unwrap();
                    tokens.push(builder_val.0.done(&self.stemmer));
                    let mut builder_val = TokenBuilder::new(index);
                    builder_val.add_symbol(symbol);
                    builder = Some((builder_val, is_numeric));
                }
            }
        }

        if let Some(builder) = builder.take() {
            tokens.push(builder.0.done(&self.stemmer));
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

#[cfg(test)]
mod test {
    use crate::{Loc, Service, Token};
    use rust_stemmers::Algorithm;

    #[test]
    fn test_tokenization() {
        let service: Service = Service::new(Algorithm::Russian);
        assert_eq!(
            service
                .sentence("w1w-12wr$re;21-12feац-ц123".to_owned())
                .unwrap()
                .tokens(),
            [
                Token::new("w".to_owned(), Loc::new(0, 1)),
                Token::new("1".to_owned(), Loc::new(1, 1)),
                Token::new("w".to_owned(), Loc::new(2, 1)),
                Token::new("-".to_owned(), Loc::new(3, 1)),
                Token::new("12".to_owned(), Loc::new(4, 2)),
                Token::new("wr".to_owned(), Loc::new(6, 2)),
                Token::new("$".to_owned(), Loc::new(8, 1)),
                Token::new("re".to_owned(), Loc::new(9, 2)),
                Token::new(";".to_owned(), Loc::new(11, 1)),
                Token::new("21".to_owned(), Loc::new(12, 2)),
                Token::new("-".to_owned(), Loc::new(14, 1)),
                Token::new("12".to_owned(), Loc::new(15, 2)),
                Token::new("feац".to_owned(), Loc::new(17, 4)),
                Token::new("-".to_owned(), Loc::new(21, 1)),
                Token::new("ц".to_owned(), Loc::new(22, 1)),
                Token::new("123".to_owned(), Loc::new(23, 3)),
            ]
        );
    }
}
