use crate::repo::models::{Payout, PayoutStatus};
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
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.into()))
            .send()
            .await
            .map_err(|e| {
                println!("{:?}", e);
                SharedError::ServerError
            })?;

        let Some(item) = output.item else {
            return Ok(None);
        };

        Payout::from_item(&item).map(Some).ok_or_else(|| {
            println!("malformed payout item: {:?}", item);
            SharedError::ServerError
        })
    }

    pub(crate) async fn create(&self, payout: Payout) -> ServiceResult<()> {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(payout.into_item()))
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await
            .map_err(|e| {
                if e.as_service_error()
                    .is_some_and(|e| e.is_conditional_check_failed_exception())
                {
                    return ServiceError::PayoutExists.into();
                }

                println!("{:?}", e);
                SharedError::ServerError
            })?;

        Ok(())
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
            .map_err(|e| {
                if e.as_service_error()
                    .is_some_and(|e| e.is_conditional_check_failed_exception())
                {
                    return ServiceError::PayoutNotFound.into();
                }

                println!("{:?}", e);
                SharedError::ServerError
            })?;

        Ok(())
    }
}
