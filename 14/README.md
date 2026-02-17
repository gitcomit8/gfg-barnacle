# Module 14: Authentication Guard Challenge

## Description
This module implements a Higher-Order Component (HOC) / Middleware that checks if a user is authenticated before allowing access to protected routes.

## The Bug 🐛
The authentication guard has a critical flaw that causes infinite redirect loops:

1. **The guard redirects unauthenticated users to `/login`**
2. **BUT the `/login` route itself is also protected by the auth guard!**

This creates a "Too many redirects" error in the browser because:
- User tries to access `/dashboard` → not authenticated → redirect to `/login`
- User arrives at `/login` → guard checks auth → still not authenticated → redirect to `/login`
- Infinite loop! 🔄

## The Challenge
Participants must fix:
1. **Route whitelisting**: Certain routes (like `/login`, `/`, `/auth/*`) should NOT trigger the auth check and should be publicly accessible
2. **Middleware logic**: The authentication middleware needs to distinguish between public and protected routes

## Running the Module

```bash
cargo build
cargo run
```

The server will start on `http://localhost:8080`

### Available Routes:
- `GET /` - Home page
- `GET /login` - Login page (BUGGY - causes infinite redirects!)
- `GET /dashboard` - Protected dashboard
- `POST /auth/login` - Authenticate user
- `POST /auth/logout` - Logout user
- `GET /auth/status` - Check auth status

## Expected Behavior (After Fix)
- Unauthenticated users should be able to access `/login` and `/` without redirects
- Authenticated users should be able to access `/dashboard`
- The `/auth/*` endpoints should be publicly accessible (for login/logout operations)

## Hints
- Look at the `auth_middleware.rs` file
- Check which routes are being protected - currently ALL routes require authentication!
- The middleware needs to check the request path before enforcing authentication
- Think about how to whitelist public routes that should skip the auth check

## Difficulty Level
**Medium-Hard** ⭐⭐⭐⭐

The bug is subtle because:
- It's a logical error in route protection
- Requires understanding of middleware execution flow
- Real-world scenario that happens in production!
- The fix is simple once you understand the problem
