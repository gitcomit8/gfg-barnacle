use actix_web::{web, App, HttpResponse, HttpServer, Result};
use futures::stream::{Stream};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

// Global counter to track active SSE connections
// This is where the bug manifests - connections keep accumulating
static CONNECTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct AppState {
    // Track all active connections (but never clean them up properly)
    active_connections: Arc<Mutex<Vec<usize>>>,
}

// SSE notification stream
// THE BUG: This function creates a new connection but the old ones are never closed
async fn sse_notifications(
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    
    // Add to active connections but never remove
    if let Ok(mut connections) = data.active_connections.lock() {
        connections.push(connection_id);
        println!("🔌 New SSE connection opened: #{} (Total active: {})", 
                 connection_id, connections.len());
    }

    let stream = create_notification_stream();

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream))
}

fn create_notification_stream() -> Pin<Box<dyn Stream<Item = Result<web::Bytes, actix_web::Error>>>> {
    let interval_duration = Duration::from_secs(2);
    let ticker = interval(interval_duration);
    let stream = IntervalStream::new(ticker);

    Box::pin(stream.map(move |_| {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let notification = format!(
            "data: {{\"message\": \"Notification at {}\", \"timestamp\": {}}}\n\n",
            chrono::Local::now().format("%H:%M:%S"),
            timestamp
        );
        Ok(web::Bytes::from(notification))
    }))
}

// Endpoint to check active connections
async fn connection_status(data: web::Data<AppState>) -> Result<HttpResponse> {
    let connections = data.active_connections.lock()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
    
    let status = serde_json::json!({
        "active_connections": connections.len(),
        "connection_ids": *connections,
        "warning": if connections.len() >= 6 { 
            "⚠️ DANGER: Approaching browser connection limit! App will freeze soon!" 
        } else { 
            "OK" 
        }
    });
    
    Ok(HttpResponse::Ok().json(status))
}

