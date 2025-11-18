use std::{net::SocketAddr, time::Duration};

use tokio::sync::oneshot;

use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use redis::AsyncCommands;
use sqlx::PgPool;
use sqlx::Row;
use std::error::Error;
use uuid::Uuid;

/// Test application harness that keeps container handles alive while tests run.
pub struct TestApp {
    pub base_url: String,
    pub pg_pool: PgPool,
    pub state: stacks_wars_be::state::AppState,
    // hold on to the containers so they live as long as TestApp (boxed as Any)
    _pg_container: Box<dyn std::any::Any + Send + Sync>,
    _redis_container: Box<dyn std::any::Any + Send + Sync>,
    jwt_secret: String,
    shutdown: Option<oneshot::Sender<()>>,
}

impl TestApp {
    /// Gracefully stop the spawned server and drop containers
    pub async fn stop(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }

        // Wait a short moment to allow graceful shutdown
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    /// Flush Redis DB used by the test instance.
    pub async fn reset_redis(&self) -> Result<(), Box<dyn Error>> {
        // Use the AppState's redis pool to FLUSHDB - pool ensures connection options are correct
        let mut conn = self.state.redis.get().await?;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut *conn).await?;
        Ok(())
    }

    /// Reset the Postgres schema by dropping and recreating `public`, then re-run migrations.
    #[allow(dead_code)]
    pub async fn reset_db(&self) -> Result<(), Box<dyn Error>> {
        // Drop and recreate public schema to ensure clean state
        sqlx::query("DROP SCHEMA public CASCADE")
            .execute(&self.pg_pool)
            .await?;
        sqlx::query("CREATE SCHEMA public")
            .execute(&self.pg_pool)
            .await?;

        // Re-run migrations
        sqlx::migrate!("./migrations").run(&self.pg_pool).await?;
        Ok(())
    }

    /// Generate a JWT for tests for the given user id. Uses JWT_SECRET env var.
    pub fn generate_jwt_for_user(&self, user_id: Uuid) -> Result<String, Box<dyn Error>> {
        let secret = &self.jwt_secret;
        let now = Utc::now();
        let expiry_days: i64 = std::env::var("TOKEN_EXPIRY_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(7);

        let claims = stacks_wars_be::auth::jwt::Claims {
            sub: user_id.to_string(),
            wallet: "test_wallet".to_string(),
            iat: now.timestamp(),
            exp: (now + ChronoDuration::days(expiry_days)).timestamp(),
            jti: Some(Uuid::new_v4().to_string()),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?;
        Ok(token)
    }
}

/// Spawn the app with Postgres+Redis test containers, run migrations, and
/// start the axum server on an ephemeral port.
pub async fn spawn_app_with_containers() -> TestApp {
    // Run Postgres and Redis containers using the community async modules
    let pg_container = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres container");
    let redis_container = Redis::default()
        .start()
        .await
        .expect("failed to start redis container");

    // Build connection strings for containers (async port lookup)
    let pg_port = pg_container.get_host_port_ipv4(5432).await.unwrap();
    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();

    let database_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        pg_port
    );
    let redis_url = format!("redis://127.0.0.1:{}/", redis_port);

    // Build Redis pool and Bot directly (avoid modifying process env in tests)
    use bb8::Pool as Bb8Pool;
    use bb8_redis::RedisConnectionManager;
    use teloxide::Bot;

    // Initialize tracing for test runs so we see server-side errors in test output
    let _ = tracing_subscriber::fmt::try_init();

    let manager = RedisConnectionManager::new(redis_url.clone()).expect("invalid redis url");
    let redis_pool: Bb8Pool<RedisConnectionManager> = Bb8Pool::builder()
        .max_size(20)
        .build(manager)
        .await
        .expect("failed to build redis pool");

    // Use a test JWT secret stored in the TestApp (avoid mutating process env)
    let jwt_secret = "stacks_wars_deep_and_hidden_secret".to_string();

    // Wait for Postgres to accept connections
    let mut retries = 0;
    let pg_pool: PgPool;
    loop {
        match PgPool::connect(&database_url).await {
            Ok(pool) => {
                pg_pool = pool;
                break;
            }
            Err(_) if retries < 30 => {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Err(e) => panic!("Could not connect to Postgres in test container: {}", e),
        }
    }

    tracing::info!("Running migrations against {}", database_url);
    // Apply migrations using sqlx::migrate! macro which looks in ./migrations
    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to run database migrations for test database");

    // Debug: list tables after migrations to ensure migrations applied
    match sqlx::query("SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname = 'public'")
        .fetch_all(&pg_pool)
        .await
    {
        Ok(rows) => {
            let names: Vec<String> = rows
                .into_iter()
                .filter_map(|r| r.try_get::<String, _>("tablename").ok())
                .collect();
            tracing::info!("Tables after migrations: {:?}", names);
        }
        Err(e) => tracing::warn!("Could not list tables after migrations: {}", e),
    }

    // Build AppState manually using the pools we created
    let bot = Bot::new("test-bot-token");
    let state = stacks_wars_be::state::AppState {
        connections: Default::default(),
        chat_connections: Default::default(),
        redis: redis_pool,
        postgres: pg_pool.clone(),
        bot,
    };

    // One-time Redis health check: log but don't fail setup on error.
    match state.redis.get().await {
        Ok(mut conn) => {
            let res: redis::RedisResult<i64> = conn.incr("__test_health__", 1).await;
            match res {
                Ok(v) => tracing::info!("redis health check incr -> {}", v),
                Err(e) => tracing::warn!("redis health check incr error: {}", e),
            }
            // attempt to cleanup the key we touched
            let _del_res: redis::RedisResult<u64> = conn.del("__test_health__").await;
        }
        Err(e) => tracing::warn!("redis health check: could not get conn: {}", e),
    }

    // Build router (same as runtime): use http::create_http_routes and ws::create_ws_routes
    // Build top-level router and attach shared AppState at the top level so
    // middleware on nested routers (rate-limiter) can read State<AppState>
    let app = stacks_wars_be::http::create_http_routes(state.clone())
        .merge(stacks_wars_be::ws::create_ws_routes(state.clone()))
        .layer(stacks_wars_be::cors_layer())
        .fallback(|| async { "404 Not Found" });

    // Bind to ephemeral port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://127.0.0.1:{}", addr.port());

    // Start server with graceful shutdown
    let (tx, rx) = oneshot::channel::<()>();

    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = rx.await;
    });

    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!("test server error: {}", e);
        }
    });

    TestApp {
        base_url,
        pg_pool,
        state,
        _pg_container: Box::new(pg_container),
        _redis_container: Box::new(redis_container),
        jwt_secret,
        shutdown: Some(tx),
    }
}
