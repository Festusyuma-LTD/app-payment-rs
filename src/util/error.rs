use shared::error::ServiceError as SharedError;

pub type ServiceResult<T> = Result<T, SharedError>;

pub enum ServiceError {
    PaymentExists,
    PaymentNotFound,
    AccountExists,
    AccountNotFound,
    PayoutExists,
    PayoutNotFound,
    PayoutNotAwaitingOtp,
    Paystack(String),
}

impl Into<SharedError> for ServiceError {
    fn into(self) -> SharedError {
        let (code, message) = match self {
            ServiceError::PaymentExists => (409, "payment already exists".into()),
            ServiceError::PaymentNotFound => (400, "payment not found".into()),
            ServiceError::AccountExists => (409, "account already exists".into()),
            ServiceError::AccountNotFound => (400, "account not found".into()),
            ServiceError::PayoutExists => (409, "payout already exists".into()),
            ServiceError::PayoutNotFound => (400, "payout not found".into()),
            ServiceError::PayoutNotAwaitingOtp => (400, "payout is not awaiting otp".into()),
            ServiceError::Paystack(message) => (400, message),
        };

        SharedError::HttpMessage(code, message)
    }
}

impl<T> Into<ServiceResult<T>> for ServiceError {
    fn into(self) -> ServiceResult<T> {
        Err(self.into())
    }
}
