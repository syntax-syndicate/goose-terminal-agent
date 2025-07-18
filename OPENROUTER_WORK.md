# OpenRouter PKCE Authentication Implementation Plan

## Overview
Implement PKCE (Proof Key for Code Exchange) authentication flow for OpenRouter to obtain user-controlled API keys. The implementation will be added to the main `goose` crate and initially consumed by `goose-cli`.

## Architecture

### 1. Core Components (in main `goose` crate)

#### a. PKCE Flow Manager
- **Location**: `crates/goose/src/config/signup_openrouter/mod.rs`
- **Responsibilities**:
  - Generate code verifier (random string)
  - Generate code challenge (SHA-256 hash of verifier, base64url encoded)
  - Build authorization URL
  - Handle code exchange for API key

#### b. Local Web Server
- **Location**: `crates/goose/src/config/signup_openrouter/server.rs`
- **Port**: localhost:3000 (hardcoded, keeping it simple)
- **Responsibilities**:
  - Listen for OAuth callback
  - Extract authorization code from URL parameters
  - Display success message to user
  - Signal main process that code was received

### 2. Dependencies to Add
- `sha2` - For SHA-256 hashing
- `base64` - For base64url encoding
- `rand` - For generating secure random strings
- `tokio` - For async runtime (if not already present)
- `axum` - For lightweight web server (already used in project)
- `open` - For opening browser automatically

### 3. API Design

```rust
// In crates/goose/src/config/signup_openrouter/mod.rs

pub struct PkceAuthFlow {
    code_verifier: String,
    code_challenge: String,
    server_handle: Option<JoinHandle<Result<String, Error>>>,
}

impl PkceAuthFlow {
    /// Create a new PKCE flow with generated verifier and challenge
    pub fn new() -> Result<Self, Error>;
    
    /// Get the authorization URL to open in browser
    pub fn get_auth_url(&self) -> String;
    
    /// Start local server and wait for callback
    pub async fn start_server(&mut self) -> Result<(), Error>;
    
    /// Exchange authorization code for API key
    pub async fn exchange_code(&self, code: String) -> Result<String, Error>;
    
    /// Complete flow: open browser, wait for callback, exchange code
    pub async fn complete_flow(&mut self) -> Result<String, Error>;
}
```

### 4. Implementation Steps

1. **Step 1: Core PKCE Logic**
   - Generate 128-character random string for code_verifier
   - Calculate SHA-256 hash of verifier
   - Base64url encode the hash for code_challenge
   - Build auth URL with parameters

2. **Step 2: Local Web Server**
   - Create minimal HTTP server on localhost:3000
   - Single route handler for OAuth callback
   - Extract `code` parameter from query string
   - Return HTML page with success message
   - Use channel to communicate code back to main flow

3. **Step 3: Browser Integration**
   - Use `open` crate to launch default browser
   - Open authorization URL automatically
   - Handle case where browser fails to open

4. **Step 4: Code Exchange**
   - Make POST request to `https://openrouter.ai/api/v1/auth/keys`
   - Include code, code_verifier, and code_challenge_method
   - Parse response to extract API key

5. **Step 5: CLI Integration**
   - Add command to goose-cli (e.g., `goose auth openrouter`)
   - Print the API key to stdout (no storage for now)
   - Provide feedback during each step

### 5. Error Handling
- Network errors
- Server bind failures (port already in use)
- Invalid/expired authorization codes
- User cancellation
- Timeout (2-3 minutes for user to complete auth)

### 6. Security Considerations
- Use cryptographically secure random generator
- Implement proper PKCE S256 method
- Don't log sensitive values (codes, keys) except final println! of key

### 7. User Experience
1. User runs `goose auth openrouter`
2. Terminal shows: "Opening browser for authentication..."
3. Browser opens to OpenRouter auth page
4. User logs in and authorizes
5. Redirected to localhost:3000
6. Page shows: "Authentication successful! You can close this tab."
7. Terminal shows: "Authentication complete! Your API key is: sk-or-v1-..."

### 8. Testing Strategy
- Unit tests for PKCE challenge generation
- Integration test with mock server
- Manual testing with real OpenRouter flow

## Next Steps
1. Set up the module structure in the main goose crate
2. Implement core PKCE logic
3. Add web server functionality
4. Integrate with goose-cli
5. Test end-to-end flow

## Implementation Status

### ✅ Completed
1. **Module Structure**: Created `src/config/signup_openrouter/` with:
   - `mod.rs` - Main PKCE flow implementation
   - `server.rs` - Axum web server for OAuth callback
   - `tests.rs` - Unit tests for PKCE challenge generation

2. **Core PKCE Logic**: 
   - Generates 128-character random code verifier
   - Creates SHA-256 hash and base64url encodes for challenge
   - Builds proper auth URL with PKCE parameters

3. **Web Server**:
   - Simple Axum server on localhost:3000
   - Handles OAuth callback with code extraction
   - Shows success/error pages to user
   - Graceful shutdown after receiving code

4. **CLI Integration**:
   - Added `goose auth openrouter` command
   - Opens browser automatically
   - Displays progress messages
   - Prints API key on success

5. **Tests**:
   - Verifies PKCE challenge generation
   - Ensures unique verifiers for each flow
   - Tests URL formatting

### 📋 Usage

```bash
# Authenticate with OpenRouter
goose auth openrouter

# Output:
# Opening browser for authentication...
# Waiting for authentication callback...
# Authorization code received. Exchanging for API key...
# 
# Authentication complete! Your API key is:
# sk-or-v1-...
```

### 🔄 Next Steps
- Add key storage functionality when needed
- Consider adding refresh token support
- Add more robust error handling for network issues
