# CLI Binaries

Standalone CLI tools for database and cache management.

## Files

| File               | Purpose                                   |
| ------------------ | ----------------------------------------- |
| `hydrate.rs`       | Populate Redis cache from PostgreSQL data |
| `migrate_redis.rs` | Run Redis data migrations                 |

## hydrate

Hydrates Redis with data from PostgreSQL. Run this after:

- Server restart (Redis was cleared)
- Database restore
- Initial deployment

```bash
cargo run --bin hydrate
```

**What it hydrates:**

- Lobby states
- Player states
- Active game sessions

## migrate_redis

Runs Redis data migrations for schema changes.

```bash
cargo run --bin migrate_redis
```

**When to use:**

- Changing Redis key structure
- Adding new fields to cached data
- Migrating data format

## Environment Variables

Both binaries require:

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
