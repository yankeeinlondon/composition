# SurrealDB Migrations

SurrealDB lacks official migration tooling, but several community solutions exist. Choose based on your deployment model.

## Tool Comparison

| Tool | Type | Key Feature | Best For |
|------|------|-------------|----------|
| **surrealdb-migrate** | CLI + Library | Rich config, verify/revert | Production apps |
| **surrealdb-migrations** | CLI + Library | Timestamp-based, schema/events | Rapid prototyping |
| **surrealdb-migrator** | Library only | Lightweight, code-defined | Embedded systems |
| **surrealdb_migration_engine** | Library only | Rust_embed, auto-detection | Single-binary deployments |

## surrealdb-migrate (Recommended)

### Setup

```bash
cargo install surrealdb-migrate-cli
cargo add surrealdb-migrate
```

### Configuration

```toml
# surrealdb-migrate.toml
[database]
address = "ws://localhost:8000"
namespace = "myapp"
database = "production"
auth_level = "Database"  # Root, Namespace, Database
username = "admin"
password = "s3cr3t"

[migrations]
folder = "./migrations"
ignore_order = false

[logging]
level = "info"
```

### Migration Structure

```
migrations/
├── 01-initial_schema/
│   └── up.surql
├── 02-add_user_roles/
│   ├── up.surql
│   └── down.surql
└── 03-add_indices/
    └── up.surql
```

### Migration File Example

```sql
-- migrations/02-add_user_roles/up.surql
DEFINE FIELD roles ON TABLE user TYPE array;
DEFINE FIELD roles.* ON TABLE user TYPE string
    ASSERT $value IN ['admin', 'user', 'guest'];

-- Backfill existing users
UPDATE user SET roles = ['user'] WHERE roles IS NONE;
```

```sql
-- migrations/02-add_user_roles/down.surql
REMOVE FIELD roles ON TABLE user;
```

### Rust Integration

```rust
use surrealdb_migrate::{
    config::{DbAuthLevel, DbClientConfig, RunnerConfig},
    runner::MigrationRunner,
    db_client::connect_to_database,
};

let db_config = DbClientConfig::default()
    .with_address("ws://localhost:8000")
    .with_namespace("myapp")
    .with_database("production")
    .with_auth_level(DbAuthLevel::Database)
    .with_username("admin")
    .with_password("secret");

let db = connect_to_database(&db_config).await?;

let runner_config = RunnerConfig::default()
    .with_migrations_folder(Path::new("./migrations"));

let runner = MigrationRunner::new(runner_config);

// Apply all pending migrations
runner.migrate(&db).await?;

// Revert last 2 migrations
runner.revert(&db, 2).await?;
```

## surrealdb-migrator (Code-Based)

For simple projects or embedded databases:

```rust
use surrealdb_migrator::{Migrations, M};

let migrations = Migrations::new(vec![
    M::up("DEFINE TABLE user; DEFINE FIELD name ON user TYPE string;")
        .down("REMOVE TABLE user;"),
    M::up("DEFINE TABLE post; DEFINE FIELD title ON post TYPE string;")
        .down("REMOVE TABLE post;"),
    M::up("CREATE user SET name = 'admin';")
        .down("DELETE user WHERE name = 'admin';"),
]);

// Migrate to latest
migrations.to_latest(&db).await?;

// Or to specific version
migrations.to_version(&db, 1).await?;
```

## Best Practices

### Separate Schema and Data

```sql
-- 01-schema.surql (DDL)
DEFINE TABLE user SCHEMAFUL;
DEFINE FIELD name ON user TYPE string;

-- 02-seed.surql (DML)
IF (SELECT count() FROM user WHERE id = user:admin) = 0 {
    CREATE user:admin SET name = 'Admin';
};
```

### Use Transactions

```sql
BEGIN TRANSACTION;

-- 1. Create new structure
DEFINE TABLE user_v2 SCHEMAFUL;
DEFINE FIELD name ON user_v2 TYPE string;
DEFINE FIELD settings ON user_v2 TYPE object;

-- 2. Migrate data
INSERT INTO user_v2 (id, name, settings)
SELECT id, name, {email: email} FROM user;

-- 3. Verify
IF (SELECT count() FROM user) != (SELECT count() FROM user_v2) {
    THROW "Migration count mismatch";
};

-- 4. Swap tables
REMOVE TABLE user;
-- Note: SurrealDB doesn't support RENAME, recreate with data

COMMIT TRANSACTION;
```

### Make Migrations Idempotent

```sql
-- Check before defining
IF (SELECT count() FROM information_schema.tables WHERE name = 'analytics') = 0 {
    DEFINE TABLE analytics SCHEMAFULL;
    DEFINE FIELD event_type ON analytics TYPE string;
};
```

### Gradual Schema Changes

```sql
-- Step 1: Add field without strict type
DEFINE FIELD email_verified ON user;

-- Step 2: Backfill (separate migration)
UPDATE user SET email_verified = false WHERE email_verified IS NONE;

-- Step 3: Add type constraint (after data is clean)
DEFINE FIELD email_verified ON user TYPE bool
    VALUE $value OR false
    ASSERT $value = true OR $value = false;
```

## Common Gotchas

### Connection Errors

```rust
// Always signin before migrating
db.signin(Root { username: "root", password: "root" }).await?;

// Verify connection
db.query("RETURN true").await?;
```

### Migration Order Conflicts

Use sequential numbering (not timestamps) for team environments:

```bash
mkdir -p migrations/04-add-index
# NOT: migrations/20240315120000-add-index
```

### Down Migration Dependencies

```sql
-- Check existence before removing
IF (SELECT count() FROM information_schema.tables WHERE name = 'temp_data') > 0 {
    REMOVE TABLE temp_data;
};
```

### Performance with Large Data

```sql
-- Batch processing in migrations
LET $batch_size = 1000;
LET $total = (SELECT count() FROM user WHERE settings IS NONE)[0].count;
LET $batches = math::ceil($total / $batch_size);

FOR $i IN 0..$batches {
    UPDATE user WHERE settings IS NONE
        LIMIT $batch_size
        SET settings = {theme: 'default'};
    SLEEP 100ms;
};
```

### Testing Migrations

```rust
#[tokio::test]
async fn test_migrations() {
    let db = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;

    let runner = MigrationRunner::new(RunnerConfig::default());

    // Test apply
    runner.migrate(&db).await?;

    // Verify schema
    let result: Vec<User> = db.select("user").await?;
    assert!(!result.is_empty());

    // Test rollback
    runner.revert(&db, 1).await?;
}
```

## Debugging

| Symptom | Check | Fix |
|---------|-------|-----|
| Connection fails | `surreal sql --conn ws://localhost:8000` | Verify DB running, credentials |
| Order error | `surmig list --verbose` | Reorder files, sequential numbers |
| Field assertion | `SELECT id FROM table WHERE field IS NOT type` | Clean data first |
| Timeout | Set `SURREALDB_TIMEOUT=300s` | Use batches |
| State corrupt | `SELECT * FROM migrations` | Reset table, re-run |

## Related

- [Rust SDK](./sdk-rust.md)
- [TypeScript SDK](./sdk-typescript.md)
