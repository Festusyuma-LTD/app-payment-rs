use crate::repo::models::Account;
use crate::util::error::{ServiceError, ServiceResult};

use aws_sdk_dynamodb::types::AttributeValue;
use shared::error::ServiceError as SharedError;

pub(crate) struct AccountRepo {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl AccountRepo {
    pub(crate) fn new(client: aws_sdk_dynamodb::Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }

    pub(crate) async fn get(&self, id: &str) -> ServiceResult<Option<Account>> {
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

        Account::from_item(&item).map(Some).ok_or_else(|| {
            println!("malformed account item: {:?}", item);
            SharedError::ServerError
        })
    }

    pub(crate) async fn create(&self, account: Account) -> ServiceResult<()> {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(account.into_item()))
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await
            .map_err(|e| {
                if e.as_service_error()
                    .is_some_and(|e| e.is_conditional_check_failed_exception())
                {
                    return ServiceError::AccountExists.into();
                }

                println!("{:?}", e);
                SharedError::ServerError
            })?;

        Ok(())
    }
}
