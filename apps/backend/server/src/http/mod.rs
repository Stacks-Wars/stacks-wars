// HTTP layer: domain routers, handlers, and route composition.
pub mod admin;
pub mod bot;
pub mod chain;
pub mod error;
pub mod games;
pub mod leaderboards;
pub mod lobbies;
pub mod public;
pub mod ratings;
pub mod seasons;
pub mod session;
pub mod stats;
pub mod users;

use axum::Router;
use axum::middleware::from_fn_with_state;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{
    middleware::{ApiRateLimit, AuthRateLimit, StrictRateLimit, rate_limit_with_state},
    state::AppState,
};

/// Build the top-level router and nest read, auth, strict, and admin groups under `/api`.
///
/// Each product domain is nested under `/api/<domain>` (e.g. `/api/users`, `/api/stats`) so
/// paths line up with module names. `public` stays at the service root (`/`, `/health`).
pub fn create_http_routes(state: AppState) -> Router {
    let state_for_layer = state.clone();

    let read_api = Router::new()
        .nest("/users", users::routes::read_routes())
        .nest("/leaderboards", leaderboards::routes::read_routes())
        .nest("/ratings", ratings::routes::read_routes())
        .nest("/games", games::routes::read_routes())
        .nest("/lobbies", lobbies::routes::read_routes())
        .nest("/seasons", seasons::routes::read_routes())
        .nest("/chain", chain::routes::read_routes())
        .nest("/stats", stats::routes::read_routes())
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<ApiRateLimit>,
        ));

    let auth_api = Router::new()
        .nest("/session", session::routes::auth_routes())
        .nest("/users", users::routes::auth_routes())
        .nest("/ratings", ratings::routes::auth_routes())
        .nest("/games", games::routes::auth_routes())
        .nest("/lobbies", lobbies::routes::auth_routes())
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<AuthRateLimit>,
        ));

    let strict_api = Router::new()
        .nest("/users", users::routes::strict_routes())
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<StrictRateLimit>,
        ));

    let admin_api = Router::new()
        .nest("/admin", admin::routes::routes())
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<AuthRateLimit>,
        ));

    Router::new()
        .merge(public::routes::routes())
        .nest(
            "/api",
            Router::new()
                .merge(read_api)
                .merge(auth_api)
                .merge(strict_api)
                .merge(admin_api)
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static(
                        "no-cache, no-store, max-age=0, must-revalidate",
                    ),
                )),
        )
        .with_state(state)
}
