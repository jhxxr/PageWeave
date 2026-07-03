use keyring::Entry;

use crate::error::{AppError, AppResult};

const SERVICE: &str = "PageWeave";

pub fn set_secret(id: &str, value: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, id).map_err(|e| AppError::Secret(e.to_string()))?;
    entry
        .set_password(value)
        .map_err(|e| AppError::Secret(e.to_string()))
}

pub fn get_secret(id: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE, id).map_err(|e| AppError::Secret(e.to_string()))?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Secret(e.to_string())),
    }
}

pub fn delete_secret(id: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, id).map_err(|e| AppError::Secret(e.to_string()))?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Secret(e.to_string())),
    }
}
