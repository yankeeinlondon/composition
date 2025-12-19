# SurrealDB as Graph Database

SurrealDB treats relationships as first-class citizens using semantic triples: `subject -> predicate -> object`. Edges are real tables with properties, enabling rich relationship modeling.

## Core Concepts

### Semantic Triples

```
node -> edge -> node
person:alice -> knows -> person:bob
```

### Record Links vs Graph Relations

| Approach | Use Case | Example |
|----------|----------|---------|
| **Record Links** | Simple foreign keys, no edge properties | `author: user:john` |
| **Graph Relations** | Relationships with properties | `RELATE user:john->wrote->post:1 SET date = time::now()` |

## Creating Relationships

### RELATE Statement

```sql
-- Create edge with properties
RELATE person:alice->knows->person:bob
SET since = time::now(), metThrough = 'work', strength = 0.8;

-- The edge table 'knows' gets automatic 'in' and 'out' fields
SELECT * FROM knows;
-- Returns: { id: knows:xyz, in: person:alice, out: person:bob, since: ..., ... }
```

### Rust Example

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Knows {
    since: String,
    met_through: String,
}

// Create relationship
let relation: Option<RecordId> = db
    .query("RELATE $from->knows->$to CONTENT $props")
    .bind(("from", RecordId::from(("person", "alice"))))
    .bind(("to", RecordId::from(("person", "bob"))))
    .bind(("props", Knows { since: "2024-01-01".into(), met_through: "work".into() }))
    .await?
    .take("id")?;
```

### TypeScript Example

```typescript
await db.relate(
  "person:alice",
  "knows",
  "person:bob",
  { since: "2024-01-01", metThrough: "work" }
);
```

## Graph Traversals

### Basic Traversals

```sql
-- Outgoing: Who does Alice know?
SELECT * FROM person:alice->knows->person;

-- Incoming: Who knows Alice?
SELECT * FROM person<-knows<-person:alice;

-- Include edge properties
SELECT
    out.name AS friend_name,
    since,
    metThrough
FROM person:alice->knows;

-- Multi-hop: Friends of friends
SELECT * FROM person:alice->knows->person->knows->person;
```

### Recursive Traversals (Depth-Limited)

```sql
-- Find people within N degrees
SELECT * FROM person:alice.{1..3}(->knows->person);

-- With filtering at each hop
SELECT * FROM person:alice.{1..5}(
    ->knows->person WHERE active = true AND last_login > time::now() - 30d
);
```

### Shortest Path (Simplified)

```sql
-- Find if path exists within max depth
SELECT VALUE $target FROM person:alice.{1..5}(->knows->person)
WHERE $target = person:bob LIMIT 1;
```

## Common Patterns

### Mutual Friends

```sql
LET $alice_friends = (SELECT VALUE out FROM person:alice->knows);
LET $bob_friends = (SELECT VALUE out FROM person:bob->knows);

SELECT * FROM person
WHERE id INSIDE $alice_friends AND id INSIDE $bob_friends;
```

### Recommendation Engine

```sql
-- Find products bought by similar users
LET $my_products = (SELECT VALUE out.id FROM $user_id->bought->product);
LET $similar_users = (
    SELECT in.id, COUNT() as common
    FROM person<-bought->product<-bought->person
    WHERE out.id IN $my_products AND in.id != $user_id
    GROUP BY in.id ORDER BY common DESC LIMIT 10
);

SELECT * FROM $similar_users->bought->product
WHERE out.id NOT IN $my_products
GROUP BY out.id ORDER BY COUNT() DESC LIMIT 10;
```

### Fraud Detection

```sql
-- Detect circular payment patterns
SELECT circle FROM account:suspect.{3..5}(->sent->account) AS circle
WHERE account:suspect IN circle;

-- Rapid transactions to multiple accounts
SELECT VALUE out.id FROM $account->sent->transaction
WHERE timestamp > time::now() - 1h
GROUP BY out HAVING count() > 10;
```

## Indexing for Performance

```sql
-- Index traversal directions
DEFINE INDEX idx_knows_in ON knows FIELDS in;
DEFINE INDEX idx_knows_out ON knows FIELDS out;

-- Composite index for filtered traversals
DEFINE INDEX idx_knows_since ON knows FIELDS since, in;
```

## Common Gotchas

### No Native Recursive Queries

SurrealDB doesn't support variable-length paths like Cypher. Use:

1. Fixed-depth traversals: `.{1..5}(->edge->node)`
2. Application-level recursion for arbitrary depth

### RELATE Syntax Spacing

```sql
-- WRONG
RELATE users:john->owns->products:abc;  -- May cause parsing errors

-- CORRECT - explicit spacing
RELATE users:john -> owns -> products:abc;

-- BEST - use parameters
RELATE $from -> owns -> $to;
```

### Performance with Deep Traversals

- Always filter at each traversal step
- Limit traversal depth
- Index frequently traversed fields
- Use `EXPLAIN` to verify index usage:

```sql
EXPLAIN SELECT * FROM person:alice->knows->person WHERE active = true;
```

### Embedding vs Relations Decision

| Use Relations When | Use Embedding When |
|-------------------|-------------------|
| Data accessed independently | Data always accessed together |
| Need to query the relationship | No independent lifecycle |
| Multiple entity types connected | Atomic updates needed |
| Edge properties required | Simple containment |

## TypeScript Graph Example

```typescript
// Find friends of friends excluding direct friends
const findFoF = async (personId: string) => {
  const [directFriends] = await db.query<string[]>(
    "SELECT VALUE out FROM $id->knows",
    { id: personId }
  );

  const [fof] = await db.query<Person[]>(`
    SELECT DISTINCT out.* FROM $id->knows->person->knows
    WHERE out.id NOT IN $direct AND out.id != $id
  `, { id: personId, direct: directFriends });

  return fof;
};
```

## Related

- [Vector Database](./vector-database.md) - Combine graph with semantic search
- [Rust SDK](./sdk-rust.md)
- [TypeScript SDK](./sdk-typescript.md)
