use twitter_ingestion::{TwitterIngester, TwitterIngesterConfig};
use chrono::{DateTime, Utc, Duration};
use std::env;
use http::StatusCode;

#[tokio::test]
async fn test_fetch_tweets_without_config_fails() {
    let ingester = TwitterIngester::new();
    let source = interfaces::defs::LiveSourceSpec {
        uri: "twitter://user/dawnsongtweets".to_string(),
    };
    
    let result = ingester.fetch_tweets(&source).await;
    match result {
        Err(e) => assert!(e.to_string().contains("No Twitter configuration provided")),
        Ok(_) => panic!("Expected error but got success"),
    }
}

#[tokio::test]
async fn test_fetch_tweets_with_config_but_invalid_token() {
    // Remove any existing token for this test
    env::remove_var("TWITTER_BEARER_TOKEN");
    
    let handle = "dawnsongtweets".to_string();
    let config = TwitterIngesterConfig::new(vec![handle], "fake_token".to_string());
    let ingester = twitter_ingestion::TwitterIngester::<twitter_ingestion::ReqwestClient>::with_config(config);
    
    let source = interfaces::defs::LiveSourceSpec {
        uri: "twitter://user/dawnsongtweets".to_string(),
    };
    
    // With invalid token, the API now gracefully handles failures and returns 0 tweets
    let result = ingester.fetch_tweets(&source).await;
    match result {
        Ok(tweets) => {
            // Should return 0 tweets due to API error being handled gracefully
            assert_eq!(tweets.len(), 0);
        },
        Err(_) => panic!("Expected success with 0 tweets but got error"),
    }
}

#[test]
fn test_date_filtering_query_params() {
    let handle = "dawnsongtweets".to_string();
    let bearer_token = "fake_token".to_string();
    let last_sync_date = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.000Z")
        .unwrap()
        .with_timezone(&Utc);
    
    let config = TwitterIngesterConfig::new_with_last_sync_date(vec![handle.clone()], bearer_token, Some(last_sync_date));
    
    // Verify the date is stored correctly
    assert_eq!(config.last_sync_date, Some(last_sync_date));
    
    // Test date formatting (this is what would be sent to Twitter API)
    let formatted_date = last_sync_date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    assert_eq!(formatted_date, "2024-01-15T10:30:45.000Z");
}

#[test]
fn test_date_filtering_with_different_timezones() {
    let handle = "dawnsongtweets".to_string();
    let bearer_token = "fake_token".to_string();
    
    // Test with different timezone input
    let pst_date = DateTime::parse_from_rfc3339("2024-01-15T02:30:45-08:00")
        .unwrap()
        .with_timezone(&Utc);
    
    let config = TwitterIngesterConfig::new_with_last_sync_date(vec![handle.clone()], bearer_token, Some(pst_date));
    
    // Should be converted to UTC
    let expected_utc = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.000Z")
        .unwrap()
        .with_timezone(&Utc);
    
    assert_eq!(config.last_sync_date, Some(expected_utc));
}

#[test]
fn test_config_without_last_sync_date() {
    let handle = "dawnsongtweets".to_string();
    let bearer_token = "fake_token".to_string();
    
    let config = TwitterIngesterConfig::new(vec![handle.clone()], bearer_token.clone());
    
    assert_eq!(config.twitter_handles[0], handle);
    assert_eq!(config.bearer_token, bearer_token);
    assert!(config.last_sync_date.is_none());
}

#[test]
fn test_with_config_and_last_sync_overwrites_date() {
    let handle = "dawnsongtweets".to_string();
    let bearer_token = "fake_token".to_string();
    let initial_date = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.000Z")
        .unwrap()
        .with_timezone(&Utc);
    let new_date = DateTime::parse_from_rfc3339("2024-01-16T12:00:00.000Z")
        .unwrap()
        .with_timezone(&Utc);
    
    let config = TwitterIngesterConfig::new_with_last_sync_date(vec![handle.clone()], bearer_token, Some(initial_date));
    let ingester = twitter_ingestion::TwitterIngester::<twitter_ingestion::ReqwestClient>::with_config_and_last_sync(config, new_date);
    
    assert_eq!(ingester.config().unwrap().last_sync_date, Some(new_date));
}

// Test edge cases for date handling
#[test]
fn test_date_edge_cases() {
    let handle = "test".to_string();
    let bearer_token = "fake_token".to_string();
    
    // Test with very recent date (1 minute ago)
    let recent_date = Utc::now() - Duration::minutes(1);
    let config = TwitterIngesterConfig::new_with_last_sync_date(vec![handle.clone()], bearer_token.clone(), Some(recent_date));
    assert!(config.last_sync_date.is_some());
    
    // Test with very old date (1 year ago)
    let old_date = Utc::now() - Duration::days(365);
    let config = TwitterIngesterConfig::new_with_last_sync_date(vec![handle.clone()], bearer_token, Some(old_date));
    assert!(config.last_sync_date.is_some());
}


