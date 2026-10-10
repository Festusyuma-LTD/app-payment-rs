use crate::repo::models::{Payout, PayoutStatus};
use crate::util::dynamo::{condition_failed_as, get_by_id, insert};
use crate::util::error::{ServiceError, ServiceResult};

use aws_sdk_dynamodb::types::AttributeValue;
use shared::error::ServiceError as SharedError;

pub(crate) struct PayoutRepo {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl PayoutRepo {
    pub(crate) fn new(client: aws_sdk_dynamodb::Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }

    pub(crate) async fn get(&self, id: &str) -> ServiceResult<Option<Payout>> {
        get_by_id(&self.client, &self.table_name, id, Payout::from_item).await
    }

    pub(crate) async fn create(&self, payout: Payout) -> ServiceResult<()> {
        insert(
            &self.client,
            &self.table_name,
            payout.into_item(),
            ServiceError::PayoutExists,
        )
        .await
    }

    pub(crate) async fn update_transfer(
        &self,
        id: &str,
        transfer_code: &str,
        status: PayoutStatus,
    ) -> ServiceResult<()> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.into()))
            .update_expression("SET transfer_code = :transfer_code, #status = :status")
            .expression_attribute_names("#status", "status")
            .expression_attribute_values(":transfer_code", AttributeValue::S(transfer_code.into()))
            .expression_attribute_values(":status", AttributeValue::S(status.as_str().into()))
            .send()
            .await
            .map_err(|e| {
                println!("{:?}", e);
                SharedError::ServerError
            })?;

        Ok(())
    }

    pub(crate) async fn update_status(&self, id: &str, status: PayoutStatus) -> ServiceResult<()> {
        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.into()))
            .update_expression("SET #status = :status")
            .condition_expression("attribute_exists(id)")
            .expression_attribute_names("#status", "status")
            .expression_attribute_values(":status", AttributeValue::S(status.as_str().into()))
            .send()
            .await
            .map_err(condition_failed_as(ServiceError::PayoutNotFound))?;

        Ok(())
    }
}
