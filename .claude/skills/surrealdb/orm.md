# SurrealDB ORM Options

SurrealDB provides multiple abstraction levels: raw SurrealQL queries, fluent SDK methods, and community ORM libraries.

## API Levels Overview

| Level | Rust | TypeScript | When to Use |
|-------|------|------------|-------------|
| **Raw Queries** | `.query()` | `.query()` | Complex queries, dynamic SQL, full SurrealQL features |
| **Fluent Methods** | `.select()`, `.create()` | `.select()`, `.create()` | Simple CRUD, type-safe operations |
| **ORM** | `surreal_orm` | `@surrealorm/orm` | Complex domain models, compile-time safety |

## Rust: surreal_orm

**Status**: Community-maintained, experimental

### Setup

```toml
[dependencies]
surreal_orm = { git = "https://github.com/Oyelowo/surreal_orm" }
surrealdb = { version = "2", features = ["kv-mem"] }
```

### Model Definition

```rust
use surreal_orm::*;
use serde::{Serialize, Deserialize};

#[derive(Node, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[orm(table = "space_ship")]
pub struct SpaceShip {
    pub id: SurrealSimpleId<Self>,
    pub name: String,
    pub crew_count: u32,
}

#[derive(Node, Serialize, Deserialize, Debug, Clone)]
#[orm(table = "pilot")]
pub struct Pilot {
    pub id: SurrealId<Self, String>,
    pub name: String,
    pub rank: String,
}
```

### Type-Safe Queries

```rust
// Instead of string queries, use type-safe builders
let ships = select(All)
    .from(SpaceShip::table())
    .where_(
        cond(SpaceShip::schema().crew_count.greater_than(10))
            .and(SpaceShip::schema().name.like("%Enterprise%"))
    )
    .return_many::<SpaceShip>(db.clone())
    .await?;
```

### Transactions

```rust
let id1 = &Account::create_id("one".into());
let id2 = &Account::create_id("two".into());
let acc = Account::schema();

transaction! {
    BEGIN TRANSACTION;

    create_only().content(Account {
        id: id1.clone(),
        balance: 1000.00,
    });

    update::<Account>(id1).set(acc.balance.increment_by(500.00));

    COMMIT TRANSACTION;
}
.run(db.clone())
.await?;
```

## TypeScript: @surrealorm/orm

**Status**: Experimental, NOT production-ready

### Setup

```bash
npm install @surrealorm/orm
```

### Entity Definition

```typescript
import { Entity, BaseEntity, Property } from '@surrealorm/orm';

@Entity()
class User extends BaseEntity {
  @Property({ unique: true })
  email!: string;

  @Property({ required: true })
  name!: string;

  @Property({ type: 'boolean', default: true })
  isActive?: boolean;
}
```

### CRUD Operations

```typescript
// Create
const user = new User();
user.email = 'jane@example.com';
user.name = 'Jane';
await orm.create(user);

// Find
const found = await orm.findUnique(User, { email: 'jane@example.com' });
const all = await orm.findAll(User);

// Update
found.name = 'Jane Doe';
await orm.update(found);

// Delete
await orm.delete(found);
```

## When to Use What

### Use ORM When

- **Type safety critical** - Compile-time verification of queries
- **Complex domain models** - Many entity relationships
- **Team productivity** - Less SQL knowledge required
- **IDE support** - Better autocomplete and refactoring

### Use Raw Queries When

- **Performance critical** - No abstraction overhead
- **Dynamic queries** - Runtime query construction
- **SurrealQL-specific features** - Graph traversals, vector search, live queries
- **Complex aggregations** - CTEs, subqueries, advanced functions

### Hybrid Approach (Recommended)

```rust
impl UserRepository {
    // ORM for common CRUD
    async fn find_active(&self) -> Result<Vec<User>> {
        select(All).from(User::table())
            .where_(User::schema().status.eq("active"))
            .return_many::<User>(self.db.clone()).await
    }

    // Raw query for complex operations
    async fn get_usage_stats(&self) -> Result<Value> {
        let mut result = self.db.query(
            "SELECT count() FROM user GROUP BY status"
        ).await?;
        result.take(0)
    }
}
```

## Repository Pattern with SDK

For production without full ORM, use the repository pattern:

```typescript
// TypeScript Repository
class UserRepository {
  private table = 'User';

  async create(data: Omit<User, 'id'>): Promise<User> {
    const db = await getDb();
    const [user] = await db.create<User>(this.table, data);
    return user;
  }

  async findByEmail(email: string): Promise<User | null> {
    const db = await getDb();
    const [result] = await db.query<User[]>(
      `SELECT * FROM ${this.table} WHERE email = $email LIMIT 1`,
      { email }
    );
    return result?.[0] || null;
  }

  // Graph operations via raw queries
  async getUserFriends(userId: string): Promise<User[]> {
    const db = await getDb();
    const [friends] = await db.query<User[]>(
      `SELECT ->friends->User.* FROM ${this.table}:${userId}`
    );
    return friends || [];
  }
}
```

## Common Gotchas

### Type Coercion Errors

```rust
// WRONG - type mismatch
db.query("LET $x: string = 9").await?;

// CORRECT - explicit conversion
db.query("LET $x = type::string(9)").await?;

// Or bind correct type
db.query("LET $x: string = $value")
    .bind(("value", "9"))  // String, not number
    .await?;
```

### RELATE Statement Syntax

```rust
// WRONG - spacing issues
let sql = format!("RELATE users:{}->owns->products:{}", uid, pid);

// CORRECT - use parameters
let sql = "RELATE $from -> owns -> $to";
db.query(sql)
    .bind(("from", user_id))
    .bind(("to", product_id))
    .await?;
```

### Live Query Type Inference

```rust
// WRONG - type cannot be inferred
let stream = db.select("person").live().await?;

// CORRECT - explicit type
let stream: LiveQuery<Person> = db.select("person").live().await?;
// Or use turbofish
let stream = db.select("person").live::<Person>().await?;
```

## ORM vs Query API Comparison

| Feature | ORM | Query API |
|---------|-----|-----------|
| Type Safety | Compile-time | Runtime |
| Learning Curve | Steeper | Gentle (SQL-like) |
| Flexibility | Limited to ORM features | Full SurrealQL |
| Performance | Slight overhead | Direct execution |
| Maintenance | Community-driven | Official support |

## Migration Strategy

If starting with ORM but may need to migrate:

1. **Phase 1**: Use ORM for rapid prototyping
2. **Phase 2**: Extract interfaces from entities
3. **Phase 3**: Implement repository pattern over SDK
4. **Phase 4**: Replace ORM calls with SDK calls

```typescript
// ORM Entity (Phase 1)
@Entity() class User extends BaseEntity { /* ... */ }

// Extracted Interface (Phase 2)
interface IUser { id: RecordId; email: string; name: string; }

// Repository (Phase 3-4)
class UserRepository {
  async findById(id: string): Promise<IUser | null> {
    const db = await getDb();
    return await db.select(new RecordId('User', id));
  }
}
```

## Related

- [Rust SDK](./sdk-rust.md)
- [TypeScript SDK](./sdk-typescript.md)
- [Migrations](./migrations.md)
