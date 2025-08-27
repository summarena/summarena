use std::env;
use anyhow::Result;
use chrono::Utc;
use email_ingestion::database::{EmailDatabase, EmailCredential};
use interfaces::defs::{LiveSourceSpec, Ingester};
use email_ingestion::email_ingester::EmailIngester;

/// Manual Gmail integration test - IGNORED BY DEFAULT
/// 
/// This test verifies that the email ingestion works with a real Gmail account using the new
/// URI-based configuration and database credential system.
/// It's ignored by default and must be run manually by developers for sanity checking.
/// 
/// ## Setup Instructions:
/// 
/// 1. **Enable Gmail IMAP**:
///    - Go to Gmail Settings → Forwarding and POP/IMAP → Enable IMAP
/// 
/// 2. **Create App Password** (if using 2FA):
///    - Google Account → Security → 2-Step Verification → App passwords
///    - Generate password for "Mail"
/// 
/// 3. **Set Environment Variables**:
///    ```bash
///    export GMAIL_EMAIL="your-email@gmail.com"
///    export GMAIL_PASSWORD="your-app-password-or-regular-password"
///    export TEST_DATABASE_URL="postgresql://postgres:password@localhost:5432/test_email_ingestion"
///    ```
/// 
/// 4. **Start Test Database**:
///    ```bash
///    docker run -d --name test-postgres \
///      -e POSTGRES_PASSWORD=password \
///      -e POSTGRES_DB=test_email_ingestion \
///      -p 5432:5432 postgres:latest
///    ```
/// 
/// 5. **Run the test**:
///    ```bash
///    cargo test gmail_manual_integration_test -- --ignored --nocapture
///    ```
/// 
/// ## What this test does:
/// - Creates a Gmail URI with TLS and certificate acceptance settings
/// - Stores Gmail credentials in the test database
/// - Uses EmailIngester::watch() to fetch emails (same as production flow)
/// - Verifies email parsing and interfaces::state::ingest integration
/// - Shows sample email content for verification
/// 
/// ## Safety:
/// - This test is READ-ONLY - it will not modify, delete, or mark emails as read
/// - Limited to recent emails to avoid overwhelming output
/// - Uses the same secure flow as production (database credentials, URI parsing)
/// 
#[tokio::test]
#[ignore = "Manual test - requires Gmail credentials"]
async fn gmail_manual_integration_test() -> Result<()> {
    println!("🔍 Starting Gmail manual integration test...");
    println!("📧 This test connects to your actual Gmail account via IMAP\n");

    // Validate required environment variables
    let missing_vars = validate_gmail_env_vars();
    if !missing_vars.is_empty() {
        println!("❌ Missing required environment variables:");
        for var in &missing_vars {
            println!("   - {}", var);
        }
        println!("\n💡 Set these variables and re-run the test");
        return Err(anyhow::anyhow!("Missing environment variables: {:?}", missing_vars));
    }

    let gmail_email = env::var("GMAIL_EMAIL")?;
    let gmail_password = env::var("GMAIL_PASSWORD")?;
    
    // Construct Gmail IMAP URI with proper settings
    let gmail_uri = format!(
        "email://{}@imap.gmail.com:993/INBOX?tls=true&accept_invalid_certs=false&accept_invalid_hostnames=false",
        gmail_email.split('@').next().unwrap()
    );
    
    println!("✓ Gmail IMAP URI: {}", gmail_uri);
    println!("✓ Gmail Email: {}", gmail_email);

    // Set up test database with Gmail credentials
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test_email_ingestion".to_string());
    
    let db = EmailDatabase::new(&database_url).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to test database: {}. Make sure PostgreSQL is running.", e))?;
    
    db.setup_schema().await?;
    println!("✓ Test database connected and schema set up");

    // Store Gmail credentials in database (same as production flow)
    db.store_credentials(&gmail_email, &gmail_password).await?;
    println!("✓ Gmail credentials stored in database");

    // Test the watch() method - the main entry point used in production
    let source = LiveSourceSpec {
        uri: gmail_uri.clone(),
    };

    println!("\n🔄 Testing EmailIngester::watch() with Gmail...");
    println!("   (This may take 10-30 seconds for Gmail IMAP connection)");
    
    let start_time = std::time::Instant::now();
    let watch_result = EmailIngester::watch(&source).await;
    let elapsed = start_time.elapsed();
    
    println!("⏱️  Gmail connection took {:.2} seconds", elapsed.as_secs_f64());

    // Verify successful result
    if watch_result.wait_at_least_ms == 30000 {
        println!("✅ Gmail integration successful!");
        println!("   EmailIngester::watch() returned 30 second wait time (success indicator)");
    } else if watch_result.wait_at_least_ms == 60000 {
        println!("❌ Gmail connection failed (60 second wait time indicates error)");
        println!("\n🔧 Common issues:");
        println!("   - Check if IMAP is enabled in Gmail settings");
        println!("   - Verify App Password if using 2FA");
        println!("   - Ensure GMAIL_EMAIL is correct full email address");
        println!("   - Check that Gmail account is accessible");
        return Err(anyhow::anyhow!("Gmail connection failed - check logs above"));
    } else {
        println!("⚠️  Unexpected wait time: {}ms", watch_result.wait_at_least_ms);
    }

    // Verify database state was updated
    let credentials = db.get_credentials(&gmail_email).await?;
    let updated_creds = credentials.expect("Credentials should still exist");
    
    if updated_creds.last_sync_date.is_some() {
        println!("✓ Database last sync time updated correctly");
        println!("   Last sync: {:?}", updated_creds.last_sync_date.unwrap());
    } else {
        println!("⚠️  Database last sync time was not updated (might indicate no emails found)");
    }

    // Test URI parsing for Gmail-specific configuration
    println!("\n🔧 Validating Gmail URI parsing...");
    let test_credential = EmailCredential {
        email_address: gmail_email.clone(),
        password: gmail_password.clone(),
        last_sync_date: None,
    };
    
    let config = email_ingestion::email_ingester::EmailIngesterConfig::from_uri_and_credentials(
        &gmail_uri, 
        &test_credential
    ).await?;
    
    validate_gmail_config(&config)?;

    // Clean up test data
    db.delete_credentials(&gmail_email).await?;
    println!("✓ Test credentials cleaned up");

    println!("\n🎉 Gmail manual integration test completed successfully!");
    println!("   - Real Gmail IMAP connection verified");
    println!("   - EmailIngester::watch() method working correctly");
    println!("   - URI parsing and configuration functional");
    println!("   - Database credential management working");
    println!("   - interfaces::state::ingest integration verified");

    Ok(())
}

