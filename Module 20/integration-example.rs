// Integration Example: Using the Buggy Session State Manager in a Web Application
// 
// This example demonstrates how to integrate the session manager with Actix-web
// WARNING: This code contains the same bugs as the module itself!

use actix_web::{
    web, App, HttpServer, HttpResponse, Responder,
    http::StatusCode,
    middleware::Logger,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Import the buggy session manager
// In a real project: use session_state_manager::SessionManager;
// For this example, we'll assume it's available
use session_state_manager::SessionManager;

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    success: bool,
    session_id: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct SessionInfo {
    username: String,
    login_time: String,
    access_count: u64,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct UpdatePreferenceRequest {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    total_sessions: u64,
    active_sessions_reported: u64,
    active_sessions_actual: usize,
    cache_hits: u64,
    cache_misses: u64,
    cleanup_queue_size: usize,
    inconsistency_detected: bool,
}

// Login endpoint - creates a new session
async fn login_handler(
    session_mgr: web::Data<Arc<SessionManager>>,
    credentials: web::Json<LoginRequest>,
) -> impl Responder {
    // Simulate authentication (in reality, check against database)
    if credentials.password.is_empty() {
        return HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            session_id: None,
            message: "Invalid credentials".to_string(),
        });
    }

    // Create a new session
    // BUG: If multiple requests for the same user arrive simultaneously,
    // multiple sessions might be created when only one should exist
    let user_id = format!("user_{}", credentials.username);
    
    match session_mgr.create_session(user_id, credentials.username.clone()) {
        Ok(session_id) => {
            HttpResponse::Ok().json(LoginResponse {
                success: true,
                session_id: Some(session_id),
                message: "Login successful".to_string(),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                session_id: None,
                message: e,
            })
        }
    }
}

