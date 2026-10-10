use crate::repo::models::{Payment, PaymentStatus};
use crate::util::dynamo::{get_by_id, insert};
use crate::util::error::{ServiceError, ServiceResult};

use aws_sdk_dynamodb::types::AttributeValue;
use shared::error::ServiceError as SharedError;

pub(crate) struct PaymentRepo {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl PaymentRepo {
    pub(crate) fn new(client: aws_sdk_dynamodb::Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }

    pub(crate) async fn get(&self, id: &str) -> ServiceResult<Option<Payment>> {
        get_by_id(&self.client, &self.table_name, id, Payment::from_item).await
    }

    pub(crate) async fn create(&self, payment: Payment) -> ServiceResult<()> {
        insert(
            &self.client,
            &self.table_name,
            payment.into_item(),
            ServiceError::PaymentExists,
        )
        .await
    }

    pub(crate) async fn update_status(&self, id: &str, status: PaymentStatus) -> ServiceResult<()> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.into()))
            .update_expression("SET #status = :status")
            .expression_attribute_names("#status", "status")
            .expression_attribute_values(":status", AttributeValue::S(status.as_str().into()))
            .send()
            .await
            .map_err(|e| {
                println!("{:?}", e);
                SharedError::ServerError
            })?;

        Ok(())
    }
}
