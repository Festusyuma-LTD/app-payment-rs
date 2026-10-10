use crate::repo::models::Account;
use crate::util::dynamo::{get_by_id, insert};
use crate::util::error::{ServiceError, ServiceResult};

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
        get_by_id(&self.client, &self.table_name, id, Account::from_item).await
    }

    pub(crate) async fn create(&self, account: Account) -> ServiceResult<()> {
        insert(
            &self.client,
            &self.table_name,
            account.into_item(),
            ServiceError::AccountExists,
        )
        .await
    }
}
