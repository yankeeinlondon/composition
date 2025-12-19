---
name: surrealdb
description: Expert knowledge for SurrealDB, a multi-model database combining document, graph, relational, and vector capabilities. Use when building applications with SurrealDB in Rust or TypeScript, working with graph traversals, vector similarity search, schema design, migrations, or troubleshooting connection/authentication issues.
last_updated: 2025-12-18T12:00:00Z
hash: 7aa2a51630913c96
---

# SurrealDB

SurrealDB is an ACID-compliant multi-model database that unifies document, graph, relational, and vector storage in a single system. Use SurrealQL (SQL-like with graph extensions) to query across all models.

## Core Principles

- **Multi-model by design** - Combine document flexibility, graph traversals, and vector search in one query
- **Edges are tables** - Graph relationships are real tables with properties (use `RELATE` to create)
- **Always use parameter binding** - Never interpolate user input into queries (SQL injection risk)
- **Check statement errors** - SDK returns `Ok(Response)` even when individual statements fail; use `.check()` or `.take_errors()`
- **RecordId is the ID type** - Use `RecordId` (Rust) or proper ID format (TypeScript), not strings
- **Connection reuse** - Use singleton/pooled connections; don't create per-request connections
- **Explicit protocols** - Specify `ws://`, `wss://`, `http://`, or embedded protocols explicitly
- **Index dimensions must match** - HNSW vector index `DIMENSION` must equal your embedding size
- **Authenticate before queries** - Always `signin()` before database operations
- **Use SCHEMAFULL for production** - Enforce types on critical tables after prototyping

## Quick Reference

### Connection Patterns

```rust
// Rust - WebSocket connection
let db = Surreal::new::<Ws>("localhost:8000").await?;
db.signin(Root { username: "root", password: "secret" }).await?;
db.use_ns("namespace").use_db("database").await?;
```

```typescript
// TypeScript - WebSocket connection
const db = new Surreal();
await db.connect("ws://localhost:8000/rpc");
await db.use({ namespace: "test", database: "test" });
await db.signin({ username: "root", password: "root" });
```

### Graph Relations

```sql
-- Create relationship with properties
RELATE person:alice->knows->person:bob SET since = time::now(), metThrough = 'work';

-- Traverse outgoing relations
SELECT * FROM person:alice->knows->person;

-- Traverse incoming relations
SELECT * FROM person<-knows<-person:alice;
```

### Vector Search

```sql
-- Define HNSW index (dimension must match embeddings)
DEFINE INDEX doc_embedding ON document FIELDS embedding HNSW DIMENSION 1536 DISTANCE COSINE;

-- Similarity search (k=5 nearest neighbors)
SELECT *, vector::distance::cosine(embedding, $query) AS distance
FROM document WHERE embedding <|5, COSINE|> $query ORDER BY distance;
```

## Topics

### Language SDKs

- [Rust SDK](./sdk-rust.md) - Connection, CRUD, queries, error handling, pooling
- [TypeScript SDK](./sdk-typescript.md) - Connection, type safety, live queries, React patterns

### Multi-Model Features

- [Graph Database](./graph-database.md) - RELATE, traversals, recursive patterns, fraud detection
- [Vector Database](./vector-database.md) - Embeddings, HNSW indexes, hybrid search, RAG applications

### Development Tools

- [Migrations](./migrations.md) - surrealdb-migrate, schema vs data migrations, rollbacks
- [ORM Options](./orm.md) - surreal_orm (Rust), @surrealorm/orm (TypeScript), when to use

## Common Patterns

### Error Handling (Rust)

```rust
let mut response = db.query("SELECT * FROM person; INVALID STATEMENT;").await?;
// Check all statement errors
if let Err(errors) = response.check() {
    eprintln!("Errors: {:?}", errors);
}
// Or extract typed results with error handling
let people: Result<Vec<Person>, _> = response.take(0);
```

### Singleton Connection (TypeScript)

```typescript
class DatabaseService {
  private static instance: Surreal;
  static async getConnection(): Promise<Surreal> {
    if (this.instance?.status === "connected") return this.instance;
    this.instance = new Surreal();
    await this.instance.connect("ws://localhost:8000/rpc");
    await this.instance.use({ namespace: "app", database: "prod" });
    return this.instance;
  }
}
```

### Batch Processing

```sql
-- Process large updates in batches
LET $batch_size = 1000;
FOR $i IN 0..$batches {
    UPDATE user WHERE settings IS NONE LIMIT $batch_size SET settings = {theme: 'default'};
    SLEEP 100ms;
};
```

## Resources

- [Official Docs](https://surrealdb.com/docs)
- [Rust SDK](https://surrealdb.com/docs/sdk/rust)
- [TypeScript SDK](https://surrealdb.com/docs/sdk/javascript)
- [Surrealist](https://surrealist.app) - Visual database management tool
- [GitHub](https://github.com/surrealdb/surrealdb)
