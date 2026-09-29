use aws_sdk_dynamodb::types::AttributeValue;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Ongoing,
    Processing,
    Queued,
    Success,
    Failed,
    Abandoned,
    Reversed,
    #[serde(other)]
    Pending,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Ongoing => "ongoing",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Queued => "queued",
            PaymentStatus::Success => "success",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Abandoned => "abandoned",
            PaymentStatus::Reversed => "reversed",
        }
    }

    fn from_str(value: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .unwrap_or(PaymentStatus::Pending)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Payment {
    pub id: String,
    pub reference: String,
    pub amount: u64,
    pub service: String,
    pub status: PaymentStatus,
}

impl Payment {
    pub(crate) fn into_item(self) -> HashMap<String, AttributeValue> {
        HashMap::from([
            ("id".into(), AttributeValue::S(self.id)),
            ("reference".into(), AttributeValue::S(self.reference)),
            ("amount".into(), AttributeValue::N(self.amount.to_string())),
            ("service".into(), AttributeValue::S(self.service)),
            (
                "status".into(),
                AttributeValue::S(self.status.as_str().into()),
            ),
        ])
    }

    pub(crate) fn from_item(item: &HashMap<String, AttributeValue>) -> Option<Self> {
        Some(Self {
            id: item.get("id")?.as_s().ok()?.clone(),
            reference: item.get("reference")?.as_s().ok()?.clone(),
            amount: item.get("amount")?.as_n().ok()?.parse().ok()?,
            service: item.get("service")?.as_s().ok()?.clone(),
            status: PaymentStatus::from_str(item.get("status")?.as_s().ok()?),
        })
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PayoutStatus {
    Otp,
    Received,
    Processing,
    Success,
    Failed,
    Reversed,
    Abandoned,
    Blocked,
    Rejected,
    #[serde(other)]
    Pending,
}

impl PayoutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayoutStatus::Pending => "pending",
            PayoutStatus::Otp => "otp",
            PayoutStatus::Received => "received",
            PayoutStatus::Processing => "processing",
            PayoutStatus::Success => "success",
            PayoutStatus::Failed => "failed",
            PayoutStatus::Reversed => "reversed",
            PayoutStatus::Abandoned => "abandoned",
            PayoutStatus::Blocked => "blocked",
            PayoutStatus::Rejected => "rejected",
        }
    }

    fn from_str(value: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .unwrap_or(PayoutStatus::Pending)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Account {
    pub id: String,
    pub recipient_code: String,
    pub account_number: String,
    pub account_name: String,
    pub bank_code: String,
    pub bank_name: String,
    pub service: String,
}

impl Account {
    pub(crate) fn into_item(self) -> HashMap<String, AttributeValue> {
        HashMap::from([
            ("id".into(), AttributeValue::S(self.id)),
            (
                "recipient_code".into(),
                AttributeValue::S(self.recipient_code),
            ),
            (
                "account_number".into(),
                AttributeValue::S(self.account_number),
            ),
            ("account_name".into(), AttributeValue::S(self.account_name)),
            ("bank_code".into(), AttributeValue::S(self.bank_code)),
            ("bank_name".into(), AttributeValue::S(self.bank_name)),
            ("service".into(), AttributeValue::S(self.service)),
        ])
    }

    pub(crate) fn from_item(item: &HashMap<String, AttributeValue>) -> Option<Self> {
        Some(Self {
            id: item.get("id")?.as_s().ok()?.clone(),
            recipient_code: item.get("recipient_code")?.as_s().ok()?.clone(),
            account_number: item.get("account_number")?.as_s().ok()?.clone(),
            account_name: item.get("account_name")?.as_s().ok()?.clone(),
            bank_code: item.get("bank_code")?.as_s().ok()?.clone(),
            bank_name: item.get("bank_name")?.as_s().ok()?.clone(),
            service: item.get("service")?.as_s().ok()?.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Payout {
    pub id: String,
    pub account_id: String,
    pub reference: String,
    pub transfer_code: Option<String>,
    pub amount: u64,
    pub service: String,
    pub status: PayoutStatus,
}

impl Payout {
    pub(crate) fn into_item(self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::from([
            ("id".into(), AttributeValue::S(self.id)),
            ("account_id".into(), AttributeValue::S(self.account_id)),
            ("reference".into(), AttributeValue::S(self.reference)),
            ("amount".into(), AttributeValue::N(self.amount.to_string())),
            ("service".into(), AttributeValue::S(self.service)),
            (
                "status".into(),
                AttributeValue::S(self.status.as_str().into()),
            ),
        ]);

        if let Some(transfer_code) = self.transfer_code {
            item.insert("transfer_code".into(), AttributeValue::S(transfer_code));
        }

        item
    }

    pub(crate) fn from_item(item: &HashMap<String, AttributeValue>) -> Option<Self> {
        Some(Self {
            id: item.get("id")?.as_s().ok()?.clone(),
            account_id: item.get("account_id")?.as_s().ok()?.clone(),
            reference: item.get("reference")?.as_s().ok()?.clone(),
            transfer_code: item
                .get("transfer_code")
                .and_then(|v| v.as_s().ok())
                .cloned(),
            amount: item.get("amount")?.as_n().ok()?.parse().ok()?,
            service: item.get("service")?.as_s().ok()?.clone(),
            status: PayoutStatus::from_str(item.get("status")?.as_s().ok()?),
        })
    }
}
