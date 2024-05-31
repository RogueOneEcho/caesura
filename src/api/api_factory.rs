use std::time::Duration;

use di::{injectable, Ref};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, ClientBuilder};

use crate::api::Api;
use crate::built_info;
use crate::options::SharedOptions;

/// The number of requests allowed per duration
const ALLOWED_REQUESTS_PER_DURATION: u64 = 10;
const REQUEST_LIMIT_DURATION: Duration = Duration::from_secs(10);

/// Create an [API]
pub struct ApiFactory {
    api_key: String,
    api_url: String,
}

#[injectable]
impl ApiFactory {
    /// Create a new [`ApiFactory`]
    #[must_use]
    pub fn new(options: Ref<SharedOptions>) -> Self {
        Self {
            api_key: options.api_key.clone().expect("Options should be set"),
            api_url: options.indexer_url.clone().expect("Options should be set"),
        }
    }

    #[must_use]
    pub fn create(&self) -> Api {
        let client = self.create_client();
        let rate_limited_client = tower::ServiceBuilder::new()
            .rate_limit(ALLOWED_REQUESTS_PER_DURATION, REQUEST_LIMIT_DURATION)
            .service(client);
        Api {
            api_url: self.api_url.clone(),
            client: rate_limited_client,
        }
    }

    fn create_client(&self) -> Client {
        ClientBuilder::new()
            .default_headers(self.get_headers())
            .build()
            .expect("Client builder should not fail")
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, self.get_user_agent());
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(header::AUTHORIZATION, self.get_authorization());
        headers
    }

    fn get_user_agent(&self) -> HeaderValue {
        let user_agent = format!(
            "{}/{} ({})",
            built_info::PKG_NAME,
            built_info::PKG_VERSION,
            built_info::PKG_HOMEPAGE
        );
        HeaderValue::try_from(user_agent).expect("User agent header should not fail")
    }

    fn get_authorization(&self) -> HeaderValue {
        let mut value = HeaderValue::try_from(self.api_key.clone())
            .expect("Authorization header should not fail");
        value.set_sensitive(true);
        value
    }
}
