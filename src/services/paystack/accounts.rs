use crate::repo::accounts::AccountRepo;
use crate::repo::models::Account;
use crate::util::config::Config;
use crate::util::error::ServiceResult;
use crate::util::paystack::{PAYSTACK_BASE_URL, send};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::error::ServiceError as SharedError;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

const BANKS_CACHE_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Bank {
    pub name: String,
    pub code: String,
}

#[derive(Deserialize)]
struct ResolvedAccount {
    account_name: String,
}

#[derive(Deserialize)]
struct RecipientDetails {
    bank_name: String,
}

#[derive(Deserialize)]
struct Recipient {
    recipient_code: String,
    details: RecipientDetails,
}

pub struct AccountService {
    service: String,
    config: Arc<Config>,
    accounts: AccountRepo,
    banks: RwLock<Option<(Instant, Vec<Bank>)>>,
}

impl AccountService {
    pub fn new(config: Arc<Config>) -> Self {
        let accounts = AccountRepo::new(config.db_client.clone(), &config.accounts_table_name);

        Self {
            config,
            accounts,
            service: String::from("paystack"),
            banks: RwLock::new(None),
        }
    }

    pub async fn list_banks(&self) -> ServiceResult<Vec<Bank>> {
        if let Some((fetched_at, banks)) = &*self.banks.read().unwrap_or_else(|e| e.into_inner())
            && fetched_at.elapsed() < BANKS_CACHE_TTL
        {
            return Ok(banks.clone());
        }

        let request = self
            .config
            .http_client
            .get(format!("{PAYSTACK_BASE_URL}/bank?currency=NGN"));

        let banks: Vec<Bank> = send(request).await?;

        *self.banks.write().unwrap_or_else(|e| e.into_inner()) =
            Some((Instant::now(), banks.clone()));

        Ok(banks)
    }

    pub async fn create_recipient(
        &self,
        bank_code: &str,
        account_number: &str,
    ) -> ServiceResult<String> {
        let url = Url::parse_with_params(
            &format!("{PAYSTACK_BASE_URL}/bank/resolve"),
            &[("account_number", account_number), ("bank_code", bank_code)],
        )
        .map_err(|e| {
            println!("{:?}", e);
            SharedError::ServerError
        })?;

        let resolved: ResolvedAccount = send(self.config.http_client.get(url)).await?;

        let request = self
            .config
            .http_client
            .post(format!("{PAYSTACK_BASE_URL}/transferrecipient"))
            .json(&json!({
                "type": "nuban",
                "name": resolved.account_name,
                "account_number": account_number,
                "bank_code": bank_code,
                "currency": "NGN",
            }));

        let recipient: Recipient = send(request).await?;
        let id = Uuid::new_v4().to_string();

        self.accounts
            .create(Account {
                id: id.clone(),
                recipient_code: recipient.recipient_code,
                account_number: account_number.into(),
                account_name: resolved.account_name,
                bank_code: bank_code.into(),
                bank_name: recipient.details.bank_name,
                service: self.service.clone(),
            })
            .await?;

        Ok(id)
    }
}
