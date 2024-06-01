use colored::Colorize;
use di::{injectable, Ref};
use log::*;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use tower::limit::RateLimit;
use tower::ServiceExt;

use crate::api::ApiFactory;
use crate::api::{ApiResponse, TorrentGroupResponse, TorrentResponse};
use crate::errors::AppError;

/// API client
///
/// Created by an [`ApiFactory`]
pub struct Api {
    pub api_url: String,
    pub client: RateLimit<Client>,
}

#[injectable]
impl Api {
    #[must_use]
    pub fn new(factory: Ref<ApiFactory>) -> Self {
        factory.create()
    }

    /// Get a torrent by id
    ///
    /// A torrent is a specific encoding of a release (album, EP, single, etc.).
    ///
    /// # See Also
    /// - <https://github.com/OPSnet/Gazelle/blob/master/docs/07-API.md#torrent>
    pub async fn get_torrent(&mut self, id: i64) -> Result<TorrentResponse, AppError> {
        let action = "get torrent";
        let url = format!("{}/ajax.php?action=torrent&id={}", self.api_url, id);
        let response = self.get(&url, action).await?;
        self.deserialize(response, action).await
    }

    /// Get a torrent group by id
    ///
    /// A torrent group is a collection of different encodings of
    /// a release (album, EP, single, etc.).
    ///
    /// # See Also
    /// - <https://github.com/OPSnet/Gazelle/blob/master/docs/07-API.md#torrent-group>
    pub async fn get_torrent_group(&mut self, id: i64) -> Result<TorrentGroupResponse, AppError> {
        let action = "get torrent group";
        let url = format!("{}/ajax.php?action=torrentgroup&id={}", self.api_url, id);
        let response = self.get(&url, action).await?;
        self.deserialize(response, action).await
    }

    /// Get the content of the .torrent file as a buffer
    ///
    /// # See Also
    /// - <https://github.com/OPSnet/Gazelle/blob/master/docs/07-API.md#download>
    pub async fn get_torrent_file_as_buffer(&mut self, id: i64) -> Result<Vec<u8>, AppError> {
        let url = format!("{}/ajax.php?action=download&id={}", self.api_url, id);
        let response = self.get(&url, "get torrent file").await?;
        let bytes = response
            .bytes()
            .await
            .expect("Response should not be empty");
        let buffer = bytes.to_vec();
        Ok(buffer)
    }

    async fn get(&mut self, url: &String, action: &str) -> Result<Response, AppError> {
        let result = self.wait_for_client().await.get(url).send().await;
        trace!("{} GET request: {}", "Sent".bold(), &url);
        let response = result.or_else(|e| AppError::request(e, action))?;
        let status_code = response.status();
        if status_code.is_success() {
            Ok(response)
        } else {
            AppError::response(status_code, action)
        }
    }

    async fn wait_for_client(&mut self) -> &Client {
        self.client
            .ready()
            .await
            .expect("client should be available")
            .get_ref()
    }

    async fn deserialize<TResponse: DeserializeOwned>(
        &self,
        response: Response,
        action: &str,
    ) -> Result<TResponse, AppError> {
        let response = response
            .json::<ApiResponse<TResponse>>()
            .await
            .or_else(|e| AppError::request(e, action))?;
        if response.status != "success" {
            AppError::explained(action, "API returned a non-success response".to_owned())
        } else {
            Ok(response.response.expect("response should be set"))
        }
    }
}
