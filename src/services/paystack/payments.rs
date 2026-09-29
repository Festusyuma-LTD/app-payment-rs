use crate::repo::models::{Payment, PaymentStatus};
use crate::repo::payments::PaymentRepo;
use crate::util::config::Config;
use crate::util::error::{ServiceError, ServiceResult};
use crate::util::paystack::{PAYSTACK_BASE_URL, send};

use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
pub struct PaymentInit {
    pub authorization_url: String,
    pub access_code: String,
    pub reference: String,
}

#[derive(Deserialize)]
struct PaystackTransaction {
    status: PaymentStatus,
    amount: u64,
}

pub struct PaymentService {
    service: String,
    config: Arc<Config>,
    payments: PaymentRepo,
}

impl PaymentService {
    pub fn new(config: Arc<Config>) -> Self {
        let payments = PaymentRepo::new(config.db_client.clone(), &config.payment_table_name);

        Self {
            config,
            payments,
            service: String::from("paystack"),
        }
    }

    pub async fn create_payment(
        &self,
        id: &str,
        email: &str,
        amount: u64,
        callback_url: Option<&str>,
    ) -> ServiceResult<PaymentInit> {
        if self.payments.get(id).await?.is_some() {
            return ServiceError::PaymentExists.into();
        }

        let mut body = json!({
            "email": email,
            "amount": amount.to_string(),
        });

        if let Some(callback_url) = callback_url {
            body["callback_url"] = callback_url.into();
        }

        let request = self
            .config
            .http_client
            .post(format!("{PAYSTACK_BASE_URL}/transaction/initialize"))
            .json(&body);

        let init: PaymentInit = send(request).await?;

        self.payments
            .create(Payment {
                id: id.into(),
                reference: init.reference.clone(),
                amount,
                service: self.service.clone(),
                status: PaymentStatus::Pending,
            })
            .await?;

        Ok(init)
    }

    pub async fn verify_payment(&self, id: &str) -> ServiceResult<PaymentStatus> {
        let Some(payment) = self.payments.get(id).await? else {
            return ServiceError::PaymentNotFound.into();
        };

        if payment.status == PaymentStatus::Success {
            return Ok(payment.status);
        }

        let request = self.config.http_client.get(format!(
            "{PAYSTACK_BASE_URL}/transaction/verify/{}",
            payment.reference
        ));

        let transaction: PaystackTransaction = send(request).await?;

        let status = match transaction.status {
            PaymentStatus::Success if transaction.amount != payment.amount => {
                println!(
                    "payment {id} amount mismatch: expected {}, paystack returned {}",
                    payment.amount, transaction.amount
                );
                PaymentStatus::Failed
            }
            status => status,
        };

        if status != payment.status {
            self.payments.update_status(id, status).await?;
        }

        Ok(status)
    }
}
