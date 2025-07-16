use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "newrelic-apikeys-cli")]
#[command(about = "A CLI tool for interacting with New Relic's Nerdgraph API")]
#[command(version = "0.0.1")]
struct Cli {
    /// New Relic API key
    #[arg(short, long, env = "NEW_RELIC_API_KEY")]
    api_key: String,

    /// New Relic API endpoint (default: https://api.newrelic.com/graphql)
    #[arg(short, long, default_value = "https://api.newrelic.com/graphql")]
    endpoint: String,

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query API keys
    Query {
        /// Key type filter
        #[arg(short, long)]
        key_type: Option<String>,

        /// Key ID to search for
        #[arg(short = 'i', long)]
        key_id: Option<String>,
    },
    /// Create a new API key
    Create {
        /// Account ID
        #[arg(short, long)]
        account_id: String,

        /// Key type
        #[arg(short, long)]
        key_type: String,

        /// Key name
        #[arg(short, long)]
        name: String,

        /// Key notes/description
        #[arg(long)]
        notes: Option<String>,
    },
    /// Update an existing API key
    Update {
        /// Key ID
        #[arg(short, long)]
        key_id: String,

        /// New name
        #[arg(short, long)]
        name: Option<String>,

        /// New notes/description
        #[arg(long)]
        notes: Option<String>,
    },
    /// Delete an API key
    Delete {
        /// Key ID
        #[arg(short, long)]
        key_id: String,
    },
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
    locations: Option<Vec<Location>>,
    path: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Location {
    line: i32,
    column: i32,
}

struct NewRelicClient {
    client: Client,
    api_key: String,
    endpoint: String,
}

impl NewRelicClient {
    fn new(api_key: String, endpoint: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint,
        }
    }

    async fn execute_query(
        &self,
        query: &str,
        variables: Option<HashMap<String, serde_json::Value>>,
    ) -> anyhow::Result<serde_json::Value> {
        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("API-Key", &self.api_key)
            .json(&request)
            .send()
            .await?;

        let response_text = response.text().await?;
        let graphql_response: GraphQLResponse = serde_json::from_str(&response_text)?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            return Err(anyhow::anyhow!(
                "GraphQL errors: {}",
                error_messages.join(", ")
            ));
        }

        Ok(graphql_response.data.unwrap_or(serde_json::Value::Null))
    }
}

async fn query_api_keys(
    client: &NewRelicClient,
    key_type: Option<String>,
    key_id: Option<String>,
) -> anyhow::Result<()> {
    // Construct the GraphQL query
    // add key_type and key_id to the query if they are provided
    let query = r#"
    query($id: ID!, $keyType: ApiAccessKeyType!) {
        actor {
            apiAccess {
                key(
                    id: $id
                    keyType: $keyType
                ) {
                    key
                    name
                    notes
                    type
                }
            }
        }
    }"#;

    let mut variables = HashMap::new();
    if let (Some(key_id), Some(key_type)) = (key_id.clone(), key_type.clone()) {
        variables.insert("id".to_string(), serde_json::Value::String(key_id));
        variables.insert("keyType".to_string(), serde_json::Value::String(key_type));
    }

    let result = client.execute_query(query, Some(variables)).await?;
    //println!("{}", serde_json::to_string_pretty(&result)?);

    if let Some(key) = result
        .get("actor")
        .and_then(|a| a.get("apiAccess"))
        .and_then(|a| a.get("key"))
    {
        println!("");
        println!("API Key Details:");
        println!(
            "Key: {}",
            key.get("key")
                .unwrap_or(&serde_json::Value::String("N/A".to_string()))
        );
        println!(
            "Name: {}",
            key.get("name")
                .unwrap_or(&serde_json::Value::String("N/A".to_string()))
        );
        println!(
            "Type: {}",
            key.get("type")
                .unwrap_or(&serde_json::Value::String("N/A".to_string()))
        );
        println!(
            "Notes: {}",
            key.get("notes")
                .unwrap_or(&serde_json::Value::String("N/A".to_string()))
        );
    } else {
        println!("No API keys found or unable to retrieve keys");
    }

    Ok(())
}

