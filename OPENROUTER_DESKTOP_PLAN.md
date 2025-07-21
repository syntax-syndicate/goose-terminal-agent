# Desktop App OpenRouter Authentication Integration Plan

## Overview
Integrate the existing OpenRouter PKCE authentication flow from the goose crate into the desktop app, triggered automatically by `GOOSE_STARTUP=openrouter` environment variable.

## Implementation Status
✅ Backend implementation complete
✅ Frontend implementation complete
❓ Integration testing in progress

## Debugging Notes (2025-07-18)

### Current Issue
The `GOOSE_STARTUP=openrouter` environment variable is set but not triggering the OpenRouter setup flow.

### What We've Implemented
1. **Backend (goose-server)**:
   - Added `/setup/openrouter/start` endpoint that wraps the existing PKCE flow
   - The endpoint is working and ready to be called

2. **Frontend**:
   - Modified `ProviderGuard.tsx` to check for `GOOSE_STARTUP=openrouter`
   - Created `SetupModal.tsx` for UI feedback
   - Created `openRouterSetup.ts` utility to call the backend

3. **Main Process**:
   - Added `GOOSE_STARTUP` to `appConfig` in `main.ts`
   - Config is passed through `additionalArguments` to the renderer

### Debugging Steps Added
1. Console logging in main.ts to verify env var is read:
   ```typescript
   console.log('[Main] Environment GOOSE_STARTUP:', process.env.GOOSE_STARTUP);
   console.log('[Main] appConfig GOOSE_STARTUP:', appConfig.GOOSE_STARTUP);
   ```

2. Console logging in ProviderGuard to trace the flow:
   ```typescript
   console.log('ProviderGuard - Full config:', config);
   console.log('ProviderGuard - GOOSE_STARTUP:', config.GOOSE_STARTUP);
   ```

### Potential Issues to Investigate

1. **Config Not Reaching Renderer Process**:
   - The config is passed via `additionalArguments` in BrowserWindow creation
   - It's parsed in `preload.ts` and exposed via `window.electron.getConfig()`
   - Need to verify this chain is working correctly

2. **Timing Issue**:
   - ProviderGuard runs early in the app lifecycle
   - If config isn't available yet, it might not see the env var

3. **Existing Provider Configuration**:
   - If user already has a provider configured, the setup won't trigger
   - Check: `~/.config/goose/config.json` or platform-specific config location

4. **Environment Variable Not Set Correctly**:
   - Verify with: `echo $GOOSE_STARTUP` before running
   - Run with: `GOOSE_STARTUP=openrouter npm run start-gui`

### Possible Solutions

1. **Add getEnv to Electron API**:
   - Currently missing from preload.ts
   - Would allow direct env var access: `window.electron.getEnv('GOOSE_STARTUP')`

2. **Pass Through startGoosed**:
   - The `startGoosed` function might need to pass GOOSE_STARTUP to the backend process
   - Currently only passes `GOOSE_SCHEDULER_TYPE`

3. **Alternative Approach - URL Parameter**:
   - Could pass as URL param when creating window
   - More reliable than env var passing through multiple processes

4. **Config Storage Check**:
   - Add a check to clear/ignore existing provider config when GOOSE_STARTUP is set
   - Or add a "force setup" mode

### Test Script Created
- `test-openrouter-startup.sh` - Runs the app with the env var set

### Next Steps
1. Run the app with debugging and check all console logs
2. Verify the config object in the renderer process
3. Check if there's an existing provider configuration interfering
4. Consider implementing one of the alternative solutions if needed

## Architecture

### 1. **Reuse Existing PKCE Implementation**
- The goose crate already has full PKCE flow in `src/config/signup_openrouter/`
- It handles browser opening, local server (port 3000), and token exchange
- Automatically configures provider and models

### 2. **Backend (goose-server) - Simple Wrapper**

#### Add Setup Route
```rust
// crates/goose-server/src/routes/setup.rs
use goose::config::signup_openrouter::OpenRouterAuth;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SetupResponse {
    pub success: bool,
    pub message: String,
}

pub fn setup_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/setup/openrouter/start", post(start_openrouter_setup))
}

async fn start_openrouter_setup() -> Result<Json<SetupResponse>, StatusCode> {
    // Run the existing PKCE flow
    let mut auth_flow = OpenRouterAuth::new()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match auth_flow.complete_flow().await {
        Ok(api_key) => {
            // The complete_flow already handles all configuration
            // (storing key, setting provider, models, etc.)
            Ok(Json(SetupResponse {
                success: true,
                message: "OpenRouter setup completed successfully".to_string(),
            }))
        }
        Err(e) => {
            Ok(Json(SetupResponse {
                success: false,
                message: format!("Setup failed: {}", e),
            }))
        }
    }
}
```

