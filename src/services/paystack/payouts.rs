use crate::repo::accounts::AccountRepo;
use crate::repo::models::{Payout, PayoutStatus};
use crate::repo::payouts::PayoutRepo;
use crate::util::config::Config;
use crate::util::error::{ServiceError, ServiceResult};
use crate::util::paystack::{PAYSTACK_BASE_URL, send};

use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
struct PaystackTransfer {
    transfer_code: String,
    status: PayoutStatus,
}

pub struct PayoutService {
    service: String,
    config: Arc<Config>,
    accounts: AccountRepo,
    payouts: PayoutRepo,
}

impl PayoutService {
    pub fn new(config: Arc<Config>) -> Self {
        let accounts = AccountRepo::new(config.db_client.clone(), &config.accounts_table_name);
        let payouts = PayoutRepo::new(config.db_client.clone(), &config.payout_table_name);

        Self {
            config,
            accounts,
            payouts,
            service: String::from("paystack"),
        }
    }

    pub async fn init_transfer(
        &self,
        id: &str,
        account_id: &str,
        amount: u64,
        reason: Option<&str>,
    ) -> ServiceResult<PayoutStatus> {
        if self.payouts.get(id).await?.is_some() {
            return ServiceError::PayoutExists.into();
        }

        let Some(account) = self.accounts.get(account_id).await? else {
            return ServiceError::AccountNotFound.into();
        };

        let reference = Uuid::new_v4().to_string();

        self.payouts
            .create(Payout {
                id: id.into(),
                account_id: account_id.into(),
                reference: reference.clone(),
                transfer_code: None,
                amount,
                service: self.service.clone(),
                status: PayoutStatus::Pending,
            })
            .await?;

        let mut body = json!({
            "source": "balance",
            "amount": amount,
            "recipient": account.recipient_code,
            "reference": reference,
        });

        if let Some(reason) = reason {
            body["reason"] = reason.into();
        }

        let request = self
            .config
            .http_client
            .post(format!("{PAYSTACK_BASE_URL}/transfer"))
            .json(&body);

        let transfer: PaystackTransfer = send(request).await?;

        self.payouts
            .update_transfer(id, &transfer.transfer_code, transfer.status)
            .await?;

        Ok(transfer.status)
    }

    pub async fn complete_transfer(&self, id: &str, otp: &str) -> ServiceResult<PayoutStatus> {
        let Some(payout) = self.payouts.get(id).await? else {
            return ServiceError::PayoutNotFound.into();
        };

        let (PayoutStatus::Otp, Some(transfer_code)) = (payout.status, payout.transfer_code) else {
            return ServiceError::PayoutNotAwaitingOtp.into();
        };

        let request = self
            .config
            .http_client
            .post(format!("{PAYSTACK_BASE_URL}/transfer/finalize_transfer"))
            .json(&json!({
                "transfer_code": transfer_code,
                "otp": otp,
            }));

        let transfer: PaystackTransfer = send(request).await?;

        if transfer.status != payout.status {
            self.payouts.update_status(id, transfer.status).await?;
        }

        Ok(transfer.status)
    }

    pub async fn verify_transfer(&self, id: &str) -> ServiceResult<PayoutStatus> {
        let Some(payout) = self.payouts.get(id).await? else {
            return ServiceError::PayoutNotFound.into();
        };

        if payout.status == PayoutStatus::Success {
            return Ok(payout.status);
        }

        let request = self.config.http_client.get(format!(
            "{PAYSTACK_BASE_URL}/transfer/verify/{}",
            payout.reference
        ));

        let transfer: PaystackTransfer = send(request).await?;

        if transfer.status != payout.status || payout.transfer_code.is_none() {
            self.payouts
                .update_transfer(id, &transfer.transfer_code, transfer.status)
                .await?;
        }

        Ok(transfer.status)
    }

    pub async fn update_transfer_status(
        &self,
        id: &str,
        status: PayoutStatus,
    ) -> ServiceResult<()> {
        self.payouts.update_status(id, status).await
    }
}
