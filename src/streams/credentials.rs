use keyring::Entry;

pub const SERVICE_SPOTIFY: &str = "Eclipse_Spotify";
pub const SERVICE_YOUTUBE: &str = "Eclipse_YouTube";

pub const USER_ACCESS_TOKEN: &str = "access_token";
pub const USER_REFRESH_TOKEN: &str = "refresh_token";
pub const USER_CLIENT_ID: &str = "client_id";
pub const USER_API_KEY: &str = "api_key";

/// Secure credential store wrapping the OS native credential store (Windows Credential Manager).
/// ZERO-SECRET PRINCIPLE: Never logs passwords/tokens or writes them to files or stdout.
pub struct CredentialStore;

impl CredentialStore {
    /// Save a secret in the Windows Credential Manager.
    pub fn set_secret(service: &str, user: &str, secret: &str) -> Result<(), String> {
        let entry = Entry::new(service, user)
            .map_err(|e| format!("Failed to create keyring entry: {e}"))?;
        entry
            .set_password(secret)
            .map_err(|e| format!("Failed to store credential for service '{service}': {e}"))?;
        Ok(())
    }

    /// Retrieve a secret from the Windows Credential Manager.
    /// Returns Ok(None) if the secret does not exist, rather than failing.
    pub fn get_secret(service: &str, user: &str) -> Result<Option<String>, String> {
        let entry = Entry::new(service, user)
            .map_err(|e| format!("Failed to create keyring entry: {e}"))?;
        match entry.get_password() {
            Ok(pass) => Ok(Some(pass)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!(
                "Failed to retrieve credential for service '{service}': {e}"
            )),
        }
    }

    /// Delete a secret from the Windows Credential Manager.
    /// Returns Ok(true) if deleted, Ok(false) if it didn't exist.
    pub fn delete_secret(service: &str, user: &str) -> Result<bool, String> {
        let entry = Entry::new(service, user)
            .map_err(|e| format!("Failed to create keyring entry: {e}"))?;
        match entry.delete_credential() {
            Ok(()) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(format!(
                "Failed to delete credential for service '{service}': {e}"
            )),
        }
    }

    /// Clean one-time migration: delete any legacy `Eclipse_Spotify` keyring entries.
    pub fn migrate_legacy_credentials() {
        let _ = Self::delete_secret(SERVICE_SPOTIFY, USER_ACCESS_TOKEN);
        let _ = Self::delete_secret(SERVICE_SPOTIFY, USER_REFRESH_TOKEN);
        let _ = Self::delete_secret(SERVICE_SPOTIFY, USER_CLIENT_ID);
    }

    // High-level helpers for YouTube

    pub fn save_youtube_api_key(api_key: &str) -> Result<(), String> {
        Self::set_secret(SERVICE_YOUTUBE, USER_API_KEY, api_key)
    }

    pub fn get_youtube_api_key() -> Result<Option<String>, String> {
        Self::get_secret(SERVICE_YOUTUBE, USER_API_KEY)
    }

    pub fn clear_youtube_credentials() -> Result<(), String> {
        let _ = Self::delete_secret(SERVICE_YOUTUBE, USER_API_KEY);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static CRED_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_nonexistent_secret_returns_none() {
        let _guard = CRED_MUTEX.lock().unwrap();
        let res = CredentialStore::get_secret("Eclipse_Nonexistent_Test_Service", "nonexistent_user");
        assert!(res.is_ok(), "Expected Ok result for nonexistent secret, got: {:?}", res);
        assert_eq!(res.unwrap(), None);
    }

    #[test]
    fn test_secret_roundtrip_and_cleanup() {
        let _guard = CRED_MUTEX.lock().unwrap();
        let test_service = "Eclipse_Test_Service_Unit_2";
        let test_user = "test_user_key_2";
        let test_secret = "super_secret_test_value_12345";

        // Clean up first just in case
        let _ = CredentialStore::delete_secret(test_service, test_user);

        // 1. Store secret
        let save_res = CredentialStore::set_secret(test_service, test_user, test_secret);
        assert!(save_res.is_ok(), "Failed to save test credential: {:?}", save_res);

        // 2. Retrieve secret
        let get_res = CredentialStore::get_secret(test_service, test_user);
        assert!(get_res.is_ok(), "Failed to retrieve test credential: {:?}", get_res);
        assert_eq!(get_res.unwrap(), Some(test_secret.to_string()));

        // 3. Delete secret
        let del_res = CredentialStore::delete_secret(test_service, test_user);
        assert!(del_res.is_ok(), "Failed to delete test credential: {:?}", del_res);
        assert!(del_res.unwrap());

        // 4. Verify it's gone
        let after_del = CredentialStore::get_secret(test_service, test_user);
        assert!(after_del.is_ok());
        assert_eq!(after_del.unwrap(), None);
    }

    #[test]
    fn test_migrate_legacy_credentials() {
        let _guard = CRED_MUTEX.lock().unwrap();
        // Store dummy legacy credentials
        let _ = CredentialStore::set_secret(SERVICE_SPOTIFY, USER_ACCESS_TOKEN, "dummy_access");
        let _ = CredentialStore::set_secret(SERVICE_SPOTIFY, USER_REFRESH_TOKEN, "dummy_refresh");
        let _ = CredentialStore::set_secret(SERVICE_SPOTIFY, USER_CLIENT_ID, "dummy_client");

        // Run migration
        CredentialStore::migrate_legacy_credentials();

        // Verify all legacy secrets are deleted
        assert_eq!(CredentialStore::get_secret(SERVICE_SPOTIFY, USER_ACCESS_TOKEN).unwrap(), None);
        assert_eq!(CredentialStore::get_secret(SERVICE_SPOTIFY, USER_REFRESH_TOKEN).unwrap(), None);
        assert_eq!(CredentialStore::get_secret(SERVICE_SPOTIFY, USER_CLIENT_ID).unwrap(), None);
    }
}