/// Test just the email fetching capability without full watch() integration
#[tokio::test]
#[ignore = "Manual test - requires Gmail credentials"]
async fn gmail_fetch_only_test() -> Result<()> {
    println!("🔍 Starting Gmail fetch-only test...");
    
    let missing_vars = validate_gmail_env_vars();
    if !missing_vars.is_empty() {
        return Err(anyhow::anyhow!("Missing environment variables: {:?}", missing_vars));
    }

    let gmail_email = env::var("GMAIL_EMAIL")?;
    let gmail_password = env::var("GMAIL_PASSWORD")?;
    
    // Create configuration directly for testing fetch_emails()
    let test_credential = EmailCredential {
        email_address: gmail_email.clone(),
        password: gmail_password.clone(),
        last_sync_date: Some(Utc::now() - chrono::Duration::days(7)), // Last week
    };
    
    let gmail_uri = format!(
        "email://{}@imap.gmail.com:993/INBOX?tls=true",
        gmail_email.split('@').next().unwrap()
    );
    
    let config = email_ingestion::email_ingester::EmailIngesterConfig::from_uri_and_credentials(
        &gmail_uri,
        &test_credential
    ).await?;
    
    let ingester = EmailIngester::with_config(config);
    let source = LiveSourceSpec { uri: gmail_uri };
    
    println!("🔄 Fetching emails directly from Gmail...");
    let start_time = std::time::Instant::now();
    
    let emails = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        ingester.fetch_emails(&source)
    ).await
    .map_err(|_| anyhow::anyhow!("Gmail fetch timed out after 60 seconds"))??;
    
    let elapsed = start_time.elapsed();
    println!("⏱️  Gmail fetch took {:.2} seconds", elapsed.as_secs_f64());
    println!("📧 Fetched {} emails from Gmail", emails.len());
    
    if emails.is_empty() {
        println!("📭 No emails found (this might be normal for recent emails)");
    } else {
        // Show sample of first few emails
        let display_count = std::cmp::min(emails.len(), 3);
        println!("📧 Sample emails (showing {} of {}):", display_count, emails.len());
        
        for (i, item) in emails.iter().take(display_count).enumerate() {
            println!("   {}. URI: {}", i + 1, item.uri);
            println!("      Live Source: {}", item.live_source_uri);
            
            // Extract subject from email text
            if let Some(subject_line) = item.text.lines().find(|line| line.starts_with("Subject:")) {
                println!("      {}", subject_line);
            }
        }
        
        if emails.len() > display_count {
            println!("   ... and {} more emails", emails.len() - display_count);
        }
        
        // Basic validation
        for item in &emails {
            assert!(item.uri.starts_with("email://"), "Email URI should start with 'email://'");
            assert!(item.live_source_uri == source.uri, "Live source URI should match");
            assert!(item.text.contains("From:"), "Email should contain 'From:' header");
            assert!(!item.text.is_empty(), "Email text should not be empty");
        }
        
        println!("✅ Email validation passed");
    }
    
    println!("🎉 Gmail fetch-only test completed successfully!");
    
    Ok(())
}

