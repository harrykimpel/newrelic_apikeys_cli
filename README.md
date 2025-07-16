# New Relic API Keys CLI

A command-line tool for managing New Relic API keys through the Nerdgraph API.

## Installation

1. Make sure you have Rust installed (<https://rustup.rs/>)
2. Clone this repository
3. Build the project:

   ```bash
   cargo build --release
   ```

## Configuration

The CLI requires a New Relic User API key. You can provide it in two ways:

1. **Environment Variable** (recommended):

   ```bash
   export NEW_RELIC_API_KEY="your-api-key-here"
   ```

2. **Command Line Flag**:

   ```bash
   newrelic-apikeys-cli --api-key "your-api-key-here" [command]
   ```

## Usage

### Basic Commands

#### Query API Keys

```bash
# Query all API keys
newrelic-apikeys-cli query

# Query API keys for a specific account
newrelic-apikeys-cli query --account-id 123456

# Query API keys of a specific type
newrelic-apikeys-cli query --key-type USER

# Query for a specific key ID
newrelic-apikeys-cli query --key-id "12345678-1234-1234-1234-123456789012"

# Query with multiple filters
newrelic-apikeys-cli query --account-id 123456 --key-type INGEST --key-id "specific-key-id"
```

#### Create API Key

```bash
# Create a new API key
newrelic-apikeys-cli create --account-id 123456 --key-type USER --name "My API Key"

# Create with notes
newrelic-apikeys-cli create --account-id 123456 --key-type INGEST --name "Data Ingestion Key" --notes "For production data ingestion"
```

#### Update API Key

```bash
# Update key name
newrelic-apikeys-cli update --key-id "key-uuid" --name "New Name"

# Update notes
newrelic-apikeys-cli update --key-id "key-uuid" --notes "Updated description"

# Update both name and notes
newrelic-apikeys-cli update --key-id "key-uuid" --name "New Name" --notes "New description"
```

#### Delete API Key

```bash
# Delete an API key
newrelic-apikeys-cli delete --key-id "key-uuid"
```

### Global Options

- `--api-key, -a`: New Relic API key (can also be set via `NEW_RELIC_API_KEY` environment variable)
- `--endpoint, -e`: New Relic API endpoint (default: <https://api.newrelic.com/graphql>)
- `--format, -f`: Output format (default: json)
- `--verbose, -v`: Enable verbose output
- `--help, -h`: Show help information

### Examples

```bash
# Set up your API key
export NEW_RELIC_API_KEY="NRAK-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

# List all API keys with verbose output
newrelic-apikeys-cli -v query

# Search for a specific key by ID
newrelic-apikeys-cli query --key-id "12345678-1234-1234-1234-123456789012"

# Create a user API key
newrelic-apikeys-cli create -a 123456 -k USER -n "My User Key"

# Update a key's name
newrelic-apikeys-cli update -k "12345678-1234-1234-1234-123456789012" -n "Updated Key Name"

# Delete a key
newrelic-apikeys-cli delete -k "12345678-1234-1234-1234-123456789012"
```

## Key Types

Common New Relic API key types:

- `USER`: User API keys for querying data
- `INGEST`: Ingest API keys for sending data to New Relic
- `BROWSER`: Browser monitoring keys
- `MOBILE`: Mobile monitoring keys

## Error Handling

The CLI provides detailed error messages for common issues:

- Invalid API key
- Network connectivity problems
- GraphQL query errors
- Missing required parameters

## Development

### Running Tests

```bash
cargo test
```

### Building for Release

```bash
cargo build --release
```

The binary will be available at `target/release/newrelic-apikeys-cli`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.
