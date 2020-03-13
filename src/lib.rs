mod enricher;
mod sentence;
mod service;

pub use enricher::*;
pub use sentence::*;
pub use service::*;

#[cfg(test)]
mod test {
    use crate::Service;
    use rust_stemmers::Algorithm;

    #[test]
    fn test() {
        let service: Service = Service::new(Algorithm::Russian);
        let sentence = service
            .sentence("Приветик, как твои дела? Я хочу купить слона. % ".to_owned())
            .unwrap();
        dbg!(sentence);
    }
}