// Get session info endpoint - retrieves session data
async fn get_session_handler(
    session_mgr: web::Data<Arc<SessionManager>>,
    session_id: web::Path<String>,
) -> impl Responder {
    match session_mgr.get_session(&session_id) {
        Ok(session) => {
            // BUG: This increment might be lost due to race condition in increment_access()
            let _ = session_mgr.increment_access(&session_id);
            
            HttpResponse::Ok().json(SessionInfo {
                username: session.username,
                login_time: session.login_time.to_string(),
                access_count: session.access_count,
                metadata: session.metadata,
            })
        }
        Err(e) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

// Update user preference endpoint
async fn update_preference_handler(
    session_mgr: web::Data<Arc<SessionManager>>,
    session_id: web::Path<String>,
    preference: web::Json<UpdatePreferenceRequest>,
) -> impl Responder {
    // BUG: If multiple preference updates arrive for the same session,
    // some updates might be lost due to the read-modify-write race in update_session()
    match session_mgr.update_session(
        &session_id,
        preference.key.clone(),
        preference.value.clone(),
    ) {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Preference updated"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

// Logout endpoint - deletes session
async fn logout_handler(
    session_mgr: web::Data<Arc<SessionManager>>,
    session_id: web::Path<String>,
) -> impl Responder {
    // BUG: During deletion, the session exists in an inconsistent state
    // Statistics might be wrong, cleanup queue might grow unboundedly
    match session_mgr.delete_session(&session_id) {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Logged out successfully"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

// Admin endpoint to view statistics
async fn stats_handler(
    session_mgr: web::Data<Arc<SessionManager>>,
) -> impl Responder {
    let stats = session_mgr.get_stats();
    let actual_count = session_mgr.get_active_count();
    let queue_size = session_mgr.get_cleanup_queue_size();
    
    // BUG: Due to race conditions, these numbers often don't match
    let inconsistency = stats.active_sessions != actual_count as u64;
    
    HttpResponse::Ok().json(StatsResponse {
        total_sessions: stats.total_sessions,
        active_sessions_reported: stats.active_sessions,
        active_sessions_actual: actual_count,
        cache_hits: stats.cache_hits,
        cache_misses: stats.cache_misses,
        cleanup_queue_size: queue_size,
        inconsistency_detected: inconsistency,
    })
}

// Health check endpoint
async fn health_handler(
    session_mgr: web::Data<Arc<SessionManager>>,
) -> impl Responder {
    let queue_size = session_mgr.get_cleanup_queue_size();
    let stats = session_mgr.get_stats();
    let actual = session_mgr.get_active_count();
    
    // Detect potential issues
    let memory_leak_warning = queue_size > 1000;
    let inconsistency_warning = (stats.active_sessions as i64 - actual as i64).abs() > 10;
    
    let status = if memory_leak_warning || inconsistency_warning {
        "degraded"
    } else {
        "healthy"
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "warnings": {
            "memory_leak": memory_leak_warning,
            "inconsistent_state": inconsistency_warning,
        },
        "metrics": {
            "cleanup_queue_size": queue_size,
            "active_sessions": actual,
            "reported_sessions": stats.active_sessions,
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Create the session manager
    let session_mgr = Arc::new(SessionManager::new());
    
    // Start cleanup task in background
    let cleanup_mgr = session_mgr.clone();
    tokio::spawn(async move {
        cleanup_mgr.run_cleanup().await;
    });
    
    println!("🚀 Starting server on http://127.0.0.1:8080");
    println!("⚠️  WARNING: This server uses the buggy session manager!");
    println!();
    println!("Available endpoints:");
    println!("  POST   /api/login              - Create session");
    println!("  GET    /api/session/:id        - Get session info");
    println!("  PUT    /api/session/:id/prefs  - Update preferences");
    println!("  DELETE /api/logout/:id         - Delete session");
    println!("  GET    /api/stats              - View statistics");
    println!("  GET    /health                 - Health check");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(session_mgr.clone()))
            .wrap(Logger::default())
            .route("/api/login", web::post().to(login_handler))
            .route("/api/session/{session_id}", web::get().to(get_session_handler))
            .route("/api/session/{session_id}/prefs", web::put().to(update_preference_handler))
            .route("/api/logout/{session_id}", web::delete().to(logout_handler))
            .route("/api/stats", web::get().to(stats_handler))
            .route("/health", web::get().to(health_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// Example client usage (for testing)
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn example_usage() {
        let mgr = Arc::new(SessionManager::new());

        // 1. User logs in
        let session_id = mgr
            .create_session("user_alice".to_string(), "alice".to_string())
            .unwrap();
        println!("Created session: {}", session_id);

        // 2. User makes some requests (increment access count)
        for i in 0..10 {
            mgr.increment_access(&session_id).ok();
            println!("Access #{}", i + 1);
        }

        // 3. User updates preferences
        mgr.update_session(&session_id, "theme".to_string(), "dark".to_string())
            .unwrap();
        mgr.update_session(&session_id, "language".to_string(), "en".to_string())
            .unwrap();

        // 4. Get session info
        let session = mgr.get_session(&session_id).unwrap();
        println!("Session info: {:?}", session);
        println!("Metadata: {:?}", session.metadata);

        // BUG: Access count might be less than 10 due to race conditions!
        println!("Access count: {} (expected: 10)", session.access_count);

        // 5. User logs out
        mgr.delete_session(&session_id).unwrap();

        // 6. Check statistics
        let stats = mgr.get_stats();
        println!("Stats: {:?}", stats);

        // BUG: Cleanup queue might have entries even though we deleted the session
        println!("Cleanup queue size: {}", mgr.get_cleanup_queue_size());
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let mgr = Arc::new(SessionManager::new());
        let session_id = mgr
            .create_session("user_bob".to_string(), "bob".to_string())
            .unwrap();

        // Simulate concurrent requests from multiple handlers
        let mut handles = vec![];
        for i in 0..20 {
            let mgr = mgr.clone();
            let sid = session_id.clone();
            handles.push(tokio::spawn(async move {
                // Each "request" updates a different preference
                mgr.update_session(
                    &sid,
                    format!("key_{}", i),
                    format!("value_{}", i),
                )
                .ok();
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        // BUG: Due to race conditions, not all 20 updates will be present!
        let session = mgr.get_session(&session_id).unwrap();
        println!(
            "Expected 20 keys, got: {}",
            session.metadata.len()
        );
    }
}

/*
EXAMPLE API USAGE:

# 1. Login
curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secret123"}'

Response:
{
  "success": true,
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Login successful"
}

# 2. Get session info
curl http://localhost:8080/api/session/550e8400-e29b-41d4-a716-446655440000

Response:
{
  "username": "alice",
  "login_time": "2024-02-17T10:30:00Z",
  "access_count": 1,
  "metadata": {}
}

# 3. Update preference
curl -X PUT http://localhost:8080/api/session/550e8400-e29b-41d4-a716-446655440000/prefs \
  -H "Content-Type: application/json" \
  -d '{"key": "theme", "value": "dark"}'

Response:
{
  "success": true,
  "message": "Preference updated"
}

# 4. Check statistics (admin)
curl http://localhost:8080/api/stats

Response:
{
  "total_sessions": 42,
  "active_sessions_reported": 15,
  "active_sessions_actual": 14,  # BUG: Doesn't match!
  "cache_hits": 1234,
  "cache_misses": 56,
  "cleanup_queue_size": 523,     # BUG: Growing unboundedly!
  "inconsistency_detected": true  # BUG: State is inconsistent!
}

# 5. Logout
curl -X DELETE http://localhost:8080/api/logout/550e8400-e29b-41d4-a716-446655440000

Response:
{
  "success": true,
  "message": "Logged out successfully"
}

# 6. Health check
curl http://localhost:8080/health

Response:
{
  "status": "degraded",
  "warnings": {
    "memory_leak": true,           # BUG: Cleanup queue leak!
    "inconsistent_state": true     # BUG: Phantom sessions!
  },
  "metrics": {
    "cleanup_queue_size": 1543,
    "active_sessions": 23,
    "reported_sessions": 25
  }
}
*/
