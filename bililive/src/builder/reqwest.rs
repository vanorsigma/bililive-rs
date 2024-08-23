use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use url::Url;

use crate::core::builder::Requester;

use super::BoxedError;

#[derive(Debug, Default)]
pub struct ReqwestClient(Client);

impl From<Client> for ReqwestClient {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

fn hashmap_to_cookies_str(cookies: HashMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

impl Requester for ReqwestClient {
    fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<T, BoxedError>> + Send + '_>> {
        let url = Url::from_str(url).unwrap();
        Box::pin(async move {
            Ok(serde_json::from_slice(
                &*self.0.get(url).send().await?.bytes().await?,
            )?)
        })
    }

    fn get_json_with_parameters<T: DeserializeOwned>(
        &self,
        url: &str,
        parameters: HashMap<String, String>,
        cookies: HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<T, BoxedError>> + Send + '_>> {
        let url = Url::from_str(url).unwrap();
        Box::pin(async move {
            Ok(serde_json::from_slice(
                &*self
                    .0
                    .get(url)
                    .query(&parameters)
                    .header("Cookie", hashmap_to_cookies_str(cookies))
                    .send()
                    .await?
                    .bytes()
                    .await?,
            )?)
        })
    }

    fn get_cookie(
        &self,
        url: String,
        cookie_name: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, BoxedError>> + '_>> {
        let url = Url::from_str(url.as_str()).unwrap();
        Box::pin(async move {
            Ok(self
                .0
                .get(url.as_str())
                .send()
                .await?
                .cookies()
                .into_iter()
                .find(|cookie| cookie.name() == cookie_name)
                .map_or("".to_string(), |x| x.value().to_string()))
        })
    }
}
