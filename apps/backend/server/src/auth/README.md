# Auth Module

Handles authentication and authorization for the backend.

## Files

| File            | Purpose                                               |
| --------------- | ----------------------------------------------------- |
| `mod.rs`        | Module exports                                        |
| `jwt.rs`        | JWT token creation, validation, and claims extraction |
| `extractors.rs` | Axum extractors for authenticated routes              |

## JWT Flow

```
1. User logs in → Server creates JWT with user claims
2. Client stores JWT (cookie or header)
3. Client sends JWT with requests
4. Server validates JWT → Extracts user info
```

## Extractors

### `AuthUser`

Extracts authenticated user from request. Returns 401 if not authenticated.

```rust
async fn protected_handler(
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    // user is guaranteed to be authenticated
}
```

### `MaybeAuthUser`

Optionally extracts user. Does not fail if not authenticated.

```rust
async fn public_handler(
    MaybeAuthUser(maybe_user): MaybeAuthUser,
) -> impl IntoResponse {
    if let Some(user) = maybe_user {
        // authenticated
    } else {
        // anonymous
    }
}
```

## Configuration

JWT settings are configured via environment variables:

- `JWT_SECRET` - Secret key for signing tokens
- Token expiration is hardcoded in `jwt.rs`
