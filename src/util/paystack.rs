use crate::util::error::{ServiceError, ServiceResult};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use shared::error::ServiceError as SharedError;

pub(crate) const PAYSTACK_BASE_URL: &str = "https://api.paystack.co";

#[derive(Deserialize)]
struct PaystackResponse<T> {
    status: bool,
    message: String,
    data: Option<T>,
}

pub(crate) async fn send<T: DeserializeOwned>(
    request: reqwest::RequestBuilder,
) -> ServiceResult<T> {
    let response = request.send().await.map_err(|e| {
        println!("{:?}", e);
        SharedError::ServerError
    })?;

    let body = response.bytes().await.map_err(|e| {
        println!("{:?}", e);
        SharedError::ServerError
    })?;

    let response = serde_json::from_slice::<PaystackResponse<T>>(&body).map_err(|e| {
        println!("{:?}: {}", e, String::from_utf8_lossy(&body));
        SharedError::ServerError
    })?;

    match response.data {
        Some(data) if response.status => Ok(data),
        _ => ServiceError::Paystack(response.message).into(),
    }
}
