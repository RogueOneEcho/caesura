use colored::Colorize;
use di::{injectable, Ref};
use log::*;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use tower::limit::RateLimit;
use tower::ServiceExt;

use crate::api::ApiError::*;
use crate::api::{ApiError, ApiFactory};
use crate::api::{ApiResponse, TorrentGroupResponse, TorrentResponse};

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
    pub async fn get_torrent(&mut self, id: i64) -> Result<TorrentResponse, ApiError> {
        let url = format!("{}/ajax.php?action=torrent&id={}", self.api_url, id);
        let response = self.get(&url).await?;
        self.deserialize(response, url, TorrentNotFound).await
    }

    /// Get a torrent group by id
    ///
    /// A torrent group is a collection of different encodings of
    /// a release (album, EP, single, etc.).
    ///
    /// # See Also
    /// - <https://github.com/OPSnet/Gazelle/blob/master/docs/07-API.md#torrent-group>
    pub async fn get_torrent_group(&mut self, id: i64) -> Result<TorrentGroupResponse, ApiError> {
        let url = format!("{}/ajax.php?action=torrentgroup&id={}", self.api_url, id);
        let response = self.get(&url).await?;
        self.deserialize(response, url, GroupNotFound).await
    }

    /// Get the content of the .torrent file as a buffer
    ///
    /// # See Also
    /// - <https://github.com/OPSnet/Gazelle/blob/master/docs/07-API.md#download>
    pub async fn get_torrent_file_as_buffer(&mut self, id: i64) -> Result<Vec<u8>, ApiError> {
        let url = format!("{}/ajax.php?action=download&id={}", self.api_url, id);
        let response = self.get(&url).await?;
        let bytes = response
            .bytes()
            .await
            .expect("Response should not be empty");
        let buffer = bytes.to_vec();
        Ok(buffer)
    }

    async fn get(&mut self, url: &String) -> Result<Response, ApiError> {
        let result = self.wait_for_client().await?.get(url).send().await;
        trace!("{} GET request: {}", "Sent".bold(), &url);
        match result {
            Ok(response) => Ok(response),
            Err(error) => Err(RequestFailure(url.clone(), error)),
        }
    }

    async fn wait_for_client(&mut self) -> Result<&Client, ApiError> {
        match self.client.ready().await {
            Ok(client) => Ok(client.get_ref()),
            Err(error) => Err(ClientFailure(error)),
        }
    }

    async fn deserialize<TResponse: DeserializeOwned>(
        &self,
        response: Response,
        url: String,
        non_success_error: ApiError,
    ) -> Result<TResponse, ApiError> {
        let response = match response.json::<ApiResponse<TResponse>>().await {
            Ok(response) => response,
            Err(error) => return Err(DeserializationFailure(url, error)),
        };
        if response.status != "success" {
            return Err(non_success_error);
        }
        match response.response {
            Some(response) => Ok(response),
            None => Err(EmptyResponse),
        }
    }
}
