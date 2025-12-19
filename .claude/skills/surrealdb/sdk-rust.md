# SurrealDB Rust SDK

The official Rust SDK provides async-first, type-safe database access with support for WebSocket, HTTP, and embedded (in-memory/RocksDB) connections.

## Setup

```toml
[dependencies]
surrealdb = { version = "2", features = ["protocol-ws"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

## Connection Patterns

```rust
use surrealdb::{Surreal, engine::remote::ws::Ws};
use surrealdb::opt::auth::Root;

// WebSocket connection
let db = Surreal::new::<Ws>("localhost:8000").await?;

// In-memory (testing)
use surrealdb::engine::local::Mem;
let db = Surreal::new::<Mem>(()).await?;

// Persistent (RocksDB)
use surrealdb::engine::local::RocksDb;
let db = Surreal::new::<RocksDb>("path/to/database.db").await?;

// Authentication & namespace selection
db.signin(Root { username: "root", password: "secret" }).await?;
db.use_ns("namespace").use_db("database").await?;
```

## CRUD Operations

```rust
use serde::{Serialize, Deserialize};
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    id: RecordId,
    name: String,
    age: u8,
}

// Create with random ID
let person: Option<Person> = db.create("person").content(Person {
    id: RecordId::from(("person", "john")),
    name: "John".to_string(),
    age: 30,
}).await?;

// Create with specific ID
let person: Option<Person> = db.create(("person", "jane")).content(data).await?;

// Select all from table
let people: Vec<Person> = db.select("person").await?;

// Select specific record
let person: Option<Person> = db.select(("person", "john")).await?;

// Update (replace)
let updated: Option<Person> = db.update(("person", "john")).content(new_data).await?;

// Merge (partial update)
let updated: Option<Person> = db.update(("person", "john")).merge(partial_data).await?;

// Delete
let _: Option<Person> = db.delete(("person", "john")).await?;
```

## Query with Parameters

```rust
// Single parameter
let mut response = db
    .query("SELECT * FROM person WHERE age > $age")
    .bind(("age", 25))
    .await?;
let people: Vec<Person> = response.take(0)?;

// Multiple parameters (struct)
#[derive(Serialize)]
struct QueryParams<'a> {
    age: u8,
    name: &'a str,
}
let params = QueryParams { age: 25, name: "John" };
let mut response = db
    .query("SELECT * FROM person WHERE age > $age AND name = $name")
    .bind(params)
    .await?;

// Multi-statement queries
let mut response = db.query(r#"
    LET $adults = SELECT * FROM person WHERE age >= 18;
    LET $count = count($adults);
    RETURN { adults: $adults, count: $count };
"#).await?;
let result: Value = response.take(2)?;
```

## Error Handling

**Critical**: SDK returns `Ok(Response)` even when individual statements fail!

```rust
let mut response = db.query("SELECT * FROM person; INVALID;").await?;

// Check all errors at once
if let Err(errors) = response.check() {
    eprintln!("Query errors: {:?}", errors);
}

// Extract errors by statement index
let errors = response.take_errors();
for (idx, err) in errors {
    eprintln!("Statement {} failed: {}", idx, err);
}

// Safe extraction with error handling
let people: Result<Vec<Person>, _> = response.take(0);
match people {
    Ok(data) => println!("Success: {:?}", data),
    Err(e) => eprintln!("Deserialization error: {}", e),
}
```

## Connection Pooling

Use `mobc-surrealdb` for production connection pooling:

```rust
use mobc::Pool;
use mobc_surrealdb::{SurrealDBConnectionManager, ConnectionProtocol};

let manager = SurrealDBConnectionManager::new_with_protocol(
    ConnectionProtocol::Ws,
    "127.0.0.1:8000",
    "root",
    "root",
);

let pool = Pool::builder()
    .max_open(20)
    .max_idle(5)
    .max_lifetime(Some(Duration::from_secs(300)))
    .build(manager);

// Use pooled connection
let conn = pool.get().await?;
conn.use_ns("test").use_db("test").await?;
```

## Common Gotchas

### RecordId vs String

```rust
// WRONG - using string for ID field
struct Person { id: String, name: String }

// CORRECT - use RecordId
struct Person { id: RecordId, name: String }

// Creating RecordId
let id = RecordId::from(("person", "john"));
println!("Table: {}, Key: {}", id.tb(), id.id());
```

### Serde Flatten Issues

Avoid `#[serde(flatten)]` with `RecordId` - causes serialization errors:

```rust
// PROBLEMATIC
#[derive(Serialize, Deserialize)]
struct User {
    pub id: RecordId,
    #[serde(flatten)]
    inner: UserData,  // Causes "untagged enum" errors
}

// SOLUTION - flatten fields directly
#[derive(Serialize, Deserialize)]
struct User {
    pub id: RecordId,
    pub name: String,
    pub email: String,
}
```

### Async Runtime

SurrealDB requires Tokio:

```rust
#[tokio::main]  // Required
async fn main() -> surrealdb::Result<()> {
    // Don't mix with async-std
    // Don't call SurrealDB operations in spawn_blocking
}
```

## Transactions

```rust
let mut tx = db.transaction().await?;

match tx.query("CREATE person:john CONTENT $data").bind(("data", person)).await {
    Ok(_) => tx.commit().await?,
    Err(e) => {
        eprintln!("Transaction failed: {}", e);
        tx.cancel().await?;
    }
}
```

## Related

- [TypeScript SDK](./sdk-typescript.md)
- [Graph Database](./graph-database.md)
- [Vector Database](./vector-database.md)
