use interfaces::defs::{LiveSourceSpec, Ingester, InputItem, WatchRest};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::future::Future;
use url::Url;
use crate::database::{EmailDatabase, EmailCredential};

#[derive(Clone)]
pub struct EmailIngesterConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub mailbox: String,
    pub use_tls: bool,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
    pub last_sync_date: Option<DateTime<Utc>>,
}

impl EmailIngesterConfig {
    /// Parse email configuration from URI and database credentials
    /// Expected URI format: email://username@server:port/mailbox?tls=true
    pub async fn from_uri_and_credentials(uri: &str, credentials: &EmailCredential) -> Result<Self> {
        let parsed_uri = Url::parse(uri)
            .map_err(|e| anyhow::anyhow!("Invalid email URI '{}': {}", uri, e))?;
        
        // Validate scheme
        if parsed_uri.scheme() != "email" {
            return Err(anyhow::anyhow!("URI must use 'email://' scheme, got: {}", parsed_uri.scheme()));
        }
        
        // Extract server and port
        let server = parsed_uri.host_str()
            .ok_or_else(|| anyhow::anyhow!("No server specified in URI: {}", uri))?
            .to_string();
        
        let port = parsed_uri.port().unwrap_or(993); // Default to IMAPS port
        
        // Extract username from URI or use email address
        let username = {
            let user = parsed_uri.username();
            if !user.is_empty() {
                user.to_string()
            } else {
                credentials.email_address.clone()
            }
        };
        
        // Extract mailbox from path (default to INBOX)
        let mailbox = {
            let path = parsed_uri.path().trim_start_matches('/');
            if path.is_empty() {
                "INBOX".to_string()
            } else {
                path.to_string()
            }
        };
        
        // Extract TLS setting from query parameters (default to true)
        let use_tls = parsed_uri.query_pairs()
            .find(|(key, _)| key == "tls")
            .map(|(_, value)| value.parse().unwrap_or(true))
            .unwrap_or(true);
        
        // Extract TLS trust settings from query parameters (default to false for security)
        // Should only be true for local testing purposes
        let accept_invalid_certs = parsed_uri.query_pairs()
            .find(|(key, _)| key == "accept_invalid_certs")
            .map(|(_, value)| value.parse().unwrap_or(false))
            .unwrap_or(false);
            
        let accept_invalid_hostnames = parsed_uri.query_pairs()
            .find(|(key, _)| key == "accept_invalid_hostnames")
            .map(|(_, value)| value.parse().unwrap_or(false))
            .unwrap_or(false);
        
        Ok(Self {
            server,
            port,
            username,
            password: credentials.password.clone(),
            mailbox,
            use_tls,
            accept_invalid_certs,
            accept_invalid_hostnames,
            last_sync_date: credentials.last_sync_date,
        })
    }
}

pub struct EmailIngester {
    config: Option<EmailIngesterConfig>,
}

impl EmailIngester {
    pub fn new() -> Self {
        Self {
            config: None,
        }
    }

