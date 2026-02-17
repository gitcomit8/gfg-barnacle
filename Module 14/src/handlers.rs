use actix_web::{web, HttpResponse, Responder, HttpRequest, cookie::Cookie};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AuthStatusResponse {
    authenticated: bool,
    message: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn home() -> impl Responder {
    HttpResponse::Ok().body(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Auth Guard Challenge - Home</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
                h1 { color: #333; }
                .warning { background: #fff3cd; border: 1px solid #ffc107; padding: 15px; margin: 20px 0; border-radius: 4px; }
                .link { display: block; margin: 10px 0; }
                a { color: #007bff; text-decoration: none; }
                a:hover { text-decoration: underline; }
            </style>
        </head>
        <body>
            <h1>🏠 Welcome to Auth Guard Challenge</h1>
            <p>This is the home page. It should be accessible without authentication.</p>
            
            <div class="warning">
                <strong>⚠️ Warning:</strong> This module contains a deliberate bug!<br>
                Clicking on the "Login" link will cause an infinite redirect loop.
            </div>
            
            <h2>Available Pages:</h2>
            <a class="link" href="/">🏠 Home (public)</a>
            <a class="link" href="/login">🔐 Login (BUGGY - will cause infinite redirects!)</a>
            <a class="link" href="/dashboard">📊 Dashboard (requires authentication)</a>
            
            <h2>The Bug:</h2>
            <p>The authentication middleware is applied to ALL routes, including the login page itself. 
            When you try to access /login without being authenticated, it redirects you to... /login! 
            This creates an infinite loop.</p>
            
            <h2>Your Challenge:</h2>
            <p>Fix the auth_middleware.rs file to:</p>
            <ol>
                <li>Whitelist public routes like /login, /, and /auth/* so they don't require authentication</li>
                <li>Only protect routes that actually need authentication (like /dashboard)</li>
                <li>Ensure the middleware checks the request path before enforcing authentication</li>
            </ol>
        </body>
        </html>
    "#)
}

pub async fn login_page() -> impl Responder {
    // If we reach here, it means the middleware allowed the request through
    // But with the bug, we'll never reach here because middleware redirects!
    HttpResponse::Ok().body(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Login Page</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 400px; margin: 50px auto; padding: 20px; }
                form { background: #f5f5f5; padding: 20px; border-radius: 8px; }
                input { width: 100%; padding: 10px; margin: 10px 0; box-sizing: border-box; }
                button { width: 100%; padding: 10px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; }
                button:hover { background: #0056b3; }
                .success { color: green; }
            </style>
        </head>
        <body>
            <h1>🔐 Login</h1>
            <p class="success">✅ You fixed the bug! This page is now accessible.</p>
            <form action="/auth/login" method="post">
                <input type="text" name="username" placeholder="Username" required>
                <input type="password" name="password" placeholder="Password" required>
                <button type="submit">Login</button>
            </form>
            <p><small>Hint: Use any username/password for demo purposes</small></p>
            <p><a href="/">← Back to home</a></p>
        </body>
        </html>
    "#)
}

pub async fn dashboard(req: HttpRequest) -> impl Responder {
    let username = req.cookie("auth_token")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    
    HttpResponse::Ok().body(format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Dashboard</title>
            <style>
                body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
                .user-info {{ background: #d4edda; padding: 15px; border-radius: 4px; margin: 20px 0; }}
            </style>
        </head>
        <body>
            <h1>📊 Dashboard</h1>
            <div class="user-info">
                <strong>Welcome, {}!</strong><br>
                You are authenticated and can view this protected page.
            </div>
            <p><a href="/">← Back to home</a></p>
            <form action="/auth/logout" method="post">
                <button type="submit">Logout</button>
            </form>
        </body>
        </html>
    "#, username))
}

pub async fn do_login(form: web::Form<LoginRequest>) -> impl Responder {
    // Simple demo: accept any credentials
    println!("Login attempt: username={}", form.username);
    
    let auth_token = format!("user_{}", form.username);
    
    let cookie = Cookie::build("auth_token", auth_token)
        .path("/")
        .finish();
    
    HttpResponse::Found()
        .cookie(cookie)
        .append_header(("Location", "/dashboard"))
        .finish()
}

pub async fn do_logout() -> impl Responder {
    let cookie = Cookie::build("auth_token", "")
        .path("/")
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();
    
    HttpResponse::Found()
        .cookie(cookie)
        .append_header(("Location", "/"))
        .finish()
}

pub async fn auth_status(req: HttpRequest) -> impl Responder {
    let authenticated = req.cookie("auth_token").is_some();
    
    let response = AuthStatusResponse {
        authenticated,
        message: if authenticated {
            "User is authenticated".to_string()
        } else {
            "User is not authenticated".to_string()
        },
    };
    
    HttpResponse::Ok().json(response)
}
