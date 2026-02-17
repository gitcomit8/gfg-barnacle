# Quick Start Guide - Session State Manager Module

## 🚀 5-Minute Integration

### Step 1: Add to Your Webapp

```rust
use session_state_manager::SessionManager;
use std::sync::Arc;

// Create a shared session manager
let session_mgr = Arc::new(SessionManager::new());
```

### Step 2: Create Sessions on Login

```rust
// In your login handler
async fn handle_login(username: String, password: String) -> Result<String, Error> {
    // ... authenticate user ...
    
    let user_id = "user_12345".to_string();
    let session_id = session_mgr.create_session(user_id, username)?;
    
    Ok(session_id)
}
```

### Step 3: Access Session Data

```rust
// In your protected route handler
async fn handle_protected_route(session_id: String) -> Result<Response, Error> {
    let session = session_mgr.get_session(&session_id)?;
    
    if !session.is_authenticated {
        return Err(Error::Unauthorized);
    }
    
    println!("User: {}, Accessed: {} times", 
             session.username, 
             session.access_count);
    
    Ok(Response::success())
}
```

### Step 4: Update Session State

```rust
// Store user preferences
session_mgr.update_session(
    &session_id, 
    "theme".to_string(), 
    "dark".to_string()
)?;

// Track activity
session_mgr.increment_access(&session_id)?;
```

### Step 5: Logout

```rust
// Delete session on logout
session_mgr.delete_session(&session_id)?;
```

## 🔧 Framework-Specific Integration

### Actix-Web

```rust
use actix_web::{web, App, HttpServer};
use session_state_manager::SessionManager;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let session_mgr = Arc::new(SessionManager::new());
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(session_mgr.clone()))
            .route("/login", web::post().to(login_handler))
            .route("/api/data", web::get().to(data_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn login_handler(
    mgr: web::Data<Arc<SessionManager>>,
    credentials: web::Json<LoginRequest>
) -> impl Responder {
    match mgr.create_session(credentials.user_id.clone(), credentials.username.clone()) {
        Ok(session_id) => HttpResponse::Ok().json(json!({ "session_id": session_id })),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn data_handler(
    mgr: web::Data<Arc<SessionManager>>,
    session_id: web::Header<String>
) -> impl Responder {
    match mgr.get_session(&session_id.0) {
        Ok(session) => {
            mgr.increment_access(&session_id.0).ok();
            HttpResponse::Ok().json(session)
        },
        Err(e) => HttpResponse::Unauthorized().body(e),
    }
}
```

### Axum

```rust
use axum::{
    Router,
    Extension,
    extract::{Json, TypedHeader},
    http::StatusCode,
    routing::{get, post},
};
use session_state_manager::SessionManager;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let session_mgr = Arc::new(SessionManager::new());
    
    let app = Router::new()
        .route("/login", post(login_handler))
        .route("/api/data", get(data_handler))
        .layer(Extension(session_mgr));
    
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn login_handler(
    Extension(mgr): Extension<Arc<SessionManager>>,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    mgr.create_session(credentials.user_id, credentials.username)
        .map(|session_id| Json(LoginResponse { session_id }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn data_handler(
    Extension(mgr): Extension<Arc<SessionManager>>,
    TypedHeader(session_id): TypedHeader<SessionIdHeader>,
) -> Result<Json<SessionData>, StatusCode> {
    mgr.get_session(&session_id.0)
        .map(Json)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}
```

### Rocket

```rust
use rocket::{State, serde::json::Json};
use session_state_manager::SessionManager;
use std::sync::Arc;

#[rocket::main]
async fn main() {
    let session_mgr = Arc::new(SessionManager::new());
    
    rocket::build()
        .manage(session_mgr)
        .mount("/", routes![login_handler, data_handler])
        .launch()
        .await
        .unwrap();
}

#[post("/login", data = "<credentials>")]
fn login_handler(
    mgr: &State<Arc<SessionManager>>,
    credentials: Json<LoginRequest>
) -> Result<Json<LoginResponse>, Status> {
    mgr.create_session(credentials.user_id.clone(), credentials.username.clone())
        .map(|session_id| Json(LoginResponse { session_id }))
        .map_err(|_| Status::InternalServerError)
}

#[get("/api/data")]
fn data_handler(
    mgr: &State<Arc<SessionManager>>,
    session_id: SessionIdGuard
) -> Result<Json<SessionData>, Status> {
    mgr.get_session(&session_id.0)
        .map(Json)
        .map_err(|_| Status::Unauthorized)
}
```

