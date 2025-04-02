use crate::error::OpenRouterError;
use keyring::Entry;
use zeroize::Zeroizing;

const SERVICE_NAME: &str = "openrouter-rs";
const DEFAULT_API_KEY: &str = "default-api-key";
const PROVISIONING_API_KEY: &str = "provisioning-api-key";

/// Securely store API key in system keychain
pub fn store(api_key: &str) -> Result<(), OpenRouterError> {
    let entry = Entry::new(SERVICE_NAME, DEFAULT_API_KEY).map_err(|e| {
        OpenRouterError::Keychain(format!("Failed to create keychain entry: {}", e))
    })?;

    entry
        .set_password(api_key)
        .map_err(|e| OpenRouterError::Keychain(format!("Failed to store key: {}", e)))
}

/// Retrieve API key from system keychain
///
/// Returned key is wrapped in Zeroizing for secure erasure
pub fn get() -> Result<Zeroizing<String>, OpenRouterError> {
    let entry = Entry::new(SERVICE_NAME, DEFAULT_API_KEY).map_err(|e| {
        OpenRouterError::Keychain(format!("Failed to create keychain entry: {}", e))
    })?;

    // Check if the entry exists before trying to get the password
    match entry.get_password() {
        Ok(password) => Ok(Zeroizing::new(password)),
        Err(keyring::Error::NoEntry) => Err(OpenRouterError::KeyNotConfigured),
        Err(e) => Err(OpenRouterError::Keychain(format!(
            "Failed to retrieve key: {}",
            e
        ))),
    }
}

/// Delete API key from system keychain
pub fn delete() -> Result<(), OpenRouterError> {
    let entry = Entry::new(SERVICE_NAME, DEFAULT_API_KEY).map_err(|e| {
        OpenRouterError::Keychain(format!("Failed to create keychain entry: {}", e))
    })?;

    // Check if the entry exists before trying to delete the credential
    match entry.get_password() {
        Ok(_) => entry
            .delete_credential()
            .map_err(|e| OpenRouterError::Keychain(format!("Failed to delete key: {}", e))),
        Err(keyring::Error::NoEntry) => Err(OpenRouterError::KeyNotConfigured),
        Err(e) => Err(OpenRouterError::Keychain(format!(
            "Failed to check key existence: {}",
            e
        ))),
    }
}

/// Securely store API key in system keychain
pub fn store_provisioning(api_key: &str) -> Result<(), OpenRouterError> {
    let entry = Entry::new(SERVICE_NAME, PROVISIONING_API_KEY).map_err(|e| {
        OpenRouterError::Keychain(format!("Failed to create keychain entry: {}", e))
    })?;

    entry
        .set_password(api_key)
        .map_err(|e| OpenRouterError::Keychain(format!("Failed to store key: {}", e)))
}

/// Retrieve API key from system keychain
///
/// Returned key is wrapped in Zeroizing for secure erasure
pub fn get_provisioning() -> Result<Zeroizing<String>, OpenRouterError> {
    let entry = Entry::new(SERVICE_NAME, PROVISIONING_API_KEY).map_err(|e| {
        OpenRouterError::Keychain(format!("Failed to create keychain entry: {}", e))
    })?;

    // Check if the entry exists before trying to get the password
    match entry.get_password() {
        Ok(password) => Ok(Zeroizing::new(password)),
        Err(keyring::Error::NoEntry) => Err(OpenRouterError::KeyNotConfigured),
        Err(e) => Err(OpenRouterError::Keychain(format!(
            "Failed to retrieve key: {}",
            e
        ))),
    }
}

/// Delete API key from system keychain
pub fn delete_provisioning() -> Result<(), OpenRouterError> {
    let entry = Entry::new(SERVICE_NAME, PROVISIONING_API_KEY).map_err(|e| {
        OpenRouterError::Keychain(format!("Failed to create keychain entry: {}", e))
    })?;

    // Check if the entry exists before trying to delete the credential
    match entry.get_password() {
        Ok(_) => entry
            .delete_credential()
            .map_err(|e| OpenRouterError::Keychain(format!("Failed to delete key: {}", e))),
        Err(keyring::Error::NoEntry) => Err(OpenRouterError::KeyNotConfigured),
        Err(e) => Err(OpenRouterError::Keychain(format!(
            "Failed to check key existence: {}",
            e
        ))),
    }
}
