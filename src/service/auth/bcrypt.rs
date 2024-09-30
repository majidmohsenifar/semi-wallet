pub fn encrypt_password(pass: &str) -> Result<String, bcrypt::BcryptError> {
    let hashed = bcrypt::hash(pass, bcrypt::DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(pass: &str, encoded_password: &str) -> Result<bool, bcrypt::BcryptError> {
    let is_verified = bcrypt::verify(pass, encoded_password)?;
    Ok(is_verified)
}