async fn create_api_key(
    client: &NewRelicClient,
    account_id: String,
    key_type: String,
    name: String,
    notes: Option<String>,
) -> anyhow::Result<()> {
    let query = r#"
        mutation($accountId: Int!, $keyType: ApiAccessKeyType!, $name: String!, $notes: String) {
            apiAccessCreateKeys(keys: [{
                accountId: $accountId,
                keyType: $keyType,
                name: $name,
                notes: $notes
            }]) {
                createdKeys {
                    id
                    name
                    type
                    key
                    notes
                }
                errors {
                    message
                    type
                }
            }
        }
    "#;

    let mut variables = HashMap::new();
    variables.insert(
        "accountId".to_string(),
        serde_json::Value::String(account_id),
    );
    variables.insert("keyType".to_string(), serde_json::Value::String(key_type));
    variables.insert("name".to_string(), serde_json::Value::String(name));

    if let Some(notes) = notes {
        variables.insert("notes".to_string(), serde_json::Value::String(notes));
    }

    let result = client.execute_query(query, Some(variables)).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

async fn update_api_key(
    client: &NewRelicClient,
    key_id: String,
    name: Option<String>,
    notes: Option<String>,
) -> anyhow::Result<()> {
    let query = r#"
        mutation($keyId: String!, $name: String, $notes: String) {
            apiAccessUpdateKeys(keys: [{
                id: $keyId,
                name: $name,
                notes: $notes
            }]) {
                updatedKeys {
                    id
                    name
                    type
                    notes
                }
                errors {
                    message
                    type
                }
            }
        }
    "#;

    let mut variables = HashMap::new();
    variables.insert("keyId".to_string(), serde_json::Value::String(key_id));

    if let Some(name) = name {
        variables.insert("name".to_string(), serde_json::Value::String(name));
    }

    if let Some(notes) = notes {
        variables.insert("notes".to_string(), serde_json::Value::String(notes));
    }

    let result = client.execute_query(query, Some(variables)).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

async fn delete_api_key(client: &NewRelicClient, key_id: String) -> anyhow::Result<()> {
    let query = r#"
        mutation($keyId: String!) {
            apiAccessDeleteKeys(keys: {ingestKeyIds: $keyId}) {
                deletedKeys {
                    id
                }
                errors {
                    message
                    type
                }
            }
        }
    "#;

    let mut variables = HashMap::new();
    variables.insert("keyId".to_string(), serde_json::Value::String(key_id));

    let result = client.execute_query(query, Some(variables)).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Using endpoint: {}", cli.endpoint);
        println!("Output format: {}", cli.format);
    }

    let client = NewRelicClient::new(cli.api_key, cli.endpoint);

    match cli.command {
        Commands::Query { key_type, key_id } => {
            query_api_keys(&client, key_type, key_id).await?;
        }
        Commands::Create {
            account_id,
            key_type,
            name,
            notes,
        } => {
            create_api_key(&client, account_id, key_type, name, notes).await?;
        }
        Commands::Update {
            key_id,
            name,
            notes,
        } => {
            update_api_key(&client, key_id, name, notes).await?;
        }
        Commands::Delete { key_id } => {
            delete_api_key(&client, key_id).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_new_relic_client_creation() {
        let client = NewRelicClient::new(
            "test-api-key".to_string(),
            "https://api.newrelic.com/graphql".to_string(),
        );

        assert_eq!(client.api_key, "test-api-key");
        assert_eq!(client.endpoint, "https://api.newrelic.com/graphql");
    }

    // #[test]
    // fn test_graphql_request_serialization() {
    //     let mut variables = HashMap::new();
    //     variables.insert(
    //         "accountId".to_string(),
    //         serde_json::Value::String("123456".to_string()),
    //     );

    //     let request = GraphQLRequest {
    //         query: "query test { actor { account { id } } }".to_string(),
    //         variables: Some(variables),
    //     };

    //     let serialized = serde_json::to_string(&request).unwrap();
    //     assert!(serialized.contains("query test"));
    //     assert!(serialized.contains("accountId"));
    //     assert!(serialized.contains("123456"));
    // }

    #[test]
    fn test_graphql_error_deserialization() {
        let error_json = r#"
        {
            "errors": [
                {
                    "message": "Invalid API key",
                    "locations": [{"line": 1, "column": 1}],
                    "path": ["actor"]
                }
            ]
        }
        "#;

        let response: GraphQLResponse = serde_json::from_str(error_json).unwrap();
        assert!(response.errors.is_some());
        assert_eq!(response.errors.unwrap()[0].message, "Invalid API key");
    }

    #[test]
    fn test_graphql_success_deserialization() {
        let success_json = r#"
        {
            "data": {
                "actor": {
                    "account": {
                        "apiAccess": {
                            "keys": [
                                {
                                    "id": "key-123",
                                    "name": "Test Key",
                                    "type": "USER",
                                    "notes": "Test notes"
                                }
                            ]
                        }
                    }
                }
            }
        }
        "#;

        let response: GraphQLResponse = serde_json::from_str(success_json).unwrap();
        assert!(response.data.is_some());
        assert!(response.errors.is_none());

        let data = response.data.unwrap();
        let keys = data["actor"]["account"]["apiAccess"]["keys"]
            .as_array()
            .unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0]["name"], "Test Key");
        assert_eq!(keys[0]["type"], "USER");
    }

    #[test]
    fn test_key_id_filtering() {
        // Test data with multiple keys
        let test_data = serde_json::json!({
            "actor": {
                "apiAccess": {
                    "keys": [
                        {
                            "id": "key-123",
                            "name": "First Key",
                            "type": "USER",
                            "notes": "First key notes"
                        },
                        {
                            "id": "key-456",
                            "name": "Second Key",
                            "type": "INGEST",
                            "notes": "Second key notes"
                        }
                    ]
                }
            }
        });

        let keys = test_data["actor"]["apiAccess"]["keys"].as_array().unwrap();
        let mut filtered_keys = keys.clone();

        // Filter by key ID
        let key_id_filter = "key-123";
        filtered_keys.retain(|key| {
            key.get("id")
                .and_then(|id| id.as_str())
                .map_or(false, |id| id == key_id_filter)
        });

        assert_eq!(filtered_keys.len(), 1);
        assert_eq!(filtered_keys[0]["id"], "key-123");
        assert_eq!(filtered_keys[0]["name"], "First Key");
    }
}
