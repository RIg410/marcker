use anyhow::Error;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::hash_map::RandomState;

pub trait Dictionary where Self: Debug + Clone {
    fn contains(&self, word: &str) -> bool;
    fn add(&mut self, word: String) -> Result<bool, Error>;
    fn sync(&mut self) -> Result<(), Error>;
    fn get_all(&self) -> Result<HashSet<String>, Error>;
}

#[derive(Debug, Clone)]
pub struct LocalDictionary<D> where D: Dictionary {
    dictionary: Arc<HashSet<String>>,
    inner: Option<D>,
}

impl<D> LocalDictionary<D> where D: Dictionary {
    pub fn new(data: Vec<String>) -> LocalDictionary<D> {
        LocalDictionary {
            dictionary: Arc::new(data.into_iter().collect()),
            inner: None,
        }
    }

    pub fn with_inner(inner: D) -> LocalDictionary<D> {
        LocalDictionary {
            dictionary: Default::default(),
            inner: Some(inner),
        }
    }
}

impl<D> Dictionary for LocalDictionary<D> where D: Dictionary {
    fn contains(&self, word: &str) -> bool {
        self.dictionary.contains(word)
    }

    fn add(&mut self, word: String) -> Result<bool, Error> {
        let dictionary = Arc::get_mut(&mut self.dictionary)
            .ok_or_else(|| Error::msg("Unexpected error."))?;
        if dictionary.insert(word.clone()) {
            if let Some(inner) = self.inner.as_mut() {
                inner.add(word)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn sync(&mut self) -> Result<(), Error> {
        if let Some(inner) = self.inner.as_ref() {
            self.dictionary = Arc::new(inner.get_all()?);
        }
        Ok(())
    }

    fn get_all(&self) -> Result<HashSet<String>, Error> {
        Ok(self.dictionary.as_ref().clone())
    }
}

#[derive(Clone, Debug)]
pub struct Empty();

impl Dictionary for Empty {
    fn contains(&self, _word: &str) -> bool {
        false
    }

    fn add(&mut self, _word: String) -> Result<bool, Error> {
        Ok(false)
    }

    fn sync(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn get_all(&self) -> Result<HashSet<String, RandomState>, Error> {
        Ok(Default::default())
    }
}