// Serve the frontend HTML
async fn serve_index() -> Result<HttpResponse> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Buggy SSE Notification Dashboard</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 900px;
            margin: 50px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 15px;
            padding: 30px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }
        h1 {
            text-align: center;
            margin-bottom: 10px;
        }
        .warning {
            background: rgba(255, 152, 0, 0.3);
            border-left: 4px solid #ff9800;
            padding: 15px;
            margin: 20px 0;
            border-radius: 5px;
        }
        .notification-box {
            background: rgba(255, 255, 255, 0.2);
            border-radius: 10px;
            padding: 20px;
            margin: 20px 0;
            min-height: 150px;
            max-height: 300px;
            overflow-y: auto;
        }
        .notification {
            background: rgba(76, 175, 80, 0.3);
            border-left: 4px solid #4CAF50;
            padding: 10px;
            margin: 10px 0;
            border-radius: 5px;
            animation: slideIn 0.3s ease-out;
        }
        @keyframes slideIn {
            from {
                transform: translateX(-20px);
                opacity: 0;
            }
            to {
                transform: translateX(0);
                opacity: 1;
            }
        }
        .status-box {
            background: rgba(255, 255, 255, 0.2);
            border-radius: 10px;
            padding: 20px;
            margin: 20px 0;
        }
        .status-item {
            display: flex;
            justify-content: space-between;
            margin: 10px 0;
            font-size: 18px;
        }
        .status-value {
            font-weight: bold;
            color: #4CAF50;
        }
        .status-value.danger {
            color: #f44336;
            animation: pulse 1s infinite;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        button {
            background: rgba(255, 255, 255, 0.3);
            border: 2px solid white;
            color: white;
            padding: 15px 30px;
            font-size: 16px;
            border-radius: 8px;
            cursor: pointer;
            margin: 10px 5px;
            transition: all 0.3s;
        }
        button:hover {
            background: rgba(255, 255, 255, 0.5);
            transform: scale(1.05);
        }
        .button-container {
            text-align: center;
            margin: 20px 0;
        }
        .bug-info {
            background: rgba(244, 67, 54, 0.3);
            border: 2px solid #f44336;
            border-radius: 10px;
            padding: 20px;
            margin: 20px 0;
        }
        .bug-info h3 {
            margin-top: 0;
            color: #ffeb3b;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔔 Real-Time Notification Dashboard</h1>
        <p style="text-align: center; opacity: 0.8;">Navigate away and come back to trigger the bug!</p>
        
        <div class="bug-info">
            <h3>🐛 Known Bug</h3>
            <p><strong>Issue:</strong> Every time you navigate away and back, a new SSE connection is opened without closing the old one.</p>
            <p><strong>Impact:</strong> After ~5 minutes (6 navigations), you'll hit the browser's max connections per domain limit, causing all API calls to hang.</p>
            <p><strong>To Reproduce:</strong> Click "Navigate Away" button repeatedly and watch the connection count increase!</p>
        </div>

        <div class="status-box">
            <h3>📊 Connection Status</h3>
            <div class="status-item">
                <span>Active Connections:</span>
                <span class="status-value" id="connection-count">0</span>
            </div>
            <div class="status-item">
                <span>Status:</span>
                <span class="status-value" id="status-text">Initializing...</span>
            </div>
        </div>

        <div class="button-container">
            <button onclick="navigateAway()">🚀 Navigate Away (Simulated)</button>
            <button onclick="refreshPage()">🔄 Refresh Page (Real)</button>
            <button onclick="checkStatus()">📡 Check Connection Status</button>
        </div>

        <div class="notification-box" id="notifications">
            <p style="opacity: 0.6;">Waiting for notifications...</p>
        </div>
    </div>

    <script>
        let eventSource = null;
        let notificationCount = 0;

        // THE BUG: We never close the old EventSource before creating a new one
        function connectSSE() {
            // BUG: Should close existing connection first, but we don't!
            // if (eventSource) {
            //     eventSource.close();
            // }

            eventSource = new EventSource('/api/notifications');
            
            eventSource.onmessage = function(event) {
                notificationCount++;
                const data = JSON.parse(event.data);
                addNotification(data);
            };

            eventSource.onerror = function(error) {
                console.error('SSE Error:', error);
                addNotification({
                    message: '❌ Connection error or limit reached!',
                    timestamp: Math.floor(Date.now() / 1000)
                });
            };

            console.log('New SSE connection opened (old one NOT closed!)');
        }

        function addNotification(data) {
            const notificationsDiv = document.getElementById('notifications');
            if (notificationCount === 1) {
                notificationsDiv.innerHTML = '';
            }
            
            const notificationEl = document.createElement('div');
            notificationEl.className = 'notification';
            notificationEl.innerHTML = `<strong>${data.message}</strong><br><small>Time: ${new Date(data.timestamp * 1000).toLocaleTimeString()}</small>`;
            
            notificationsDiv.insertBefore(notificationEl, notificationsDiv.firstChild);
            
            // Keep only last 10 notifications
            while (notificationsDiv.children.length > 10) {
                notificationsDiv.removeChild(notificationsDiv.lastChild);
            }
        }

        async function checkStatus() {
            try {
                const response = await fetch('/api/status');
                const status = await response.json();
                
                document.getElementById('connection-count').textContent = status.active_connections;
                const statusText = document.getElementById('status-text');
                statusText.textContent = status.warning;
                
                if (status.active_connections >= 6) {
                    statusText.classList.add('danger');
                    alert('⚠️ WARNING: You have ' + status.active_connections + ' active connections!\nBrowser connection limit reached. The app will hang on next API call!');
                } else {
                    statusText.classList.remove('danger');
                }
            } catch (error) {
                console.error('Failed to fetch status:', error);
                document.getElementById('status-text').textContent = 'Error fetching status';
            }
        }

        // Simulate navigation by disconnecting and reconnecting
        function navigateAway() {
            // Simulate navigating to another page and back
            // In real app, this would be React Router navigation, etc.
            console.log('Simulating navigation away from dashboard...');
            
            // THE BUG: We just create a new connection without closing the old one!
            setTimeout(() => {
                console.log('Back to dashboard - creating NEW connection...');
                connectSSE();
                checkStatus();
                
                addNotification({
                    message: '⚠️ New connection opened! Old one still active (LEAKED)',
                    timestamp: Math.floor(Date.now() / 1000)
                });
            }, 500);
        }

        function refreshPage() {
            location.reload();
        }

        // Auto-check status every 3 seconds
        setInterval(checkStatus, 3000);

        // Initialize on page load
        connectSSE();
        checkStatus();
    </script>
</body>
</html>"#;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = AppState {
        active_connections: Arc::new(Mutex::new(Vec::new())),
    };

    println!("🚀 Starting Buggy SSE Notification Server on http://localhost:8080");
    println!("📝 Open http://localhost:8080 in your browser");
    println!("🐛 The bug: SSE connections are never closed when navigating away!");
    println!("⚠️  After 6 navigations, your browser will hit connection limit and hang.\n");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(serve_index))
            .route("/api/notifications", web::get().to(sse_notifications))
            .route("/api/status", web::get().to(connection_status))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
