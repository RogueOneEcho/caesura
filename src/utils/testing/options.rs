use crate::options::*;

pub struct TestOptionsFactory;

impl TestOptionsFactory {
    #[must_use]
    pub fn from<T: Options>(mut options: T) -> T {
        let provider = OptionsProvider::new();
        options.merge(&provider.get());
        options
    }
}