## 🐛 Observing the Bugs

### Trigger Lost Updates

```rust
use std::thread;
use std::sync::Arc;

let mgr = Arc::new(SessionManager::new());
let session_id = mgr.create_session("user1".into(), "alice".into()).unwrap();

// Spawn multiple threads updating the same session
let mut handles = vec![];
for i in 0..10 {
    let mgr = mgr.clone();
    let sid = session_id.clone();
    handles.push(thread::spawn(move || {
        for j in 0..100 {
            mgr.update_session(&sid, format!("key_{}", i), format!("value_{}", j)).ok();
        }
    }));
}

for h in handles {
    h.join().unwrap();
}

// BUG: Some updates will be lost!
let session = mgr.get_session(&session_id).unwrap();
println!("Expected 10 keys, got: {}", session.metadata.len());
```

### Trigger Lost Increments

```rust
// Multiple threads incrementing access count
let mgr = Arc::new(SessionManager::new());
let session_id = mgr.create_session("user2".into(), "bob".into()).unwrap();

let mut handles = vec![];
for _ in 0..20 {
    let mgr = mgr.clone();
    let sid = session_id.clone();
    handles.push(thread::spawn(move || {
        for _ in 0..50 {
            mgr.increment_access(&sid).ok();
        }
    }));
}

for h in handles {
    h.join().unwrap();
}

// BUG: Count will be < 1000 due to lost updates
let session = mgr.get_session(&session_id).unwrap();
println!("Expected 1000, got: {}", session.access_count);
```

### Trigger Memory Leak

```rust
let mgr = SessionManager::new();

println!("Initial queue size: {}", mgr.get_cleanup_queue_size());

// Create and delete 1000 sessions
for i in 0..1000 {
    let id = mgr.create_session(
        format!("user_{}", i),
        format!("name_{}", i)
    ).unwrap();
    
    mgr.delete_session(&id).unwrap();
}

// BUG: Queue keeps growing!
println!("Queue size after 1000 deletes: {}", mgr.get_cleanup_queue_size());
// Should be 0 or small, but will be ~500 (half fail the arbitrary condition)
```

## 📊 Monitoring

```rust
// Get session statistics
let stats = session_mgr.get_stats();
println!("Total sessions: {}", stats.total_sessions);
println!("Active sessions: {}", stats.active_sessions);
println!("Cache hits: {}", stats.cache_hits);
println!("Cache misses: {}", stats.cache_misses);
println!("Failed cleanups: {}", stats.failed_cleanups);

// Get actual count (may not match stats due to bug #4!)
let actual_count = session_mgr.get_active_count();
println!("Actual sessions in store: {}", actual_count);
```

## ⚠️ Warning Signs in Production

If you (mistakenly) use this in production, watch for:

1. **Inconsistent counters**: Access counts don't add up
2. **Lost user data**: Preferences/settings disappear
3. **Growing memory**: Check `get_cleanup_queue_size()` over time
4. **Metric discrepancies**: `get_stats()` vs `get_active_count()` don't match
5. **Random logouts**: Sessions appear deleted when they shouldn't be

## 🏥 Health Check Endpoint

```rust
async fn health_check(
    Extension(mgr): Extension<Arc<SessionManager>>,
) -> Json<HealthStatus> {
    let stats = mgr.get_stats();
    let actual = mgr.get_active_count();
    let queue_size = mgr.get_cleanup_queue_size();
    
    Json(HealthStatus {
        active_sessions_reported: stats.active_sessions,
        active_sessions_actual: actual as u64,
        discrepancy: (stats.active_sessions as i64 - actual as i64).abs(),
        cleanup_queue_size: queue_size,
        memory_leak_detected: queue_size > 1000,
    })
}
```

## 🚨 Critical Notes

1. **This is intentionally buggy** - don't use in production!
2. **Tests may pass** - race conditions are non-deterministic
3. **Load testing required** - bugs only appear under concurrent access
4. **Monitor memory** - cleanup queue leak will grow over time
5. **Check metrics** - inconsistent counts indicate bugs

## 📖 Next Steps

- Read `README.md` for detailed bug descriptions
- Read `TECHNICAL.md` for in-depth analysis
- Try fixing the bugs as a learning exercise
- Compare with correct implementations

## 🔗 Resources

- Rust Concurrency: https://doc.rust-lang.org/book/ch16-00-concurrency.html
- RwLock pitfalls: https://docs.rs/parking_lot/
- Race condition patterns: https://en.wikipedia.org/wiki/Race_condition
