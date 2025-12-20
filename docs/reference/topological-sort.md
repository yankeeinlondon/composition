This is a classic computer science problem called **Topological Sort**, specifically a "layered" or "grouped" topological sort.

You are 100% correct in your intuition: "Leaf nodes" (dependencies) go first, then the items that depended on them, and so on. In Graph Theory, this specific approach of peeling off the layers of a graph is known as **Kahn’s Algorithm**.

Here is how to structure this in SurrealDB and Rust to work with `rayon`.

---

### 1. The Theory: Directed Acyclic Graphs (DAG)

Your documents and their dependencies form a **DAG** (Directed Acyclic Graph).

* **Directed:** Dependency goes one way (A needs B).
* **Acyclic:** You cannot have loops (A needs B, B needs A).

The goal of your `get_work_schedule` function is to slice this graph into **Generations** (or Layers).

* **Generation 0:** Documents with **0** incomplete dependencies (Leaves).
* **Generation 1:** Documents that only depend on Generation 0.
* **Generation 2:** Documents that depend on Gen 0 and 1.

All documents in the **same generation** can be processed in parallel by `rayon`.

---

### 2. The SurrealDB Schema

We need a way to track the `status` of a document so the database knows which dependencies are actually finished.

```surrealql
-- 1. The Document Node
-- We add a 'status' field to track progress (PENDING, DONE)
DEFINE TABLE document SCHEMAFULL;
DEFINE FIELD status ON TABLE document TYPE string DEFAULT 'PENDING';

-- 2. The Dependency Edge
-- logical direction: "in" (the Child) -> depends_on -> "out" (the Parent)
-- Example: Report -> depends_on -> DataSheet
DEFINE TABLE depends_on SCHEMAFULL;
DEFINE FIELD in ON TABLE depends_on TYPE record<document>;
DEFINE FIELD out ON TABLE depends_on TYPE record<document>;

-- 3. Indexes for performance
DEFINE INDEX idx_status ON TABLE document COLUMNS status;
DEFINE INDEX idx_in ON TABLE depends_on COLUMNS in;
DEFINE INDEX idx_out ON TABLE depends_on COLUMNS out;

```

---

### 3. The "Next Batch" Query

Instead of trying to calculate the *entire* schedule upfront (which is complex and brittle if a job fails), the most robust pattern for a task runner is **Iterative State**.

You ask the database: *"Give me all PENDING documents that usually have dependencies, but currently have **ZERO** dependencies that are still PENDING."*

This is the query you run at the start of every loop iteration:

```surrealql
SELECT * FROM document
WHERE
    -- 1. We only want work that isn't finished
    status = 'PENDING'
    AND
    -- 2. Check that it has NO dependencies that are NOT done
    count(->depends_on[WHERE out.status != 'DONE']) = 0;

```

**How this works:**

1. **Generation 0:** The query finds the true "leaf nodes" (documents that point to nothing, or point only to things already marked 'DONE').
2. **Processing:** You run these in `rayon`.
3. **Update:** You mark them as `DONE`.
4. **Generation 1:** You run the exact same query. The documents that were waiting on Gen 0 now see 0 *pending* dependencies, so they are returned.

---

### 4. Rust Implementation Logic

Here is how you implement `get_work_schedule` (or rather, `process_work`) using the iterative approach. This handles the graph traversal naturally.

```rust
use rayon::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Document {
    id: String,
    // other fields...
}

// Pseudocode for your worker function
async fn execute_render_job(db: &Surreal<Client>) {
    loop {
        // 1. Fetch the next "Layer" of parallelizable work
        //    (Using the query defined in step 3)
        let batch: Vec<Document> = db
            .query("SELECT * FROM document WHERE status = 'PENDING' AND count(->depends_on[WHERE out.status != 'DONE']) = 0")
            .await
            .unwrap()
            .take(0)
            .unwrap();

        // 2. If no work is returned, we are either finished or deadlocked
        if batch.is_empty() {
            // Check if there are still pending items (which would imply a cycle/circular dependency error)
            break;
        }

        // 3. Process this layer in parallel using Rayon
        //    This blocks until the whole batch is finished
        batch.par_iter().for_each(|doc| {
            println!("Rendering doc: {}", doc.id);
            // render_document(doc);
        });

        // 4. Mark this batch as DONE in SurrealDB
        //    (So the next layer becomes available in the next loop)
        let ids: Vec<String> = batch.iter().map(|d| d.id.clone()).collect();

        db.query("UPDATE $ids SET status = 'DONE'")
            .bind(("ids", ids))
            .await
            .unwrap();
    }
}

```

### Why this approach?

1. **Simplicity:** You don't need complex Graph Theory algorithms in your Rust code. The database logic (`count(...) = 0`) handles the topological sorting for you automatically.
2. **Resilience:** If the program crashes halfway through, you don't lose the schedule. You just restart, and the database knows exactly what is `DONE` and what is `PENDING`.
3. **Concurrency:** It perfectly aligns with `rayon`. You fetch a group, fan out (parallel), join (wait), and update.

