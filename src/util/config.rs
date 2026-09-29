use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

#[derive(Default)]
pub struct ConfigBuilder {
    paystack_secret: Option<String>,
    payment_table_name: Option<String>,
    accounts_table_name: Option<String>,
    payout_table_name: Option<String>,
}

impl ConfigBuilder {
    pub fn paystack_secret(self, paystack_secret: impl Into<String>) -> Self {
        Self {
            paystack_secret: Some(paystack_secret.into()),
            ..self
        }
    }

    pub fn payment_table_name(self, payment_table_name: impl Into<String>) -> Self {
        Self {
            payment_table_name: Some(payment_table_name.into()),
            ..self
        }
    }

    pub fn accounts_table_name(self, accounts_table_name: impl Into<String>) -> Self {
        Self {
            accounts_table_name: Some(accounts_table_name.into()),
            ..self
        }
    }

    pub fn payout_table_name(self, payout_table_name: impl Into<String>) -> Self {
        Self {
            payout_table_name: Some(payout_table_name.into()),
            ..self
        }
    }

    pub async fn build(self) -> Config {
        Config::new(self).await
    }
}

pub struct Config {
    pub(crate) db_client: aws_sdk_dynamodb::Client,
    pub(crate) http_client: reqwest::Client,
    pub(crate) paystack_secret: String,
    pub(crate) payment_table_name: String,
    pub(crate) accounts_table_name: String,
    pub(crate) payout_table_name: String,
}

impl Config {
    async fn new(builder: ConfigBuilder) -> Config {
        let paystack_secret = builder
            .paystack_secret
            .expect("paystack_secret is required");
        let payment_table_name = builder
            .payment_table_name
            .expect("payment_table_name is required");
        let accounts_table_name = builder
            .accounts_table_name
            .expect("accounts_table_name is required");
        let payout_table_name = builder
            .payout_table_name
            .expect("payout_table_name is required");

        let config = aws_config::load_from_env().await;

        let mut auth = HeaderValue::from_str(&format!("Bearer {paystack_secret}"))
            .expect("paystack_secret must be a valid header value");
        auth.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, auth);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build http client");

        Self {
            db_client: aws_sdk_dynamodb::Client::new(&config),
            http_client,
            paystack_secret,
            payment_table_name,
            accounts_table_name,
            payout_table_name,
        }
    }

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}
