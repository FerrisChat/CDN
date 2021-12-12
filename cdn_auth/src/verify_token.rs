use num_bigint::BigInt;
use sqlx::types::BigDecimal;

use cdn_common::VerifyTokenFailure;

use crate::DATABASE_POOL;

#[allow(clippy::missing_panics_doc)]
/// Verify a user's token.
///
/// # Errors
/// Returns an error if any of the following happen:
/// * The DB pool is not initialized.
/// * Auth data is invalid.
/// * The DB returns an error.
/// * The global verifier is not found.
/// * A verification error occurs.
pub async fn verify_token(user_id: u128, secret: String) -> Result<(), VerifyTokenFailure> {
    let id_bigint = BigDecimal::new(BigInt::from(user_id), 0);
    let db = DATABASE_POOL
        .get()
        .ok_or(VerifyTokenFailure::MissingDatabase)?;

    let db_token = sqlx::query!(
        "SELECT (auth_token) FROM auth_tokens WHERE user_id = $1",
        id_bigint
    )
    .fetch_optional(db)
    .await?
    .ok_or(VerifyTokenFailure::InvalidToken)?
    .auth_token;

    if argon2_async::verify(secret, db_token).await? {
        Ok(())
    } else {
        Err(VerifyTokenFailure::InvalidToken)
    }
}
