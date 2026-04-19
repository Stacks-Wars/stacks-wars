# HTTP module

Axum routers and request handlers. Layout is **domain-first**: each area under `http/` has `routes.rs` (path wiring) and `handlers.rs` (or split modules like `chain/`). Top-level composition lives in [`mod.rs`](mod.rs).

## Layout

```text
http/
├── mod.rs              # create_http_routes: merge groups, nest /api, with_state
├── error.rs            # ApiError (IntoResponse, From<AppError>)
├── public/             # /health, / (no /api prefix)
├── users/              # /api/users/…
├── session/            # /api/session/…
├── games/              # /api/games/…
├── lobbies/            # /api/lobbies/…
├── seasons/            # /api/seasons/…
├── ratings/            # /api/ratings/… (platform_rating)
├── leaderboards/       # /api/leaderboards/…
├── stats/              # /api/stats/…
├── chain/              # /api/chain/…
├── admin/              # /api/admin/…
└── bot/                # Telegram (unchanged)
```

## URL nesting

Every product domain is mounted under **`/api/<domain>`** so paths mirror folder names (easier to search the tree and the URL at once). `public` stays at the service root.

Examples:

| Domain        | Example paths |
|---------------|----------------|
| `users`       | `GET /api/users/user/{id}`, `POST /api/users/register`, `PATCH /api/users/user/profile` |
| `session`     | `GET /api/session/me`, `POST /api/session/logout` |
| `games`       | `GET /api/games`, `GET /api/games/{identifier}`, `POST /api/games` |
| `lobbies`     | `GET /api/lobbies`, `GET /api/lobbies/{id}`, `GET /api/lobbies/game/{id}/lobbies` |
| `seasons`     | `GET /api/seasons`, `GET /api/seasons/current` |
| `ratings`     | `GET /api/ratings/platform-rating`, `POST /api/ratings/platform-rating` |
| `leaderboards`| `GET /api/leaderboards`, `GET /api/leaderboards/{user_id}`, `GET /api/leaderboards/stats/games` |
| `stats`       | `GET /api/stats` |
| `chain`       | `GET /api/chain/contract`, `GET /api/chain/balance/{addr}` |
| `admin`       | `POST /api/admin/season`, `PATCH /api/admin/game/{id}/active` |

## How `/api` is composed

1. **Read API** — `nest("/users", …)`, `nest("/leaderboards", …)`, … then one **`ApiRateLimit`** layer.
2. **Auth writes** — `nest("/session", …)`, `nest("/users", auth)`, … then **`AuthRateLimit`**.
3. **Strict** — `nest("/users", strict)` (`POST …/register`) + **`StrictRateLimit`**.
4. **Admin** — `nest("/admin", …)` + **`AuthRateLimit`**.

Public routes (`/health`, `/`) have **no** `/api` prefix and no API rate limit. Cache-Control `no-cache` applies under `/api`.

## Adding a route

1. Pick the domain (or add `http/<domain>/` with `mod.rs`, `routes.rs`, `handlers.rs`).
2. Implement the handler in that domain’s `handlers.rs`.
3. Add the path in `routes.rs` **relative to `/api/<domain>`** (no leading `/api` in the string).
4. Wire `nest("/<domain>", …)` in the correct group in [`mod.rs`](mod.rs) if you added a new domain.
5. Run `cargo check -p stacks_wars_server` and update `apps/web` / tests if URLs change.

## Integration tests

HTTP endpoints are covered under `tests/integration.rs` → `tests/integration/http/` (one module per `src/http/` domain). Run:

`cargo test -p stacks_wars_server --test integration`

## Errors

- Repositories still use [`AppError`](../errors.rs).
- [`ApiError`](error.rs) is the HTTP JSON error type; **`get_user`** returns `Result<_, ApiError>` as a template.
