use std::{any::type_name, borrow::Borrow, collections::HashMap, marker::PhantomData};

use anyhow::{anyhow, Result};
use log::{debug, error};
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str, to_string};

use crate::client::{Method, Response, API};

pub struct Request<In: Serialize, Out: DeserializeOwned> {
    pub name: &'static str,
    _p:       PhantomData<fn(In) -> Out>,
}

impl<In: Serialize, Out: DeserializeOwned> Request<In, Out> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _p: PhantomData,
        }
    }

    fn full_url(&self) -> String {
        format!("{}/{}", API::base_url(), self.name)
    }
}

impl<Param: Serialize, Output: DeserializeOwned> Request<Param, Output> {
    pub async fn send(&self, param: impl Borrow<Param>) -> Result<Output> {
        let body = to_string(param.borrow())?;
        request_object(Method::Get, self.full_url(), &API::headers(), body.into()).await
    }

    pub async fn with_token(&self, param: impl Borrow<Param>, token: impl ToString) -> Result<Output> {
        let body = to_string(param.borrow())?;
        request_object(
            Method::Get,
            self.full_url(),
            &[("token".to_string(), token.to_string())].into(),
            body.into(),
        )
        .await
    }

    pub async fn with_headers(
        &self,
        param: impl Borrow<Param>,
        headers: impl Into<HashMap<String, String>>,
    ) -> Result<Output> {
        let body = to_string(param.borrow())?;
        request_object(Method::Get, self.full_url(), &headers.into(), body.into()).await
    }
}

async fn request_object<T>(
    method: Method,
    url: String,
    headers: &HashMap<String, String>,
    body: Option<String>,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let response = raw_request(method, url, headers, body).await?;

    if response.status == 404 {
        Err(anyhow!("404 not found"))
    } else if response.status != 200 {
        Err(anyhow!("Object request failed: {response:?}"))
    } else {
        Ok(parse(&response.body)?)
    }
}

fn parse<T: DeserializeOwned>(json: impl ToString) -> Result<T> {
    let json = json.to_string();
    match from_str(&json) {
        Ok(obj) => Ok(obj),
        Err(error) => {
            let message = format!("Failed to parse {} from {json}. Error: {error}", type_name::<T>());
            error!("{message}");
            Err(anyhow!(message))
        }
    }
}

pub async fn raw_request(
    method: Method,
    url: impl ToString,
    headers: &HashMap<String, String>,
    body: Option<String>,
) -> Result<Response> {
    let url = url.to_string();
    let client = Client::new();

    let mut request = match method {
        Method::Get => client.get(&url),
        Method::Post => client.post(&url),
    };

    request = request.header("content-type", "application/json");

    for (key, value) in headers {
        request = request.header(key, value)
    }

    let request = match &body {
        Some(body) => request.body(body.clone()),
        None => request,
    };

    debug!("Request: {url} - {method} {body:?}");

    let response = request.send().await.map_err(|e| {
        error!("Failed to send request: {e}");
        e
    })?;

    let status = response.status();
    let body = response.text().await?;

    let response = Response { url, status, body };

    debug!("Response: {} - {}", response.url, response.status);

    Ok(response)
}
