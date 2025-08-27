use anyhow::Result;
use reqwest;

/// Helper utilities for interacting with GreenMail server during tests
pub struct GreenMailHelper {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct TestEmail {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
}

impl GreenMailHelper {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Check if GreenMail server is running and accessible
    pub async fn is_server_running(&self) -> bool {
        // Check if the web interface is accessible (indicates server is running)
        match self.client.get(&self.base_url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Send a test email via GreenMail's SMTP interface using lettre
    pub async fn send_test_email(&self, email: &TestEmail) -> Result<()> {
        use lettre::transport::smtp::client::Tls;
        use lettre::{Message, SmtpTransport, Transport};
        
        // Create email message
        let message = Message::builder()
            .from(email.from.parse()?)
            .to(email.to.parse()?)
            .subject(&email.subject)
            .body(email.body.clone())?;

        // Create SMTP transport (no TLS for GreenMail)
        let mailer = SmtpTransport::builder_dangerous("localhost")
            .port(3025)
            .tls(Tls::None)
            .build();

        // Send the email
        match mailer.send(&message) {
            Ok(_) => {
                println!("✓ Email sent successfully: {}", email.subject);
                Ok(())
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to send email via SMTP: {}", e))
            }
        }
    }

    /// Clear all emails from GreenMail server
    pub async fn clear_all_emails(&self) -> Result<()> {
        let response = self
            .client
            .delete(&format!("{}/api/mail", self.base_url))
            .send()
            .await?;

        // Don't fail if clearing returns error - just log it
        if !response.status().is_success() {
            println!("Warning: Clear emails returned HTTP {}", response.status());
        }

        Ok(())
    }

    /// Reset GreenMail server state
    pub async fn reset_server(&self) -> Result<()> {
        // Try to clear emails, but don't fail if it doesn't work
        let _ = self.clear_all_emails().await;
        Ok(())
    }
}

/// Create a set of test emails for comprehensive testing
pub fn create_test_emails() -> Vec<TestEmail> {
    vec![
        TestEmail {
            from: "sender1@example.com".to_string(),
            to: "test@localhost".to_string(),
            subject: "Test Email 1".to_string(),
            body: "This is the first test email content.".to_string(),
        },
        TestEmail {
            from: "sender2@example.com".to_string(),
            to: "test@localhost".to_string(),
            subject: "Important Update 📧".to_string(),
            body: "This email contains unicode characters: 🚀 测试 中文\n\nMultiple paragraphs:\n- Item 1\n- Item 2\n\nBest regards!".to_string(),
        },
        TestEmail {
            from: "notifications@system.com".to_string(),
            to: "test@localhost".to_string(),
            subject: "System Alert".to_string(),
            body: "This is a system notification with special characters:\n\nStatus: ✅ OK\nTime: 2024-01-01 12:00:00 UTC\n\nEnd of message.".to_string(),
        },
    ]
}

/// Wait for GreenMail server to be ready
pub async fn wait_for_server_ready(timeout_seconds: u64) -> Result<()> {
    let helper = GreenMailHelper::new();
    let start = std::time::Instant::now();
    
    loop {
        if helper.is_server_running().await {
            return Ok(());
        }
        
        if start.elapsed().as_secs() > timeout_seconds {
            return Err(anyhow::anyhow!(
                "GreenMail server not ready after {} seconds. Make sure it's running with:\n\
                docker run -d --name greenmail-test \\\n\
                  -p 3025:3025 -p 3143:3143 -p 3993:3993 -p 8080:8080 \\\n\
                  -e GREENMAIL_OPTS=\"-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.auth.disabled -Dgreenmail.verbose\" \\\n\
                  greenmail/standalone:2.0.1",
                timeout_seconds
            ));
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}