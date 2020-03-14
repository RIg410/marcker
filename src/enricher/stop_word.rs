use crate::dictionary::Dictionary;
use crate::{Enricher, Meta, Sentence, Loc};
use anyhow::Error;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::{Arc, RwLock};

const STOP_WORD: &str = "stop_word";

#[derive(Clone)]
pub struct StopWordEnricher<D> where D: Dictionary {
    dictionary: Arc<RwLock<D>>,
    brake_on_stop_word: bool,
}

impl<D> StopWordEnricher<D> where D: Dictionary {
    pub fn new(dictionary: D, brake_on_stop_word: bool) -> StopWordEnricher<D> {
        StopWordEnricher {
            dictionary: Arc::new(RwLock::new(dictionary)),
            brake_on_stop_word,
        }
    }

    pub fn extract(sentence: &Sentence) -> Vec<(Loc, String)> {
        let mut locs = vec![];

        for index in 0..sentence.tokens().len() {
            let token = &sentence.tokens()[index];
            if token.get_meta(STOP_WORD).is_some() {
                locs.push((Loc::new(index, 1), sentence.by_loc(&token.loc)));
            }
        }

        locs
    }

    pub fn sync(&self) -> Result<(), Error> {
        self.dictionary.write().unwrap().sync()
    }
}

impl<D> Enricher for StopWordEnricher<D> where D: Dictionary {
    fn enrich(&self, sentence: &mut Sentence) -> Result<(), Error> {
        let dictionary = self.dictionary.read().unwrap();
        for index in 0..sentence.tokens().len() {
            let is_stop_word = {
                let token = &sentence.tokens()[index];
                if dictionary.contains(&token.val) {
                    if self.brake_on_stop_word {
                        return Err(Error::new(StopWordError {
                            stop_word: sentence.by_loc(&token.loc),
                        }));
                    } else {
                        true
                    }
                } else {
                    false
                }
            };

            if is_stop_word {
                let token = &mut sentence.tokens_mut()[index];
                token.add_meta(STOP_WORD, Meta::Bool(true));
            }
        }

        Ok(())
    }

    fn update(&self, sentence: &Sentence) -> Result<(), Error> {
        let mut dictionary = self.dictionary.write().unwrap();
        for index in 0..sentence.tokens().len() {
            let token = &sentence.tokens()[index];
            if token.get_meta(STOP_WORD).is_some() {
                dictionary.add(token.val.to_owned())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StopWordError {
    pub stop_word: String,
}

impl StdError for StopWordError {
    fn description(&self) -> &str {
        "Stop word"
    }
}

impl Display for StopWordError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "stop word:[{}]", self.stop_word)
    }
}

#[cfg(test)]
mod test {
    use crate::{Service, StopWordEnricher, Loc, Meta};
    use rust_stemmers::Algorithm;
    use crate::dictionary::{LocalDictionary, Empty};
    use super::STOP_WORD;


    #[test]
    fn test_stop_words_break_pipeline() {
        let mut service: Service = Service::new(Algorithm::Russian);
        let dictionary: LocalDictionary<Empty> = LocalDictionary::new(vec!["w", "s", "f", "g"].into_iter()
            .map(|v| v.to_owned())
            .collect());

        service.add_enricher(StopWordEnricher::new(dictionary, true));
        assert!(service.sentence("У меня 120 w печеник".to_owned()).is_err());
    }

    #[test]
    fn test_stop_words() {
        let mut service: Service = Service::new(Algorithm::Russian);
        let dictionary: LocalDictionary<Empty> = LocalDictionary::new(vec!["w", "s", "f", "g"].into_iter()
            .map(|v| v.to_owned())
            .collect());

        service.add_enricher(StopWordEnricher::new(dictionary, false));
        let sentence = service.sentence("У g меня 120 w печеник s кукиу f".to_owned()).unwrap();
        assert_eq!(
            StopWordEnricher::<Empty>::extract(&sentence),
            vec![
                (Loc::new(1, 1), "g".to_owned()),
                (Loc::new(4, 1), "w".to_owned()),
                (Loc::new(6, 1), "s".to_owned()),
                (Loc::new(8, 1), "f".to_owned()),
            ]);
    }

    #[test]
    fn test_add_stop_words() {
        let mut service: Service = Service::new(Algorithm::Russian);
        service.add_enricher(StopWordEnricher::new(LocalDictionary::<Empty>::new(vec![]), false));
        let mut sentence = service.sentence("У g меня 120 w печеник s кукиу f".to_owned()).unwrap();
        assert_eq!(StopWordEnricher::<Empty>::extract(&sentence), vec![]);
        let tokens = sentence.tokens_mut();
        tokens[1].add_meta(STOP_WORD, Meta::Bool(true));
        tokens[4].add_meta(STOP_WORD, Meta::Bool(true));
        tokens[6].add_meta(STOP_WORD, Meta::Bool(true));
        tokens[8].add_meta(STOP_WORD, Meta::Bool(true));
        service.back_propagation(sentence).unwrap();

        let mut sentence = service.sentence("У g меня 120 w печеник s кукиу f".to_owned()).unwrap();
        assert_eq!(
            StopWordEnricher::<Empty>::extract(&sentence),
            vec![
                (Loc::new(1, 1), "g".to_owned()),
                (Loc::new(4, 1), "w".to_owned()),
                (Loc::new(6, 1), "s".to_owned()),
                (Loc::new(8, 1), "f".to_owned()),
            ]);
    }
}