### 3. **Frontend - Auto-trigger on Startup**

#### 3.1 Check Environment Variable
```typescript
// In App.tsx or main initialization
useEffect(() => {
    const checkStartupMode = async () => {
        const startupMode = window.electron.getEnv('GOOSE_STARTUP');
        
        if (startupMode === 'openrouter' && !hasProviderConfigured) {
            // Start OpenRouter setup automatically
            await startOpenRouterSetup();
        }
    };
    
    checkStartupMode();
}, []);
```

#### 3.2 Setup Flow Handler
```typescript
// ui/desktop/src/utils/openRouterSetup.ts
export async function startOpenRouterSetup() {
    try {
        // Show setup UI
        showSetupModal({
            title: 'Setting up OpenRouter',
            message: 'A browser window will open for authentication...',
            showProgress: true
        });
        
        // Call backend to start the flow
        const response = await fetch('/setup/openrouter/start', { 
            method: 'POST' 
        });
        
        const result = await response.json();
        
        if (result.success) {
            showSetupModal({
                title: 'Setup Complete!',
                message: 'OpenRouter has been configured successfully.',
                showProgress: false,
                autoClose: 3000
            });
            
            // Refresh the app to load the new configuration
            window.location.reload();
        } else {
            showSetupModal({
                title: 'Setup Failed',
                message: result.message,
                showProgress: false,
                showRetry: true,
                onRetry: startOpenRouterSetup
            });
        }
    } catch (error) {
        showSetupModal({
            title: 'Setup Error',
            message: 'Failed to complete OpenRouter setup',
            showProgress: false,
            showRetry: true,
            onRetry: startOpenRouterSetup
        });
    }
}
```

#### 3.3 Setup Modal Component
```typescript
// ui/desktop/src/components/SetupModal.tsx
interface SetupModalProps {
    title: string;
    message: string;
    showProgress?: boolean;
    showRetry?: boolean;
    onRetry?: () => void;
    autoClose?: number;
}

export function SetupModal({ 
    title, 
    message, 
    showProgress, 
    showRetry, 
    onRetry,
    autoClose 
}: SetupModalProps) {
    useEffect(() => {
        if (autoClose) {
            const timer = setTimeout(() => {
                closeModal();
            }, autoClose);
            return () => clearTimeout(timer);
        }
    }, [autoClose]);
    
    return (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg p-6 max-w-md w-full">
                <h2 className="text-xl font-bold mb-4">{title}</h2>
                <p className="mb-6">{message}</p>
                
                {showProgress && (
                    <div className="flex justify-center mb-4">
                        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
                    </div>
                )}
                
                {showRetry && (
                    <button 
                        onClick={onRetry}
                        className="w-full bg-blue-500 text-white rounded py-2 hover:bg-blue-600"
                    >
                        Retry Setup
                    </button>
                )}
            </div>
        </div>
    );
}
```

### 4. **Electron Main Process**
```javascript
// In main.js or preload.js
contextBridge.exposeInMainWorld('electron', {
    getEnv: (key) => process.env[key],
    // ... other methods
});
```

## User Experience Flow

1. **User launches app with `GOOSE_STARTUP=openrouter`**
2. **App detects no provider configured + env var set**
3. **Shows "Setting up OpenRouter" modal**
4. **Backend triggers existing PKCE flow**:
   - Opens browser to OpenRouter auth page
   - Runs local server on port 3000
   - Captures callback
   - Exchanges code for API key
   - Configures everything automatically
5. **Frontend shows success and reloads**
6. **User lands on configured app ready to use**

## Implementation Steps

1. **Add setup route to goose-server**
   - Simple wrapper around existing `OpenRouterAuth::complete_flow()`
   - No need to duplicate any auth logic

2. **Add startup check in frontend**
   - Check `GOOSE_STARTUP` env var on app initialization
   - Trigger setup if needed

3. **Create minimal setup UI**
   - Simple modal showing progress
   - Success/error states
   - Auto-reload on success

## Benefits

- **Reuses existing code**: No duplication of PKCE logic
- **Simple integration**: Just a thin wrapper in goose-server
- **Automated flow**: No buttons or manual triggers needed
- **Consistent behavior**: Same as CLI `goose auth openrouter`

## Notes

- The existing PKCE implementation in goose crate handles everything
- No need for popup windows or postMessage - the goose crate opens system browser
- No manual API key entry - this is purely for automated setup
- The modal is just for user feedback while the setup runs
