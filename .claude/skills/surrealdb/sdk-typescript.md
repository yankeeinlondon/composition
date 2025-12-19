# SurrealDB TypeScript SDK

The official TypeScript SDK supports browser and Node.js environments with WebSocket, HTTP, and embedded (WASM/Node engine) connections.

## Setup

```bash
npm install surrealdb
# For embedded Node.js engine
npm install @surrealdb/node
```

## Connection Patterns

```typescript
import Surreal from 'surrealdb';

// Basic WebSocket connection
const db = new Surreal();
await db.connect("ws://localhost:8000/rpc");
await db.use({ namespace: "test", database: "test" });
await db.signin({ username: "root", password: "root" });

// With embedded Node engine
import { surrealdbNodeEngines } from '@surrealdb/node';

const db = new Surreal({ engines: surrealdbNodeEngines() });
await db.connect("surrealkv://./data.db");  // Persistent
// Or: await db.connect("mem://");  // In-memory
```

## Type-Safe CRUD

```typescript
interface User {
  id?: string;
  username: string;
  email: string;
  metadata?: Record<string, unknown>;
}

// Create
const user = await db.create<User>("User", {
  username: "johndoe",
  email: "john@example.com"
});

// Select all
const users = await db.select<User>("User");

// Select specific record
const user = await db.select<User>("User:john");

// Update (merge)
await db.merge("User:john", { email: "newemail@example.com" });

// Delete
await db.delete("User:john");
```

## Parameterized Queries

```typescript
// CORRECT - always use parameters
const [users] = await db.query<User[]>(
  "SELECT * FROM User WHERE email CONTAINS $domain",
  { domain: "@example.com" }
);

// WRONG - SQL injection vulnerable
await db.query(`SELECT * FROM User WHERE name = "${userInput}"`);  // NEVER!
```

## Live Queries

```typescript
const queryId = await db.live("User", (notification) => {
  switch (notification.action) {
    case "CREATE":
      console.log("New user:", notification.result);
      break;
    case "UPDATE":
      console.log("Updated:", notification.result);
      break;
    case "DELETE":
      console.log("Deleted:", notification.id);
      break;
  }
});

// Stop listening
await db.kill(queryId);
```

## Singleton Pattern (Recommended)

```typescript
class DatabaseService {
  private static instance: Surreal;
  private static connecting: Promise<Surreal> | null = null;

  static async getConnection(): Promise<Surreal> {
    if (this.instance?.status === "connected") {
      return this.instance;
    }

    if (this.connecting) {
      return this.connecting;
    }

    this.connecting = (async () => {
      this.instance = new Surreal();
      await this.instance.connect("ws://localhost:8000/rpc");
      await this.instance.use({ namespace: "app", database: "prod" });
      await this.instance.signin({ username: "root", password: "root" });
      return this.instance;
    })();

    return this.connecting;
  }

  static async close() {
    if (this.instance) {
      await this.instance.close();
      this.instance = null!;
      this.connecting = null;
    }
  }
}
```

## Runtime Validation with Zod

TypeScript types don't guarantee database schema matches. Use Zod:

```typescript
import { z } from 'zod';

const UserSchema = z.object({
  id: z.string().optional(),
  username: z.string(),
  email: z.string().email(),
  age: z.number().int().positive()
});

type User = z.infer<typeof UserSchema>;

async function safeSelect(id: string): Promise<User | null> {
  const result = await db.select<User>(`User:${id}`);
  const validated = UserSchema.safeParse(result[0]);
  return validated.success ? validated.data : null;
}
```

## Transactions

```typescript
try {
  await db.query("BEGIN");
  await db.query("UPDATE account:from SET balance -= $amount", { amount: 100 });
  await db.query("UPDATE account:to SET balance += $amount", { amount: 100 });
  await db.query("COMMIT");
} catch (err) {
  await db.query("ROLLBACK");
  throw err;
}
```

## Common Gotchas

### RecordId Double-Wrapping

```typescript
import { RecordId } from 'surrealdb';

// WRONG - may double-wrap
const id = new RecordId('User', someIdThatMightBeRecordId);

// CORRECT - check first
function ensureRecordId(table: string, id: string | RecordId): RecordId {
  return id instanceof RecordId ? id : new RecordId(table, id);
}
```

### DELETE ONLY Returns Error

```typescript
// WRONG - always fails
await db.query("DELETE ONLY User:123");

// CORRECT - add RETURN clause
const [deleted] = await db.query<User>("DELETE ONLY User:123 RETURN $before");
```

### Authentication Token Refresh

```typescript
// Enable automatic token refresh (SDK 2.0+)
await db.connect("ws://localhost:8000/rpc", {
  namespace: "test",
  database: "test",
  renewAccess: true,
  authentication: async () => {
    const token = await getFreshToken();
    return { token };
  }
});
```

### React Cleanup

```typescript
useEffect(() => {
  let isMounted = true;
  const db = new Surreal();

  db.connect("ws://localhost:8000/rpc").then(() => {
    if (isMounted) {
      // Perform operations
    }
  });

  return () => {
    isMounted = false;
    db.close();
  };
}, []);
```

## Environment Variables

```bash
# Connection limits
SURREAL_WEBSOCKET_MAX_CONCURRENT_REQUESTS=50
SURREAL_WEBSOCKET_MAX_MESSAGE_SIZE=134217728

# Memory limits
SURREAL_SCRIPTING_MAX_MEMORY_LIMIT=10485760
SURREAL_TRANSACTION_CACHE_SIZE=1000

# Debug auth (development only!)
SURREAL_INSECURE_FORWARD_ACCESS_ERRORS=true
```

## Node Engine Notes

- Requires ES Modules: `"type": "module"` in package.json
- Use `surrealkv://` for persistence, `mem://` for testing
- Versioned queries: `surrealkv+versioned://` for temporal data

## Related

- [Rust SDK](./sdk-rust.md)
- [Graph Database](./graph-database.md)
- [ORM Options](./orm.md)