// Tests with mocked HTTP client
mod mock_client;

mod mock_tests {
    use super::*;
    use crate::mock_client::MockHttpClient;

    #[tokio::test]
    async fn test_fetch_tweets_with_real_api_response() {
        // Load the real Twitter API response
        let tweets_response = include_str!("response.json");
        let user_response = r#"{"data": {"id": "535136727", "name": "Dawn Song", "username": "dawnsongtweets"}}"#;

        let mock_client = MockHttpClient::new()
            .add_json_response("https://api.twitter.com/2/users/by/username/dawnsongtweets", user_response)
            .add_json_response("https://api.twitter.com/2/users/535136727/tweets", tweets_response);

        let config = TwitterIngesterConfig::new(
            vec!["dawnsongtweets".to_string()],
            "fake_token".to_string(),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/dawnsongtweets".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        assert!(result.is_ok());
        
        let tweets = result.unwrap();
        assert_eq!(tweets.len(), 5);
        
        // Verify first tweet details
        let first_tweet = &tweets[0];
        assert_eq!(first_tweet.uri, "twitter://tweet/1953166647994532286");
        assert!(first_tweet.text.contains("@dawnsongtweets"));
        assert!(first_tweet.text.contains("2025-08-06T18:49:51.000Z"));
        assert!(first_tweet.text.contains("Special thanks to our amazing sponsors"));
        assert!(first_tweet.text.contains("Likes: 10"));
        assert!(first_tweet.text.contains("Retweets: 0"));
        assert!(first_tweet.text.contains("https://twitter.com/dawnsongtweets/status/1953166647994532286"));

        // Verify last tweet details  
        let last_tweet = &tweets[4];
        assert_eq!(last_tweet.uri, "twitter://tweet/1953166634883137621");
        assert!(last_tweet.text.contains("Still buzzing from the incredible #AgenticAI Summit"));
        assert!(last_tweet.text.contains("Likes: 115"));
        assert!(last_tweet.text.contains("Retweets: 20"));
        
        // Verify all tweets have proper structure
        for tweet in &tweets {
            assert!(tweet.uri.starts_with("twitter://tweet/"));
            assert_eq!(tweet.live_source_uri, "twitter://user/dawnsongtweets");
            assert!(tweet.text.contains("@dawnsongtweets"));
            assert!(tweet.text.contains("https://twitter.com/dawnsongtweets/status/"));
            assert!(tweet.text.contains("Likes:"));
        }
    }

    #[tokio::test]
    async fn test_fetch_tweets_with_real_response_and_date_filtering() {
        // Load the real Twitter API response
        let tweets_response = include_str!("response.json");
        let user_response = r#"{"data": {"id": "535136727", "name": "Dawn Song", "username": "dawnsongtweets"}}"#;

        let mock_client = MockHttpClient::new()
            .add_json_response("https://api.twitter.com/2/users/by/username/dawnsongtweets", user_response)
            .add_json_response("https://api.twitter.com/2/users/535136727/tweets", tweets_response);

        // Create config with last sync date (should be included in API request)
        let last_sync_date = DateTime::parse_from_rfc3339("2025-08-06T18:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc);
        let config = TwitterIngesterConfig::new_with_last_sync_date(
            vec!["dawnsongtweets".to_string()],
            "fake_token".to_string(),
            Some(last_sync_date),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/dawnsongtweets".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        assert!(result.is_ok());
        
        let tweets = result.unwrap();
        assert_eq!(tweets.len(), 5);
        
        // All tweets in response.json are after our last sync date, so they should all be included
        // Verify the tweets are from the AgenticAI Summit
        assert!(tweets.iter().any(|t| t.text.contains("#AgenticAI Summit")));
        assert!(tweets.iter().any(|t| t.text.contains("Special thanks to our amazing sponsors")));
        
        // Verify metrics parsing for the viral tweet
        let viral_tweet = tweets.iter().find(|t| t.text.contains("Still buzzing")).unwrap();
        assert!(viral_tweet.text.contains("Likes: 115"));
        assert!(viral_tweet.text.contains("Retweets: 20"));
        assert!(viral_tweet.text.contains("Replies: 7"));
        assert!(viral_tweet.text.contains("Quotes: 6"));
    }

    #[tokio::test]
    async fn test_fetch_tweets_with_mock_success() {
        // Create mock responses
        let user_response = r#"{"data": {"id": "12345", "name": "Test User", "username": "testuser"}}"#;
        let tweets_response = r#"{
            "data": [
                {
                    "id": "1001",
                    "text": "Hello world!",
                    "created_at": "2024-01-15T10:30:45.000Z",
                    "author_id": "12345",
                    "public_metrics": {
                        "like_count": 42,
                        "retweet_count": 12,
                        "reply_count": 8,
                        "quote_count": 3
                    }
                }
            ]
        }"#;

        let mock_client = MockHttpClient::new()
            .add_json_response("https://api.twitter.com/2/users/by/username/testuser", user_response)
            .add_json_response("https://api.twitter.com/2/users/12345/tweets", tweets_response);

        let config = TwitterIngesterConfig::new(
            vec!["testuser".to_string()],
            "fake_token".to_string(),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/testuser".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        assert!(result.is_ok());
        
        let tweets = result.unwrap();
        assert_eq!(tweets.len(), 1);
        
        let tweet = &tweets[0];
        assert_eq!(tweet.uri, "twitter://tweet/1001");
        assert!(tweet.text.contains("Hello world!"));
        assert!(tweet.text.contains("@testuser"));
        assert!(tweet.text.contains("Likes: 42"));
    }

    #[tokio::test]
    async fn test_fetch_tweets_with_mock_user_not_found() {
        let mock_client = MockHttpClient::new()
            .add_error_response("https://api.twitter.com/2/users/by/username/nonexistent", StatusCode::NOT_FOUND.as_u16());

        let config = TwitterIngesterConfig::new(
            vec!["nonexistent".to_string()],
            "fake_token".to_string(),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/nonexistent".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        // Should return 0 tweets since the user doesn't exist (API handles errors gracefully)
        match result {
            Ok(tweets) => assert_eq!(tweets.len(), 0),
            Err(_) => panic!("Expected success with 0 tweets but got error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_tweets_with_mock_unauthorized() {
        let mock_client = MockHttpClient::new()
            .add_error_response("https://api.twitter.com/2/users/by/username/testuser", StatusCode::UNAUTHORIZED.as_u16());

        let config = TwitterIngesterConfig::new(
            vec!["testuser".to_string()],
            "invalid_token".to_string(),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/testuser".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        // Should return 0 tweets due to invalid token (API handles errors gracefully)
        match result {
            Ok(tweets) => assert_eq!(tweets.len(), 0),
            Err(_) => panic!("Expected success with 0 tweets but got error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_tweets_with_mock_multiple_handles() {
        let user1_response = r#"{"data": {"id": "12345", "name": "User One", "username": "user1"}}"#;
        let user2_response = r#"{"data": {"id": "67890", "name": "User Two", "username": "user2"}}"#;
        let tweets1_response = r#"{
            "data": [
                {
                    "id": "1001",
                    "text": "Tweet from user1",
                    "created_at": "2024-01-15T10:30:45.000Z",
                    "author_id": "12345"
                }
            ]
        }"#;
        let tweets2_response = r#"{
            "data": [
                {
                    "id": "2001",
                    "text": "Tweet from user2",
                    "created_at": "2024-01-15T11:00:00.000Z",
                    "author_id": "67890"
                }
            ]
        }"#;

        let mock_client = MockHttpClient::new()
            .add_json_response("https://api.twitter.com/2/users/by/username/user1", user1_response)
            .add_json_response("https://api.twitter.com/2/users/by/username/user2", user2_response)
            .add_json_response("https://api.twitter.com/2/users/12345/tweets", tweets1_response)
            .add_json_response("https://api.twitter.com/2/users/67890/tweets", tweets2_response);

        let config = TwitterIngesterConfig::new(
            vec!["user1".to_string()],
            "fake_token".to_string(),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/user1".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        assert!(result.is_ok());
        
        let tweets = result.unwrap();
        assert_eq!(tweets.len(), 1);
        
        // Check that we got tweet from user1
        let tweet_texts: Vec<&str> = tweets.iter().map(|t| &*t.text).collect();
        assert!(tweet_texts.iter().any(|text| text.contains("Tweet from user1")));
    }

    #[tokio::test]
    async fn test_fetch_tweets_with_mock_empty_timeline() {
        let user_response = r#"{"data": {"id": "12345", "name": "Test User", "username": "testuser"}}"#;
        let empty_tweets_response = r#"{"data": []}"#;

        let mock_client = MockHttpClient::new()
            .add_json_response("https://api.twitter.com/2/users/by/username/testuser", user_response)
            .add_json_response("https://api.twitter.com/2/users/12345/tweets", empty_tweets_response);

        let config = TwitterIngesterConfig::new(
            vec!["testuser".to_string()],
            "fake_token".to_string(),
        );

        let ingester = TwitterIngester::with_config_and_client(config, mock_client);
        let source = interfaces::defs::LiveSourceSpec {
            uri: "twitter://user/testuser".to_string(),
        };

        let result = ingester.fetch_tweets(&source).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
