use crate::util::error::{ServiceError, ServiceResult};

use aws_sdk_dynamodb::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_dynamodb::types::AttributeValue;
use shared::error::ServiceError as SharedError;
use std::collections::HashMap;
use std::fmt::Debug;

pub(crate) async fn get_by_id<T>(
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    id: &str,
    from_item: impl FnOnce(&HashMap<String, AttributeValue>) -> Option<T>,
) -> ServiceResult<Option<T>> {
    let output = client
        .get_item()
        .table_name(table_name)
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

    from_item(&item).map(Some).ok_or_else(|| {
        println!("malformed item in {table_name}: {:?}", item);
        SharedError::ServerError
    })
}

pub(crate) async fn insert(
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    item: HashMap<String, AttributeValue>,
    exists_error: ServiceError,
) -> ServiceResult<()> {
    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .condition_expression("attribute_not_exists(id)")
        .send()
        .await
        .map_err(condition_failed_as(exists_error))?;

    Ok(())
}

pub(crate) fn condition_failed_as<E, R>(
    error: ServiceError,
) -> impl FnOnce(SdkError<E, R>) -> SharedError
where
    E: ProvideErrorMetadata + Debug,
    R: Debug,
{
    move |e| {
        if e.code() == Some("ConditionalCheckFailedException") {
            return error.into();
        }

        println!("{:?}", e);
        SharedError::ServerError
    }
}