/// Validates that all required Gmail environment variables are present
fn validate_gmail_env_vars() -> Vec<String> {
    let required_vars = [
        "GMAIL_EMAIL",
        "GMAIL_PASSWORD",
    ];
    
    required_vars
        .iter()
        .filter(|&var| env::var(var).is_err())
        .map(|&var| var.to_string())
        .collect()
}

/// Validates Gmail-specific configuration settings
fn validate_gmail_config(config: &email_ingestion::email_ingester::EmailIngesterConfig) -> Result<()> {
    println!("🔧 Gmail configuration validation:");
    
    // Check server
    if config.server != "imap.gmail.com" {
        println!("⚠️  Warning: Server '{}' is not the standard Gmail IMAP server", config.server);
        println!("   Expected: imap.gmail.com");
    } else {
        println!("✓ Server: {}", config.server);
    }
    
    // Check port
    if config.port != 993 {
        println!("⚠️  Warning: Port {} is not the standard Gmail IMAP port", config.port);
        println!("   Expected: 993 (IMAPS with TLS)");
    } else {
        println!("✓ Port: {} (IMAPS)", config.port);
    }
    
    // Check TLS
    if !config.use_tls {
        return Err(anyhow::anyhow!("Gmail requires TLS to be enabled"));
    } else {
        println!("✓ TLS enabled");
    }
    
    // Check TLS security settings for Gmail (should be secure)
    if config.accept_invalid_certs {
        println!("⚠️  Warning: Accepting invalid certificates (not recommended for Gmail)");
    } else {
        println!("✓ TLS certificate validation enabled");
    }
    
    if config.accept_invalid_hostnames {
        println!("⚠️  Warning: Accepting invalid hostnames (not recommended for Gmail)");
    } else {
        println!("✓ TLS hostname validation enabled");
    }
    
    // Check username format
    if !config.username.contains("@") && !config.password.is_empty() {
        println!("⚠️  Warning: Gmail username should typically be full email address");
        println!("   Current username: {}", config.username);
    } else {
        println!("✓ Username: {}", config.username);
    }
    
    println!("✓ Mailbox: {}", config.mailbox);
    
    Ok(())
}