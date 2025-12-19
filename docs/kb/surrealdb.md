---
name: surrealdb
description: Comprehensive guide to SurrealDB - a multi-model database combining document, graph, relational, and vector capabilities with Rust and TypeScript SDKs
created: 2025-01-15
last_updated: 2025-12-18T00:00:00Z
hash: b07b83acb5c68738
tags:
  - database
  - surrealdb
  - multi-model
  - graph-database
  - vector-database
  - rust
  - typescript
---

# SurrealDB: Comprehensive Developer Guide

SurrealDB is a multi-model database that combines document, relational, graph, and vector capabilities within a single ACID-compliant system. Unlike traditional databases requiring separate storage engines for different data models, SurrealDB provides a unified query language (SurrealQL) that seamlessly handles all paradigms. This guide covers SDK usage in Rust and TypeScript, advanced features like graph traversals and vector search, migration strategies, and common pitfalls with solutions.

## Table of Contents

- [Foundation](#foundation)
  - [Core Architecture](#core-architecture)
  - [Multi-Model Capabilities](#multi-model-capabilities)
  - [SurrealQL Overview](#surrealql-overview)
- [SDK Integration](#sdk-integration)
  - [Rust SDK](#rust-sdk)
  - [TypeScript SDK](#typescript-sdk)
  - [Connection Management](#connection-management)
  - [Authentication Patterns](#authentication-patterns)
- [Data Modeling](#data-modeling)
  - [Record Links vs Graph Relations](#record-links-vs-graph-relations)
  - [Schema Design](#schema-design)
  - [Type-Safe Operations](#type-safe-operations)
- [Graph Database Features](#graph-database-features)
  - [The RELATE Statement](#the-relate-statement)
  - [Graph Traversals](#graph-traversals)
  - [Recursive Queries](#recursive-queries)
- [Vector Database Features](#vector-database-features)
  - [Vector Storage and Indexing](#vector-storage-and-indexing)
  - [Similarity Search](#similarity-search)
  - [Hybrid Search](#hybrid-search)
- [ORM and Abstractions](#orm-and-abstractions)
  - [Rust ORM Options](#rust-orm-options)
  - [TypeScript ORM Options](#typescript-orm-options)
  - [When to Use ORM vs Raw Queries](#when-to-use-orm-vs-raw-queries)
- [Migration Tools](#migration-tools)
  - [Tool Comparison](#tool-comparison)
  - [surrealdb-migrate](#surrealdb-migrate)
  - [Migration Best Practices](#migration-best-practices)
- [Tooling](#tooling)
  - [Surrealist GUI](#surrealist-gui)
- [Common Gotchas and Solutions](#common-gotchas-and-solutions)
  - [SDK-Specific Issues](#sdk-specific-issues)
  - [Query and Performance Issues](#query-and-performance-issues)
  - [Authentication Issues](#authentication-issues)
- [Production Patterns](#production-patterns)
  - [Error Handling](#error-handling)
  - [Connection Pooling](#connection-pooling)
  - [Performance Optimization](#performance-optimization)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

---

## Foundation

### Core Architecture

SurrealDB is built on three fundamental layers:

| Layer | Purpose | Key Components |
|-------|---------|----------------|
| **Engine Layer** | Transport protocols | HTTP, WebSocket, embedded WASM |
| **Storage Layer** | Data persistence | Memory, RocksDB, SurrealKV |
| **Query Layer** | Data operations | SurrealQL parser and executor |

Connection methods vary by deployment:

| Connection Type | Protocol | Use Case |
|-----------------|----------|----------|
| WebSockets | `ws://` / `wss://` | Real-time, live queries, persistent connections |
| HTTP | `http://` / `https://` | Stateless, serverless environments |
| Embedded (Memory) | `mem://` | Testing, prototyping |
| Embedded (File) | `file://` / `surrealkv://` | Single-binary deployments, edge computing |

### Multi-Model Capabilities

SurrealDB unifies multiple data paradigms:

- **Document Store**: Flexible JSON-like records with optional schema enforcement
- **Relational**: Table structures, JOINs, foreign key concepts via record links
- **Graph Database**: First-class edges with properties, bidirectional traversals
- **Vector Database**: Native embeddings, HNSW indexing, similarity search
- **Time Series**: Temporal queries with versioned storage (SurrealKV)

### SurrealQL Overview

SurrealQL extends SQL with graph and document operations:

```sql
-- Create a record
CREATE person:john SET name = 'John Doe', age = 30;

-- Graph relation with properties
RELATE person:john->bought->product:laptop
    SET quantity = 1, purchased_at = time::now();

-- Graph traversal
SELECT * FROM person:john->bought->product;

-- Vector similarity search
SELECT * FROM document
    WHERE embedding <|5, COSINE|> $query_embedding;
```

---

## SDK Integration

### Rust SDK

**Dependencies (Cargo.toml)**:

```toml
[dependencies]
surrealdb = { version = "2", features = ["protocol-ws"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

**Basic Connection Pattern**:

```rust
use surrealdb::{Surreal, engine::remote::ws::Ws};
use surrealdb::opt::auth::Root;

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    // Connect to the database
    let db = Surreal::new::<Ws>("localhost:8000").await?;

    // Authenticate
    db.signin(Root {
        username: "root",
        password: "secret",
    }).await?;

    // Select namespace and database
    db.use_ns("namespace").use_db("database").await?;

    Ok(())
}
```

**CRUD Operations**:

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
let person: Option<Person> = db
    .create("person")
    .content(Person {
        id: RecordId::from(("person", "john")),
        name: "John Doe".to_string(),
        age: 30,
    })
    .await?;

// Create with specific ID
let person: Option<Person> = db
    .create(("person", "jane"))
    .content(person_data)
    .await?;

// Query with parameters
let mut response = db
    .query("SELECT * FROM person WHERE age > $age")
    .bind(("age", 25))
    .await?;

let people: Vec<Person> = response.take(0)?;
```

### TypeScript SDK

**Installation**:

```bash
npm install surrealdb
# For embedded Node.js engine:
npm install @surrealdb/node
```

**Basic Connection**:

```typescript
import Surreal from 'surrealdb';

const db = new Surreal();

async function connect() {
    await db.connect('http://127.0.0.1:8000/rpc');
    await db.use({ namespace: 'test', database: 'test' });
    await db.signin({
        username: 'root',
        password: 'root'
    });
}
```

**Type-Safe Operations**:

```typescript
interface User {
    id?: string;
    username: string;
    email: string;
}

// Create
const user = await db.create<User>('User', {
    username: 'johndoe',
    email: 'john@example.com'
});

// Query with parameters (prevents SQL injection)
const [users] = await db.query<User[]>(
    'SELECT * FROM User WHERE email CONTAINS $domain',
    { domain: '@example.com' }
);
```

**Embedded Node.js Engine** (2024+):

```typescript
import { Surreal } from 'surrealdb';
import { surrealdbNodeEngines } from '@surrealdb/node';

const db = new Surreal({
    engines: surrealdbNodeEngines()
});

// In-memory for testing
await db.connect('mem://');

// Persistent file storage
await db.connect('surrealkv://./data.db');
```

### Connection Management

**Singleton Pattern (TypeScript)**:

```typescript
class DatabaseService {
    private static instance: Surreal;
    private static connecting: Promise<Surreal> | null = null;

    static async getConnection(): Promise<Surreal> {
        if (this.instance?.status === 'connected') {
            return this.instance;
        }

        if (this.connecting) {
            return this.connecting;
        }

        this.connecting = (async () => {
            this.instance = new Surreal();
            await this.instance.connect(process.env.SURREAL_URL);
            await this.instance.use({
                namespace: process.env.SURREAL_NS,
                database: process.env.SURREAL_DB
            });
            return this.instance;
        })();

        return this.connecting;
    }
}
```

**Connection Pooling (Rust)** using `mobc-surrealdb`:

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

### Authentication Patterns

SurrealDB supports multiple authentication levels:

| Level | Use Case | Rust Example |
|-------|----------|--------------|
| Root | Full system access | `Root { username, password }` |
| Namespace | Multi-tenant admin | `Namespace { namespace, username, password }` |
| Database | Application user | `Database { namespace, database, username, password }` |
| Record | Row-level security | `Record { access, namespace, database, params }` |

**Token-Based Auth (TypeScript)**:

```typescript
// Sign up and get token
const token = await db.signup(Record({
    access: 'account',
    namespace: 'ns',
    database: 'db',
    params: { name: 'user', pass: 'pass' }
}));

// Authenticate with token
await db.authenticate(token);
```

---

## Data Modeling

### Record Links vs Graph Relations

SurrealDB offers two approaches for relationships:

**Record Links** (simple references):

```sql
-- Store reference as field
CREATE post SET title = 'Hello', author = user:john;

-- Query via reference
SELECT * FROM post WHERE author = user:john;
SELECT author.name FROM post;  -- Fetch linked data
```

**Graph Relations** (edges with properties):

```sql
-- Create relationship with properties
RELATE user:john->wrote->post:hello
    SET created_at = time::now(), role = 'author';

-- Query relationship and properties
SELECT * FROM user:john->wrote->post;
SELECT *, ->wrote.created_at FROM user:john->wrote->post;
```

**Decision Guide**:

| Use Record Links When | Use Graph Relations When |
|----------------------|-------------------------|
| Simple one-to-many relationships | Relationships have their own properties |
| No metadata needed on the link | Bidirectional traversal required |
| Performance is critical | Querying the relationship itself |

### Schema Design

**Schemaless** (flexible):

```sql
DEFINE TABLE user SCHEMALESS;
```

**Schemafull** (enforced types):

```sql
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD age ON user TYPE number ASSERT $value >= 0 AND $value <= 150;
DEFINE FIELD roles ON user TYPE array;
DEFINE FIELD roles.* ON user TYPE string ASSERT $value IN ['admin', 'user', 'guest'];
```

### Type-Safe Operations

**Rust with Serde**:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<RecordId>,
    email: String,
    #[serde(flatten)]
    profile: UserProfile,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    name: String,
    bio: Option<String>,
}
```

**TypeScript with Zod** (runtime validation):

```typescript
import { z } from 'zod';

const UserSchema = z.object({
    id: z.string().optional(),
    email: z.string().email(),
    name: z.string(),
    age: z.number().int().positive()
});

type User = z.infer<typeof UserSchema>;

async function safeSelect(id: string): Promise<User | null> {
    const result = await db.select<User>(`User:${id}`);
    const validated = UserSchema.safeParse(result[0]);
    return validated.success ? validated.data : null;
}
```

---

## Graph Database Features

### The RELATE Statement

The `RELATE` statement creates graph edges using semantic triples:

```
subject -> predicate -> object
```

When executed, SurrealDB automatically creates `in` and `out` fields on the edge table:

```sql
RELATE person:billy->wishlist->product:01HGAR7A0R9BETTCMATM6SSXPT
    SET time.created_at = time::now();

-- The wishlist edge now has:
-- in: person:billy
-- out: product:01HGAR7A0R9BETTCMATM6SSXPT
```

### Graph Traversals

**Basic Traversal Syntax**:

```sql
-- Outgoing edges (person bought what products?)
SELECT * FROM person:john->bought->product;

-- Incoming edges (who bought this product?)
SELECT * FROM product:laptop<-bought<-person;

-- Multi-hop traversal
SELECT * FROM person:alice->knows->person->knows->person;
```

**Rust Example**:

```rust
impl MyDatabase {
    pub async fn get_products_bought_by_person(
        &self,
        person_id: RecordId,
    ) -> Vec<Product> {
        let result: Vec<Product> = self.db
            .query("SELECT * FROM $person_id->bought->product")
            .bind(("person_id", person_id))
            .await?
            .take(0)?;
        result
    }

    pub async fn find_similar_buyers(
        &self,
        person_id: RecordId,
    ) -> Vec<Person> {
        let query = r#"
            LET $bought_products = (SELECT VALUE out.id FROM $person_id->bought->product);
            SELECT DISTINCT in.* FROM bought
                WHERE out.id IN $bought_products
                AND in.id != $person_id
        "#;

        self.db.query(query)
            .bind(("person_id", person_id))
            .await?
            .take(0)?
    }
}
```

**TypeScript Example**:

```typescript
// Find friends of friends
const findFriendsOfFriends = async () => {
    const result = await db.query(`
        SELECT DISTINCT VALUE out.out
        FROM person:alice->friend_with->person->friend_with
        WHERE out.out != person:alice
            AND out.out NOT INSIDE (SELECT VALUE out FROM person:alice->friend_with)
    `);
    return result;
};
```

### Recursive Queries

> **Note:** SurrealDB does not currently support true recursive queries (like Cypher's variable-length paths). Use application-level recursion or fixed-depth queries.

**Fixed-Depth Traversal**:

```sql
-- Find people within 3 degrees of separation
SELECT * FROM person:tobie->{1..3}(->knows->person);
```

**Application-Level Recursion (TypeScript)**:

```typescript
async function findAllConnections(
    personId: string,
    maxDepth: number = 3
): Promise<Connection[]> {
    const visited = new Set<string>();
    const connections: Connection[] = [];

    async function traverse(currentId: string, depth: number, path: string[]) {
        if (depth > maxDepth || visited.has(currentId)) return;
        visited.add(currentId);

        const directConnections = await db.query(`
            SELECT out.id AS id, out.name AS name
            FROM $currentId->friend_with->person
            WHERE out.id NOT INSIDE $visited
        `, { currentId, visited: Array.from(visited) });

        for (const conn of directConnections) {
            connections.push({ ...conn, depth, path: [...path, currentId] });
            await traverse(conn.id, depth + 1, [...path, currentId]);
        }
    }

    await traverse(personId, 1, []);
    return connections;
}
```

---

## Vector Database Features

### Vector Storage and Indexing

**Define Vector Table with HNSW Index**:

```sql
DEFINE TABLE document SCHEMAFULL;
DEFINE FIELD title ON TABLE document TYPE string;
DEFINE FIELD content ON TABLE document TYPE string;
DEFINE FIELD embedding ON TABLE document TYPE array<float>;

-- HNSW index for approximate nearest neighbor search
DEFINE INDEX document_embedding_idx ON document
    FIELDS embedding HNSW DIMENSION 1536 DISTANCE COSINE;
```

**Supported Distance Metrics**:

| Metric | Use Case |
|--------|----------|
| COSINE | Text embeddings, normalized vectors |
| EUCLIDEAN | Spatial data, image features |
| MANHATTAN | High-dimensional sparse data |

**Rust Example**:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Document {
    title: String,
    content: String,
    embedding: Vec<f32>,
}

async fn store_document(db: &Surreal<Any>, doc: Document) -> surrealdb::Result<()> {
    db.create("document").content(doc).await?;
    Ok(())
}
```

### Similarity Search

**Basic Vector Search**:

```sql
SELECT
    id, title, content,
    vector::distance::cosine(embedding, $query_embedding) AS distance
FROM document
WHERE embedding <|5, COSINE|> $query_embedding
ORDER BY distance
LIMIT 10;
```

**Rust Implementation**:

```rust
#[derive(Debug, Deserialize)]
struct SearchResult {
    id: RecordId,
    title: String,
    content: String,
    distance: f32,
}

async fn semantic_search(
    db: &Surreal<Any>,
    query_embedding: Vec<f32>,
    limit: u32
) -> surrealdb::Result<Vec<SearchResult>> {
    let mut response = db.query("
        SELECT
            id, title, content,
            vector::distance::cosine(embedding, $embedding) AS distance
        FROM document
        WHERE embedding <|$limit, COSINE|> $embedding
        ORDER BY distance
    ")
    .bind(("embedding", query_embedding))
    .bind(("limit", limit))
    .await?;

    response.take(0)
}
```

### Hybrid Search

Combine vector similarity with metadata filtering:

```sql
SELECT
    id, title, content,
    vector::distance::cosine(embedding, $embedding) AS vector_distance
FROM document
WHERE
    embedding <|5, COSINE|> $embedding
    AND category = $category
    AND created_at > $date_threshold
ORDER BY vector_distance
LIMIT 10;
```

**TypeScript Example**:

```typescript
const hybridSearch = async (
    queryEmbedding: number[],
    category: string,
    dateThreshold: string
) => {
    const [results] = await db.query(`
        SELECT
            id, title, content,
            vector::distance::cosine(embedding, $embedding) AS distance
        FROM document
        WHERE
            embedding <|5, COSINE|> $embedding
            AND category = $category
            AND created_at > $dateThreshold
        ORDER BY distance
        LIMIT 10
    `, { embedding: queryEmbedding, category, dateThreshold });

    return results;
};
```

---

## ORM and Abstractions

### Rust ORM Options

**Official SDK Methods** (Mid-level API):

```rust
// Fluent methods for common operations
let person: Option<Person> = db.select(("person", "john")).await?;
db.create(("person", "jane")).content(new_person).await?;
db.update(("person", "john")).merge(updates).await?;
db.delete(("person", "john")).await?;
```

**Community ORM: `surreal_orm`** (High-level):

```rust
use surreal_orm::*;

#[derive(Node, Serialize, Deserialize, Debug, Clone)]
#[orm(table = "space_ship")]
pub struct SpaceShip {
    pub id: SurrealSimpleId<Self>,
    pub name: String,
    pub age: u8,
}

// Type-safe query building
let spaceships = select(All)
    .from(SpaceShip::table())
    .where_(
        cond(SpaceShip::schema().age.greater_than(10))
            .and(SpaceShip::schema().name.like("%Enterprise%"))
    )
    .return_many::<SpaceShip>(db.clone())
    .await?;
```

### TypeScript ORM Options

**Community ORM: `@surrealorm/orm`** (Experimental):

> **Warning:** SurrealORM is explicitly marked as "not yet ready for production use."

```typescript
import { Entity, BaseEntity, Property, SurrealORM } from '@surrealorm/orm';

@Entity()
class User extends BaseEntity {
    @Property({ unique: true })
    email!: string;

    @Property({ required: true })
    name!: string;
}

const orm = new SurrealORM({
    url: 'http://localhost:8000',
    namespace: 'app',
    database: 'production'
});

await orm.connect();
const user = await orm.findUnique(User, { email: 'jane@example.com' });
```

### When to Use ORM vs Raw Queries

| Scenario | Recommendation |
|----------|----------------|
| Production applications | Official SDK (stable, supported) |
| Complex SurrealQL features | Raw queries |
| Type safety is critical | ORM or SDK with TypeScript/Zod |
| Graph traversals | Raw queries (ORM support limited) |
| Rapid prototyping | ORM (faster development) |
| Performance-critical paths | Raw queries (no abstraction overhead) |

---

## Migration Tools

### Tool Comparison

| Tool | Type | Key Feature | Best For |
|------|------|-------------|----------|
| **surrealdb-migrate** | CLI + Library | Verify/revert, rich config | Production |
| **surrealdb-migrations** | CLI + Library | Timestamp-based, schema/events | Prototyping |
| **surrealdb-migrator** | Library only | Lightweight, code-defined | Embedded |
| **surrealdb_migration_engine** | Library only | Rust_embed, auto-detection | Single-binary |

### surrealdb-migrate

**Installation**:

```bash
cargo install surrealdb-migrate-cli
```

**Configuration (surrealdb-migrate.toml)**:

```toml
[database]
address = "ws://localhost:8000"
namespace = "myapp"
database = "production"
auth_level = "Database"
username = "admin"
password = "s3cr3t"

[migrations]
folder = "./migrations"
ignore_order = false
```

**Migration File Structure**:

```
migrations/
├── 01-initial_schema/
│   └── up.surql
├── 02-add_user_roles/
│   ├── up.surql
│   └── down.surql
```

**Rust Integration**:

```rust
use surrealdb_migrate::{
    config::{DbAuthLevel, DbClientConfig, RunnerConfig},
    runner::MigrationRunner,
};

let runner = MigrationRunner::new(RunnerConfig::default()
    .with_migrations_folder(Path::new("./migrations")));

runner.migrate(&db).await?;
runner.revert(&db, 2).await?;  // Revert last 2
```

### Migration Best Practices

1. **Always use transactions** for multi-step migrations
2. **Make migrations idempotent** - safe to run multiple times
3. **Test rollbacks** before production deployment
4. **Keep migrations small** - one logical change per file
5. **Use sequential numbering** in team environments
6. **Backup before production migrations**

---

## Tooling

### Surrealist GUI

Surrealist is the official graphical interface for SurrealDB, available as web app and desktop client.

**Key Features**:

- **Designer View**: Interactive schema diagrams with real-time visualization
- **Explorer View**: Browse tables, inspect records, follow graph relationships
- **Query Editor**: SurrealQL syntax highlighting, saved queries, live results
- **Graph Visualization**: Transform query results into node-edge diagrams
- **AI Assistant**: "Ask Sidekick" for query generation and schema design
- **Cloud Management**: SurrealDB Cloud provisioning and monitoring

**Use Cases**:

| Use Case | Surrealist Feature |
|----------|-------------------|
| Schema design | Designer View with drag-and-drop |
| Query testing | Query Editor with sandboxed environment |
| Data exploration | Explorer View with record inspection |
| Relationship analysis | Graph visualization |
| Multi-database management | Instance switching panel |

---

## Common Gotchas and Solutions

### SDK-Specific Issues

#### Query Response Error Handling (Rust)

**Problem**: SDK returns `Ok(Response)` even when individual statements fail.

```rust
// This returns Ok even though second statement fails!
let res = db.query("
    LET $x = 9;
    LET $x: string = 9;  // Type check fails
").await?;
```

**Solution**: Always check for errors:

```rust
let mut response = db.query("...").await?;

// Check all errors
if let Err(errors) = response.check() {
    eprintln!("Query errors: {:?}", errors);
}

// Or extract errors manually
let errors = response.take_errors();
```

#### RecordId Double-Wrapping (TypeScript)

**Problem**: Passing RecordId to RecordId constructor creates nested IDs.

```typescript
// WRONG - causes silent failures
const id = new RecordId('User', someRecordIdOrString);
```

**Solution**: Type guard before wrapping:

```typescript
function ensureRecordId(table: string, id: string | RecordId): RecordId {
    if (id instanceof RecordId) return id;
    return new RecordId(table, id);
}
```

#### Serialization with Nested Structs (Rust)

**Problem**: Flattening structs with RecordId causes serialization errors.

```rust
// This causes issues!
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: RecordId,
    #[serde(flatten)]
    inner: UserData,  // Error with flatten + RecordId
}
```

**Solution**: Avoid flattening with RecordId or use manual conversion:

```rust
// Option 1: Inline fields directly
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: RecordId,
    pub name: String,  // Inline instead of flatten
}

// Option 2: Use Value as intermediate
let value: Value = response.take(0)?;
let user = serde_json::from_value::<User>(value.into())?;
```

### Query and Performance Issues

#### Deep Traversal Performance

**Problem**: Graph traversals become slow with increasing depth.

**Solutions**:

1. **Filter at each step**:

```sql
-- Instead of this (slow):
SELECT * FROM person:tobie->{1..10}(->knows->person);

-- Use this (faster):
SELECT * FROM person:tobie->{1..10}(
    ->knows->person WHERE active = true AND last_login > time::now() - 30d
);
```

2. **Create appropriate indexes**:

```sql
DEFINE INDEX idx_person_active ON person FIELDS active;
DEFINE INDEX idx_bought_in ON bought FIELDS in;
DEFINE INDEX idx_bought_out ON bought FIELDS out;
```

3. **Use EXPLAIN to verify index usage**:

```sql
EXPLAIN SELECT * FROM person WHERE age > 30;
```

#### Vector Dimension Mismatch

**Problem**: Index dimensions don't match embedding dimensions.

```sql
-- Index defined for 768 dimensions
DEFINE INDEX bad_idx ON document FIELDS embedding HNSW DIMENSION 768;
-- But embeddings have 1536 dimensions - causes errors
```

**Solution**: Validate dimensions before storing:

```rust
const MAX_DIMENSIONS: usize = 16384;

fn validate_embedding(embedding: &[f32]) -> Result<(), String> {
    if embedding.len() > MAX_DIMENSIONS {
        return Err(format!("Dimension {} exceeds max {}",
            embedding.len(), MAX_DIMENSIONS));
    }
    Ok(())
}
```

### Authentication Issues

**Problem**: Generic "InvalidAuth" errors without details.

**Debugging Steps**:

1. Enable debug logging on server: `surreal start --log debug`
2. For development, enable error forwarding:
   ```bash
   export SURREAL_INSECURE_FORWARD_ACCESS_ERRORS=true
   ```
3. Verify credentials in CLI first:
   ```bash
   surreal sql --conn ws://localhost:8000 --user root --pass root
   ```

**Solution**: Explicit auth level and verification:

```rust
// Verify connection first
let health = db.query("RETURN true").await;
match health {
    Ok(_) => println!("Connection healthy"),
    Err(e) => eprintln!("Connection failed: {}", e),
}
```

---

## Production Patterns

### Error Handling

**Rust with thiserror**:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),
    #[error("Not found")]
    NotFound,
    #[error("Constraint violation: {0}")]
    Constraint(String),
}

impl From<surrealdb::Error> for RepositoryError {
    fn from(err: surrealdb::Error) -> Self {
        match err {
            surrealdb::Error::Db(Db::RecordExists { .. }) => {
                RepositoryError::Constraint("Record already exists".into())
            }
            surrealdb::Error::Db(Db::RecordNotFound) => RepositoryError::NotFound,
            other => RepositoryError::Database(other),
        }
    }
}
```

### Connection Pooling

**Global Connection (Rust)**:

```rust
use once_cell::sync::OnceCell;
use std::sync::Arc;

static DB: OnceCell<Arc<Surreal<Any>>> = OnceCell::new();

async fn get_db() -> Arc<Surreal<Any>> {
    DB.get_or_init(|| async {
        let db = connect("ws://localhost:8000").await.unwrap();
        db.signin(Root { username: "root", password: "secret" }).await.unwrap();
        db.use_ns("prod").use_db("app").await.unwrap();
        Arc::new(db)
    }).await.clone()
}
```

### Performance Optimization

1. **Use WebSocket protocol** for persistent connections
2. **Batch operations** with multi-statement queries
3. **Use FETCH** instead of N+1 queries:
   ```sql
   SELECT * FROM post FETCH author, comments;
   ```
4. **Index frequently queried fields**
5. **Limit returned fields** instead of SELECT *
6. **Use transactions** for related operations

**Environment Variables for Production**:

```bash
# Connection limits
export SURREAL_WEBSOCKET_MAX_CONCURRENT_REQUESTS=50
export SURREAL_NET_MAX_CONCURRENT_REQUESTS=10000

# Memory limits
export SURREAL_SCRIPTING_MAX_MEMORY_LIMIT=10485760
export SURREAL_TRANSACTION_CACHE_SIZE=1000

# Security
export SURREAL_INSECURE_FORWARD_ACCESS_ERRORS=false
```

---

## Quick Reference

### SurrealQL Cheat Sheet

| Operation | Syntax |
|-----------|--------|
| Create | `CREATE table:id SET field = value` |
| Select | `SELECT * FROM table WHERE condition` |
| Update | `UPDATE table:id SET field = value` |
| Delete | `DELETE table:id` |
| Relate | `RELATE from->edge->to SET props` |
| Traverse Out | `SELECT * FROM node->edge->target` |
| Traverse In | `SELECT * FROM target<-edge<-node` |
| Vector Search | `WHERE embedding <\|k, METRIC\|> $vec` |
| Explain | `EXPLAIN SELECT ...` |

### Distance Metrics

| Metric | Function | Use Case |
|--------|----------|----------|
| Cosine | `vector::distance::cosine()` | Text embeddings |
| Euclidean | `vector::distance::euclidean()` | Spatial data |
| Manhattan | `vector::distance::manhattan()` | Sparse vectors |

---

## Resources

### Official Documentation

- [SurrealDB Documentation](https://surrealdb.com/docs)
- [Rust SDK Documentation](https://surrealdb.com/docs/sdk/rust)
- [JavaScript SDK Documentation](https://surrealdb.com/docs/sdk/javascript)
- [SurrealQL Reference](https://surrealdb.com/docs/surrealql)

### Video Tutorials

**Rust**:
- [Getting Started with SurrealDB Rust SDK](https://www.youtube.com/watch?v=I1cuddL6A8o) - Official quick start (2024)
- [Building Rust Backend with Actix Web and SurrealDB](https://www.youtube.com/watch?v=PspYOlksxv0)

**TypeScript**:
- [Getting Started with JavaScript SDK](https://www.youtube.com/watch?v=fLC0_dlgIrs) - Official guide
- [Run SurrealDB inside Node.js](https://www.youtube.com/watch?v=uW_9Hqg7N4Q) - Embedded engine tutorial

### Community Resources

- [Surreal ORM (Rust)](https://github.com/Oyelowo/surreal_orm) - Community ORM
- [SurrealDB Discord](https://discord.gg/surrealdb) - Community support
- [Surrealist](https://surrealist.app) - Official GUI tool
