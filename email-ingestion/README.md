# Email Ingestion Integration Tests

This directory contains a complete email ingestion testing framework that integrates with the summarena interfaces.

## Overview

The integration test demonstrates:
1. **GreenMail Email Server**: Uses a real IMAP/SMTP server for comprehensive testing
2. **Email Ingestion**: Uses the `Ingester` interface to fetch emails via real IMAP protocol
3. **Content Processing**: Converts emails to `InputItem` structs as required by the interface
4. **Database Storage**: Stores email credentials and sync state in database
5. **State Integration**: Emails are processed through `interfaces::state::ingest`

## Running the Tests

### Prerequisites

1. **GreenMail Test Server**: Start the GreenMail Docker container before running tests
   ```bash
   # Start GreenMail container with IMAP and SMTP support
   docker run -d --name greenmail-test \
      -p 3025:3025 -p 3143:3143 -p 3993:3993 -p 8080:8080 \
      -e GREENMAIL_OPTS="-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.auth.disabled -Dgreenmail.verbose" \
      greenmail/standalone:2.1.2
   ```

2. **Postgres Database**: Set up a test database for email credentials
   ```bash
   # Using Docker
   docker run -d --name test-postgres \
     -e POSTGRES_PASSWORD=password \
     -e POSTGRES_DB=test_email_ingestion \
     -p 5432:5432 postgres:latest
   
   # Or set custom database URL
   export TEST_DATABASE_URL="postgresql://username:password@localhost:5432/your_test_db"
   ```

3. **Run the integration tests**:
   ```bash
   cd email-ingestion
   cargo test --test greenmail_integration_test -- --nocapture
   ```

4. **Stop containers when done**:
   ```bash
   docker stop greenmail-test test-postgres
   docker rm greenmail-test test-postgres
   ```

## Test Structure

### `test_email_ingestion_with_real_imap()`
The main integration test performs these steps:

1. **Sets up Test Environment**:
   - Assumes GreenMail container is running
   - Sets up Postgres database with email credentials
   - Sends test emails via GreenMail SMTP API

2. **Tests Real IMAP Protocol**:
   - Creates `LiveSourceSpec` with GreenMail IMAP URI: `email://test@localhost:3143/INBOX?tls=false`
   - Calls `EmailIngester::watch()` method (the main entry point)
   - Fetches emails via actual IMAP protocol

3. **Verifies End-to-End Flow**:
   - Emails are processed through `interfaces::state::ingest`
   - Database last sync time is updated correctly
   - URI parsing works with real server connections

4. **Validates Content and Behavior**:
   - Email content is accurately fetched and parsed
   - Error handling works correctly
   - Edge cases are handled properly

### GreenMail Ports
- **SMTP**: 3025 (for sending test emails)
- **IMAP**: 3143 (for email retrieval)  
- **IMAPS**: 3993 (SSL/TLS IMAP)
- **API**: 8080 (HTTP API for management)

## Key Components

- **GreenMail Server**: Real IMAP/SMTP server for authentic protocol testing
- **EmailIngester**: Implements the `Ingester` trait with URI-based configuration
- **EmailDatabase**: Handles credential storage and sync state management
- **Integration Tests**: End-to-end validation using real email protocols

## Interface Compliance

The implementation properly uses the summarena interfaces:
- `LiveSourceSpec`: Contains email URI with server, port, mailbox, and TLS settings
- `Ingester::watch()`: Main entry point that fetches emails and calls `interfaces::state::ingest`
- `InputItem`: Email content with URI, live_source_uri, text, and vision fields
- `WatchRest`: Specifies polling wait time (30 seconds on success, 60s on error)

## Expected Output

When tests pass, you'll see:
```
✓ GreenMail server is running and accessible
✓ Test emails sent via SMTP
✓ Email credentials stored in database
✓ Successfully fetched 3 emails via real IMAP
✓ All emails processed through interfaces::state::ingest
✓ Database sync time updated correctly
🎉 All integration tests passed!
```

## Troubleshooting

- **Connection refused**: Make sure GreenMail container is running on the expected ports
- **Database errors**: Verify Postgres is running and accessible
- **Authentication errors**: GreenMail is configured with auth disabled for testing