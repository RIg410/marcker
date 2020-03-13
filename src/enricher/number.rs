use crate::{Enricher, Loc, Meta, Sentence};
use anyhow::Error;
use std::str::FromStr;

const SIGN_INDEX: &str = "number.sign_index";
const NUMBER_INDEX: &str = "number.number_index";
const VALUE: &str = "number.number";

pub struct NumberEnricher();

impl NumberEnricher {
    pub fn extract(sentence: &Sentence) -> Vec<(Loc, Number)> {
        let mut numbers = vec![];

        for index in 0..sentence.tokens().len() {
            let token = &sentence.tokens()[index];
            if let Some(value) = token.get_meta(VALUE) {
                if let Some(sign) = token.get_meta(SIGN_INDEX) {
                    numbers.push((Loc::new(sign.as_usize(), 2), Number::Signed(value.as_i64())));
                } else {
                    numbers.push((Loc::new(index, 1), Number::Unsigned(value.as_u64())));
                }
            }
        }

        numbers
    }
}

impl Enricher for NumberEnricher {
    fn enrich(&self, sentence: &mut Sentence) -> Result<(), Error> {
        for index in 0..sentence.tokens().len() {
            let number = {
                let token = &sentence.tokens()[index];
                if let Ok(val) = u64::from_str(&token.val) {
                    if index > 0 {
                        let token = &sentence.tokens()[index - 1];
                        if &token.val == "-" || &token.val == "минус" {
                            Some(Number::Signed((val as i64) * -1))
                        } else {
                            Some(Number::Unsigned(val))
                        }
                    } else {
                        Some(Number::Unsigned(val))
                    }
                } else {
                    None
                }
            };

            if let Some(number) = number {
                match number {
                    Number::Signed(number) => {
                        let token = &mut sentence.tokens_mut()[index];
                        token.add_meta(VALUE, Meta::I64(number));
                        token.add_meta(SIGN_INDEX, Meta::Usize(index - 1));
                        let token = &mut sentence.tokens_mut()[index - 1];
                        token.add_meta(NUMBER_INDEX, Meta::Usize(index));
                    }
                    Number::Unsigned(number) => {
                        let token = &mut sentence.tokens_mut()[index];
                        token.add_meta(VALUE, Meta::U64(number));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Number {
    Signed(i64),
    Unsigned(u64),
}

#[cfg(test)]
mod test {
    use crate::enricher::number::{Number, NumberEnricher};
    use crate::{Loc, Service};
    use rust_stemmers::Algorithm;

    #[test]
    fn test_unsigned_number() {
        let mut service: Service = Service::new(Algorithm::Russian);
        service.add_enricher(Box::new(NumberEnricher()));
        let sentence = service.sentence("У меня 120 печеник".to_owned()).unwrap();
        let numbers = NumberEnricher::extract(&sentence);
        assert_eq!(numbers, vec![(Loc::new(2, 1), Number::Unsigned(120))]);
    }

    #[test]
    fn test_signed_number() {
        let mut service: Service = Service::new(Algorithm::Russian);
        service.add_enricher(Box::new(NumberEnricher()));
        let sentence = service.sentence("У меня -120 печеник".to_owned()).unwrap();
        let numbers = NumberEnricher::extract(&sentence);
        assert_eq!(numbers, vec![(Loc::new(2, 2), Number::Signed(-120))]);
    }

    #[test]
    fn test_number() {
        let mut service: Service = Service::new(Algorithm::Russian);
        service.add_enricher(Box::new(NumberEnricher()));
        let sentence = service.sentence("У меня 120 печеник и 30 котов. И - 50 и -90. 140% 40 % 70".to_owned()).unwrap();
        let numbers = NumberEnricher::extract(&sentence);
        assert_eq!(numbers, vec![
            (Loc::new(2, 1), Number::Unsigned(120)),
            (Loc::new(5, 1), Number::Unsigned(30)),
            (Loc::new(9, 2), Number::Signed(-50)),
            (Loc::new(12, 2), Number::Signed(-90)),
            (Loc::new(15, 1), Number::Unsigned(140)),
            (Loc::new(17, 1), Number::Unsigned(40)),
            (Loc::new(19, 1), Number::Unsigned(70)),
        ]);
    }
}
