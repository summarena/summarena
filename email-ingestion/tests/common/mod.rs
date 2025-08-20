// Re-export commonly used test types and utilities
pub use email_ingestion::email_ingester::EmailIngester;
pub use email_ingestion::database::{EmailDatabase, EmailCredential};

// GreenMail integration utilities
pub use crate::greenmail_helper::{GreenMailHelper, TestEmail, create_test_emails, wait_for_server_ready};

/// Test configuration constants
pub const TEST_EMAIL_ADDRESS: &str = "test@localhost";
pub const GREENMAIL_IMAP_URI: &str = "email://test@localhost:3143/INBOX?tls=false";
pub const GREENMAIL_IMAPS_URI: &str = "email://test@localhost:3993/INBOX?tls=true";

/// Create test database URL with fallback
pub fn get_test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test_email_ingestion".to_string())
}

/// Set up test database with schema and clean state
pub async fn setup_test_database() -> anyhow::Result<EmailDatabase> {
    let db = EmailDatabase::new(&get_test_database_url()).await?;
    db.setup_schema().await?;
    Ok(db)
}

/// Set up GreenMail test environment
pub async fn setup_greenmail_environment() -> anyhow::Result<GreenMailHelper> {
    wait_for_server_ready(10).await?;
    let greenmail = GreenMailHelper::new();
    greenmail.reset_server().await?;
    Ok(greenmail)
}

/// Create and store test credentials in database
pub async fn setup_test_credentials(db: &EmailDatabase, email: &str, password: &str) -> anyhow::Result<()> {
    db.store_credentials(email, password).await?;
    Ok(())
}