## Isolating to a specific Root Document

To isolate your schedule to a specific "Root Document" (so you don't accidentally process unrelated documents), we can use a **Recursive Graph Traversal**. This ensures that we only look at the specific tree or "Forest" that supports your target document.

### 1. The Strategy: "Sub-Graph Isolation"

When you call `get_work_schedule(target_doc)`, we want to:

1. Identify all documents that the target document depends on (the transitive closure).
2. Among *those* documents, find which ones have zero **incomplete** dependencies.

### 2. The Refined SurrealQL Query

This query uses the `..` operator, which tells SurrealDB to traverse the `depends_on` relationship recursively to any depth.

```surrealql
-- Get all dependencies for a specific document
-- then filter for the ones that can be worked on right now.
LET $tree = (
    SELECT VALUE out FROM (
        -- Traverse 'out' (parents/dependencies) recursively starting from our target
        SELECT ->depends_on->document AS out FROM type::record($target_doc)
    )
);

-- Find documents in that specific tree that are ready for processing
SELECT * FROM document
WHERE
    id IN $tree
    AND status = 'PENDING'
    -- Only return if every dependency it has is already 'DONE'
    AND count(->depends_on[WHERE out.status != 'DONE']) = 0;

```

---

### 3. Handling the "Work Schedule" in Rust

In a real-world scenario, you might want to pre-calculate the groups to see the "Plan" before executing. Here is how you can represent those layers in Rust:

```rust
use rayon::prelude::*;

// A 'Layer' is just a group of documents that can run at the same time
type WorkLayer = Vec<Document>;
type WorkSchedule = Vec<WorkLayer>;

async fn plan_work_schedule(target_doc: &str, db: &Surreal<Client>) -> WorkSchedule {
    let mut schedule = Vec::new();

    // Note: This 'planning' loop would need to simulate the 'DONE' status
    // or use a temporary table/variable to avoid mutating real data
    // while just trying to preview the schedule.

    loop {
        let current_layer = fetch_ready_nodes(target_doc, db).await;
        if current_layer.is_empty() { break; }

        schedule.push(current_layer);
        // Mark as 'SIMULATED_DONE' for the next loop iteration...
    }

    schedule
}

```

### 4. Important Considerations for Rust + Rayon

* **The "Join" Point:** When using `batch.par_iter()`, Rayon creates a synchronization point. No document in "Layer 2" will start until every document in "Layer 1" is finished. This is exactly what you want for dependency safety.
* **Deadlocks (Circular Dependencies):** If Document A depends on B, and B depends on A, the query `count(...) = 0` will never be true for either. Your Rust code should check if `batch.is_empty()` while there are still `PENDING` records in the tree—if that happens, you have a circular dependency error.
* **Granularity:** If your documents render very quickly (microseconds), the overhead of querying the database for every layer might be high. In that case, you'd fetch the whole graph into a local Rust `petgraph` structure. But if rendering takes milliseconds or seconds, letting SurrealDB handle the logic is much cleaner.

## Validating No Circular Dependencies Exist

To ensure your system never hangs, you can implement a check to detect circular dependencies before you start the render. In a Directed Acyclic Graph (DAG), a circle is a "poison pill" that prevents a task runner from ever reaching a `count() = 0` state.

### 1. The Circular Dependency Check (SurrealQL)

You can use a "Path" check. If any document can reach **itself** by following the `depends_on` chain, you have a loop.

```surrealql
-- Check if any document in the target tree is part of a cycle
LET $is_cyclic = (
    SELECT id FROM (
        -- Traverse recursively and return the path taken
        SELECT id, <->depends_on<->document AS path FROM type::record($target_doc)
    )
    -- Check if the ID appears more than once in its own ancestry path
    WHERE id IN path
);

-- If $is_cyclic is not empty, throw an error or handle accordingly

```

---

### 2. Full Logic Flow: `get_work_schedule`

Here is how you would structure the final logic in your Rust application to bridge SurrealDB and `rayon`.

#### The Workflow:

1. **Identify the Scope:** Query the database for the full tree of dependencies for `doc`.
2. **Validate:** Run the cycle check above.
3. **Group into Generations:**
* Find nodes where `count(depends_on) == 0`. These are **Generation 0**.
* Remove Gen 0 from the mental "pending" list.
* Find nodes that now have 0 dependencies remaining. These are **Generation 1**.
* Repeat until all nodes are assigned a generation.



---

### 3. Implementation in Rust

Since you are using `rayon`, you want to return a `Vec<Vec<Document>>` where each inner `Vec` is a generation to be passed to `par_iter()`.

```rust
pub async fn get_work_schedule(target_doc: &str, db: &Surreal<Client>) -> Result<Vec<Vec<Document>>, Error> {
    let mut schedule = Vec::new();
    let mut processed_ids = HashSet::new();

    // 1. Fetch the entire flattened tree for the target_doc
    let mut all_docs: Vec<Document> = db.query(
        "SELECT * FROM (SELECT ->depends_on->document AS deps FROM type::record($target))"
    ).bind(("target", target_doc)).await?.take(0)?;

    // 2. Iteratively peel off layers (Kahn's approach)
    while !all_docs.is_empty() {
        // Find docs whose dependencies are all in 'processed_ids'
        let (generation, remaining): (Vec<Document>, Vec<Document>) = all_docs
            .into_iter()
            .partition(|doc| {
                // In SurrealDB, you'd check if doc.depends_on IDs are all in processed_ids
                doc.dependencies.iter().all(|dep_id| processed_ids.contains(dep_id))
            });

        if generation.is_empty() {
            return Err(Error::CircularDependency);
        }

        for doc in &generation {
            processed_ids.insert(doc.id.clone());
        }

        schedule.push(generation);
        all_docs = remaining;
    }

    Ok(schedule)
}

```

---

### 4. Executing with Rayon

Once you have your schedule, executing it in parallel is straightforward:

```rust
let schedule = get_work_schedule("doc:123", &db).await?;

for layer in schedule {
    // Rayon processes this generation in parallel
    layer.par_iter().for_each(|doc| {
        render_document(doc);
    });
    // The loop waits here (implicit join) before starting the next layer
}

```

### Summary of Benefits

* **Safety:** The cycle check prevents infinite loops.
* **Efficiency:** `rayon` maximizes CPU usage for each "layer."
* **Consistency:** By letting the database define the "Tree," you ensure that the Rust code stays thin and focused on execution rather than complex graph math.

## Caching Results Back to Database

To implement an incremental build system (similar to `make` or `turbo`), you need to track "State." You only want to render a document if **it** has changed or if **any of its dependencies** have changed.

In SurrealDB, we can handle this by storing a hash of the content and a timestamp.

### 1. Enhanced Schema for Caching

We add `content_hash` to detect changes in the file itself and `last_rendered_hash` to store the state of the tree at the time of the last successful build.

```surrealql
DEFINE TABLE document SCHEMAFULL;
DEFINE FIELD name ON TABLE document TYPE string;
DEFINE FIELD content_hash ON TABLE document TYPE string; -- Current hash of file
DEFINE FIELD last_rendered_hash ON TABLE document TYPE string; -- Hash at last render
DEFINE FIELD status ON TABLE document TYPE string DEFAULT 'PENDING';

-- Edge remains the same
DEFINE TABLE depends_on SCHEMAFULL;

```

---

### 2. The "Dirty" Check Query

A document is "dirty" (needs rendering) if:

1. Its current `content_hash` != `last_rendered_hash`.
2. **OR** any of its direct dependencies are currently marked as `DIRTY` or have been updated more recently than this document.

You can identify the work that *actually* needs to be done using this query:

```surrealql
-- Find nodes that need to be re-rendered
UPDATE document SET status = 'DIRTY'
WHERE
    content_hash != last_rendered_hash
    OR
    -- Check if any parent (dependency) is dirty
    count(->depends_on->document[WHERE status = 'DIRTY']) > 0;

```

---

### 3. Rust Integration: Hashing and Updating

In your Rust code, you will generate the `content_hash` (e.g., using the `sha2` crate) before starting the schedule.

**IMPORTANT:** we will be using **xxHash** not **sha2** as it's much faster and being a cryptographic has holds not benefits!

```rust
use sha2::{Sha256, Digest};

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

// Inside your execution loop...
batch.par_iter().for_each(|doc| {
    // 1. Perform the heavy lifting
    let result = render_document(doc);

    // 2. On success, prepare the update
    // We set last_rendered_hash = current content_hash
    // and reset status to 'DONE'
});

```

---

### 4. Efficient Multi-User/Multi-Project Support

If you have multiple people rendering different documents, you don't want them stepping on each other's `status` fields. Instead of a simple string like `'PENDING'`, use a **Session ID** or a **Job ID**.

```surrealql
-- Instead of a global status, use a relation or a specific field
UPDATE document SET render_jobs += {
    job_id: $current_job,
    status: 'COMPLETE',
    timestamp: time::now()
};

```

### The Final Workflow

1. **Scan:** Rust scans files, calculates hashes, and updates `content_hash` in SurrealDB.
2. **Mark Dirty:** Run the "Dirty Check" query to propagate changes down the tree.
3. **Schedule:** Use the "Next Batch" query from our previous step, but filter only for `status = 'DIRTY'`.
4. **Execute:** `rayon` renders the batches.
5. **Commit:** Update `last_rendered_hash` so the next run skips these files.

### Next Step

This system is now quite robust. Would you like to see how to implement **Error Propagation**? (i.e., if a leaf node fails to render, how to automatically cancel all documents that depend on it?)