    pub fn with_config(config: EmailIngesterConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    pub fn with_config_and_last_sync(mut config: EmailIngesterConfig, last_sync_date: DateTime<Utc>) -> Self {
        config.last_sync_date = Some(last_sync_date);
        Self {
            config: Some(config),
        }
    }


    pub async fn fetch_emails(&self, source: &LiveSourceSpec) -> Result<Vec<InputItem>> {
        let config = match &self.config {
            Some(cfg) => cfg,
            None => return Err(anyhow::anyhow!("No email configuration provided. Use EmailIngester::with_config() or set environment variables.")),
        };

        self.fetch_from_imap_server(config, source).await
    }

    async fn fetch_from_imap_server(&self, config: &EmailIngesterConfig, source: &LiveSourceSpec) -> Result<Vec<InputItem>> {
        let _domain = format!("{}:{}", config.server, config.port);
        
        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .danger_accept_invalid_hostnames(config.accept_invalid_hostnames)
            .build()?;

        let client = if config.use_tls {
            imap::connect((config.server.as_str(), config.port), &config.server, &tls)?
        } else {
            imap::connect_starttls((config.server.as_str(), config.port), &config.server, &tls)?
        };

        let mut imap_session = client.login("test", "testpass")
            .map_err(|e| anyhow::anyhow!("Login failed: {:?}", e))?;
        imap_session.select(&config.mailbox)?;

        let mut input_items = Vec::new();

        // Use SEARCH to find relevant emails based on date criteria
        let search_criteria = if let Some(last_sync) = &config.last_sync_date {
            // Format date for IMAP SINCE command (dd-MMM-yyyy format)
            let since_date = last_sync.format("%d-%b-%Y").to_string();
            format!("SINCE {}", since_date)
        } else {
            // If no date provided, get recent emails (last 100)
            "ALL".to_string()
        };

        let message_ids = imap_session.search(&search_criteria)?;
        
        // Limit to avoid overwhelming memory for large mailboxes  
        let mut limited_ids: Vec<u32> = message_ids.into_iter().collect();
        if limited_ids.len() > 100 {
            // Sort to get highest IDs (most recent) and take last 100
            limited_ids.sort();
            limited_ids = limited_ids.into_iter().rev().take(100).collect();
        }

        if limited_ids.is_empty() {
            imap_session.logout()?;
            return Ok(input_items);
        }

        // Convert IDs to sequence set for fetch
        let sequence_set = if limited_ids.len() == 1 {
            limited_ids[0].to_string()
        } else {
            format!("{}:{}", limited_ids.iter().min().unwrap(), limited_ids.iter().max().unwrap())
        };

        let messages = imap_session.fetch(&sequence_set, "RFC822")?;
        
        for message in messages.iter() {
            if let Some(body) = message.body() {
                let parsed = mail_parser::MessageParser::default().parse(body)
                    .ok_or_else(|| anyhow::anyhow!("Failed to parse email"))?;
                
                let from = parsed.from()
                    .and_then(|addrs| addrs.first())
                    .and_then(|addr| addr.address.as_ref())
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                let to = parsed.to()
                    .and_then(|addrs| addrs.first())
                    .and_then(|addr| addr.address.as_ref())
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                let subject = parsed.subject()
                    .unwrap_or("(No Subject)");
                
                let body_text = parsed.body_text(0)
                    .unwrap_or_else(|| parsed.body_html(0).unwrap_or(std::borrow::Cow::Borrowed("")));
                
                let email_id = format!("{}_{}", message.uid.unwrap_or(0), 
                    parsed.message_id().unwrap_or("unknown"));
                
                input_items.push(InputItem {
                    uri: format!("email://{}", email_id),
                    live_source_uri: source.uri.clone(),
                    text: format!(
                        "From: {}\nTo: {}\nSubject: {}\n\n{}", 
                        from, to, subject, body_text
                    ),
                    vision: None,
                });
            }
        }

        imap_session.logout()?;
        Ok(input_items)
    }
}

impl Ingester for EmailIngester {
    fn watch(source: &LiveSourceSpec) -> impl Future<Output = WatchRest> {
        async move {
            // Extract email address from URI
            let email_address = match extract_email_address(&source.uri) {
                Ok(addr) => addr,
                Err(e) => {
                    eprintln!("Failed to parse email address from URI '{}': {}", source.uri, e);
                    return WatchRest {
                        wait_at_least_ms: 300000, // Wait 5 minutes on parse error
                    };
                }
            };
            
            // Connect to database and get credentials
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test_email_ingestion".to_string());
            let db = match EmailDatabase::new(&database_url).await {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("Failed to connect to database: {}", e);
                    return WatchRest {
                        wait_at_least_ms: 60000, // Wait 1 minute on DB error
                    };
                }
            };
            
            let credentials = match db.get_credentials(&email_address).await {
                Ok(Some(creds)) => creds,
                Ok(None) => {
                    eprintln!("No credentials found for email address: {}", email_address);
                    return WatchRest {
                        wait_at_least_ms: 300000, // Wait 5 minutes if no credentials
                    };
                }
                Err(e) => {
                    eprintln!("Database error getting credentials for {}: {}", email_address, e);
                    return WatchRest {
                        wait_at_least_ms: 60000, // Wait 1 minute on DB error
                    };
                }
            };
            
            // Build configuration from URI and credentials
            let config = match EmailIngesterConfig::from_uri_and_credentials(&source.uri, &credentials).await {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Failed to build config from URI '{}': {}", source.uri, e);
                    return WatchRest {
                        wait_at_least_ms: 300000, // Wait 5 minutes on config error
                    };
                }
            };

            let ingester = EmailIngester::with_config(config);
            
            match ingester.fetch_emails(source).await {
                Ok(emails) => {
                    let email_count = emails.len();
                    
                    // Process each email through the state system
                    for email in emails {
                        interfaces::state::ingest(&email).await;
                    }
                    
                    // Update last sync time in database
                    let now = Utc::now();
                    if let Err(e) = db.update_last_sync(&email_address, now).await {
                        eprintln!("Failed to update last sync time for {}: {}", email_address, e);
                    }
                    
                    println!("Successfully ingested {} emails for {}", email_count, email_address);
                    
                    WatchRest {
                        wait_at_least_ms: 30000, // Check again in 30 seconds
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch emails for {}: {}", email_address, e);
                    WatchRest {
                        wait_at_least_ms: 60000, // Wait longer on error
                    }
                }
            }
        }
    }
}

/// Extract email address from URI
/// Expected format: email://username@server:port/mailbox?tls=true
fn extract_email_address(uri: &str) -> Result<String> {
    let parsed = Url::parse(uri)
        .map_err(|e| anyhow::anyhow!("Invalid URI '{}': {}", uri, e))?;
    
    // If username contains @, it's likely a full email address
    let username = parsed.username();
    if username.contains('@') {
        return Ok(username.to_string());
    }
    
    // Otherwise, construct email from username@host
    let host = parsed.host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in URI: {}", uri))?;
    
    if username.is_empty() {
        return Err(anyhow::anyhow!("No username in URI: {}", uri));
    }
    
    Ok(format!("{}@{}", username, host))
}