# Topological Sort and Ways to Work with Surreal DB

> This document represents a discussion I was having a Gemini chatbot about how to [schedule concurrent work](./work-scheduling.md) while working effectively with **SurrealDB**. Each H2 heading represents a new part of the conversation.



## Intro

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


## Working with a broader view of our Requirements

This broader picture shifts your database from a simple task list into a **Global Dependency Graph**. You are dealing with four distinct "Resource Types," and your schema needs to treat them as such while maintaining a unified interface for the scheduler.

### 1. The Polymorphic Schema

To handle local vs. remote and document vs. image, we can use a single `resource` table with different fields, or separate tables. For SurrealDB, using a single table with a `type` field (or separate tables and a `record` link) works best.

```surrealql
-- The core Resource table
DEFINE TABLE resource SCHEMAFULL;
DEFINE FIELD type ON TABLE resource TYPE string; -- 'document', 'image'
DEFINE FIELD source ON TABLE resource TYPE string; -- 'local', 'remote'
DEFINE FIELD url ON TABLE resource TYPE string ASSERT is::url($value) OR $value = NONE;
DEFINE FIELD path ON TABLE resource TYPE string; -- local filesystem path
DEFINE FIELD content_hash ON TABLE resource TYPE string;
DEFINE FIELD last_checked ON TABLE resource TYPE datetime DEFAULT time::now();
DEFINE FIELD expiry_days ON TABLE resource TYPE int DEFAULT 1;

-- Optimized variations for images
DEFINE TABLE optimized_image SCHEMAFULL;
DEFINE FIELD parent_image ON TABLE optimized_image TYPE record<resource>;
DEFINE FIELD width ON TABLE optimized_image TYPE int;
DEFINE FIELD format ON TABLE optimized_image TYPE string; -- 'webp', 'avif'

```

### 2. The Atomic "Dirty" Transaction

You want to run a transaction that decides what needs to be worked on. This logic implements your **Time-Based Expiry** for remote resources and **Hash Variance** for local ones.

```surrealql
BEGIN TRANSACTION;

-- 1. Mark Remote resources as DIRTY if their "lease" has expired
UPDATE resource SET status = 'DIRTY'
WHERE source = 'remote'
  AND last_checked + <duration> (expiry_days + "d") < time::now();

-- 2. Mark Local resources as DIRTY if hashes don't match
-- (This assumes your Rust code updated 'content_hash' before this transaction)
UPDATE resource SET status = 'DIRTY'
WHERE source = 'local'
  AND content_hash != last_rendered_hash;

-- 3. PROPAGATION: If a dependency is DIRTY, the parent is DIRTY
-- We run this recursively to ensure the 'dirty' state flows up the tree
UPDATE resource SET status = 'DIRTY'
WHERE status = 'PENDING'
  AND count(->depends_on->resource[WHERE status = 'DIRTY']) > 0;

COMMIT TRANSACTION;

```

---

### 3. Integrating Images & Optimization

For your image optimization, the "Leaf Nodes" are the specific widths/formats you need. In your Rust `get_work_schedule(doc)`, the schedule will look like this:

1. **Layer 0:** Fetch Remote Images (HTTP) + Fetch Remote Docs.
2. **Layer 1:** Run `xxHash` on the fetched files.
3. **Layer 2:** Generate optimized image variants (Parallelized with `rayon`).
4. **Layer 3:** Render the Document using the local paths of the optimized images.

---

### 4. Rust Logic: Handling Foreign Resources

Since you are using `rayon`, you can handle the HTTP fetches and image processing in parallel.

```rust
// Logic for get_work_schedule with Resource Types
match resource.source {
    Source::Remote => {
        if needs_refresh(resource) {
            let data = reqwest::get(resource.url).await?;
            let new_hash = xxhash_rust::xxh3::xxh3_64(&data);
            // Update SurrealDB with new hash and set status
        }
    },
    Source::Local => {
        // Just check xxHash of local file
    }
}

// Image Optimization Layer
if resource.type == ResourceType::Image {
    let variants = vec![640, 1280, 1920];
    variants.par_iter().for_each(|width| {
        optimize_image(&resource.path, *width);
    });
}

```

### 5. Why this works for your goals

* **Caching:** By storing the `content_hash` and `last_rendered_hash` in SurrealDB, images are only optimized once. Even if you restart the app, SurrealDB remembers the hashes.
* **Expiry:** The transaction handles the 1-day logic automatically using SurrealDB's `datetime` math.
* **Concurrency:** `rayon` doesn't care if it's resizing an image or rendering a document; it just sees a `Vec<Task>` in the current "Generation."


## A SurrealQL Function for `last_checked`


To automate the expiry logic and keep your Rust code clean, we can move the "revalidation" logic directly into SurrealDB using a custom function. This ensures that the logic for "what is stale" is centralized in the database.

### 1. The Stale-Check Function

This function evaluates a resource and returns a boolean. It encapsulates your "1-day expiry" rule and can be easily adjusted later without recompiling your Rust binary.

```surrealql
DEFINE FUNCTION fn::is_resource_stale($res: record<resource>) {
    -- If it's a remote resource, check the time
    IF $res.source = 'remote' {
        RETURN $res.last_checked + <duration> ($res.expiry_days + "d") < time::now();
    };

    -- If it's local, it's only stale if the hash changed
    -- (This check is usually done via Rust updating the content_hash first)
    RETURN $res.content_hash != $res.last_rendered_hash;
};

```

---

### 2. The Atomic "Preparation" Transaction

Before you call `get_work_schedule` in Rust, you run this transaction. It prepares the "Dirty" list by looking at the specific tree of your target document.

```surrealql
BEGIN TRANSACTION;

-- 1. Identify all resources in the target tree
LET $tree = (SELECT VALUE out FROM (SELECT ->depends_on->resource AS out FROM type::record($target_doc)));

-- 2. Mark specifically stale remote resources as DIRTY
UPDATE resource SET status = 'DIRTY'
WHERE id IN $tree
  AND fn::is_resource_stale(self);

-- 3. Mark parents of DIRTY items as DIRTY (The Ripple Effect)
-- We use a loop to ensure propagation through deep trees
FOR $i IN 1..10 {
    UPDATE resource SET status = 'DIRTY'
    WHERE id IN $tree
      AND status = 'PENDING'
      AND count(->depends_on->resource[WHERE status = 'DIRTY']) > 0;
};

COMMIT TRANSACTION;

```

---

### 3. The Multi-Type Rust Workflow

With this setup, your Rust logic for handling different types (Images, Remote Docs, etc.) becomes much more organized. You can use an `enum` to dispatch work within the `rayon` parallel iterator.

```rust
#[derive(Deserialize)]
enum ResourceType { Document, Image }

#[derive(Deserialize)]
enum Source { Local, Remote }

// This runs inside a layer.par_iter()
fn process_resource(res: &Resource) {
    match (res.r_type, res.source) {
        (ResourceType::Image, Source::Remote) => {
            // 1. Download
            // 2. Hash
            // 3. Optimize (spawn sub-tasks if needed)
        }
        (ResourceType::Image, Source::Local) => {
            // 1. Hash local file
            // 2. If different, re-optimize
        }
        (ResourceType::Document, Source::Remote) => {
            // 1. Fetch via HTTP
            // 2. Parse/Render
        }
        _ => { /* etc */ }
    }
}

```

---

### 4. Why this handles your constraints:

* **Remote Image Optimization:** If a remote image is updated, the `fn::is_resource_stale` triggers. The image is re-downloaded, its new hash is stored, and because its status is `DIRTY`, all its "Optimized Leaf Nodes" (the resized versions) are automatically flagged for re-generation by the ripple effect in the transaction.
* **Local Image Caching:** If a local image hasn't changed, the hash check returns false, the status remains `PENDING`, and `rayon` skips it entirely.
* **Time-based Expiry:** The `last_checked` field combined with `expiry_days` ensures that HTTP requests are only made when the "lease" is up, saving bandwidth and preventing rate-limiting on foreign documents.

## Cleaning up Orphaned Images

To clean up "orphaned" optimized images in SurrealDB, you should leverage **Table Events**.

In a graph database, when a "parent" node (the original image) is deleted, the edges (`depends_on`) usually disappear, but the "child" nodes (the optimized versions) might linger as garbage. SurrealDB Events act like triggers to ensure that when a resource is removed, its derivatives are purged automatically.

### 1. The Cleanup Event

We define an event on the `resource` table. When a record is deleted, we look for any `optimized_image` records that point to it and delete them as well.

```surrealql
DEFINE EVENT cleanup_optimized_variants ON TABLE resource WHEN $event = "DELETE" THEN {
    -- Delete any optimized versions that were linked to this specific resource
    DELETE optimized_image WHERE parent_image = $before.id;

    -- Also delete any outgoing dependency edges to keep the graph clean
    DELETE depends_on WHERE in = $before.id OR out = $before.id;
};

```

---

### 2. Implementation: The "Refined" Render Cycle

Now that we have the schema, the expiry logic, and the cleanup events, here is how the complete `get_work_schedule` cycle looks from the perspective of your Rust application using `rayon`.

#### Step 1: Pre-Process (Local Hashes)

Before querying SurrealDB, your Rust code should quickly scan the local `path` for all local resources and compute their **xxHash**.

* Update the `content_hash` in the database.
* This makes the "Dirty Check" transaction accurate.

#### Step 2: The "Preparation" Transaction

Run the transaction we built previously. This marks the "stale" remote items and "dirty" local items.

#### Step 3: Fetch and Execute Layers

You call your `get_work_schedule` function. It returns layers. Because of your specific requirements, a layer might look like this:

**Layer 1 (The truly independent leaves):**

* Remote Images (need download)
* Remote Documents (need fetch)
* Local Images (need xxHash check)

**Layer 2 (The dependents):**

* Image Optimization (starts only after the parent image is downloaded and hashed)
* Sub-documents (that depend on the remote docs from Layer 1)

**Layer 3:**

* The final Document Render (merging the optimized images and text)

---

### 3. Final Rust Structure with Error Propagation

Using `rayon`, you can handle the different resource types within a single parallel loop.

```rust
for layer in schedule {
    layer.par_iter().for_each(|res| {
        let result = match (res.r_type, res.source) {
            (Image, Remote) => handle_remote_image(res, &db),
            (Image, Local)  => handle_local_image(res, &db),
            (Doc, Remote)   => handle_remote_doc(res, &db),
            (Doc, Local)    => handle_local_doc(res, &db),
        };

        if let Err(e) = result {
            // "Poison" the branch in SurrealDB so dependents are skipped
            let _ = db.query("UPDATE resource SET status = 'FAILED' WHERE id = $id")
                .bind(("id", &res.id)).at_once();
        }
    });
}

```

### 4. Summary of the Architecture

1. **SurrealDB** acts as the **State Machine** and **Graph Engine**. It knows what is stale, what is dirty, and what depends on what.
2. **xxHash** provides the **Identity**. It tells the database if a local file has actually changed.
3. **Rayon** provides the **Compute**. It ensures your CPU cores are fully utilized while respecting the dependency boundaries set by the database.
4. **Events** provide the **Sanitation**. They ensure that your database doesn't fill up with "zombie" optimized images when you delete a source file.

## "Shadowing" for Parallelism

To get the most out of `rayon`, you should treat every specific optimization task (e.g., "Resizing Image A to 300px WebP") as a distinct node in the graph. This allows your scheduler to treat image processing as just another set of leaf nodes that can be distributed across all available CPU cores.

### 1. The "Derivative" Schema

We will link optimized variants to their parent resource using a specific relationship. This allows the graph to "fan out" from a single source image into multiple parallel tasks.

```surrealql
-- The relationship from a Source Image to its target Optimized Variant
DEFINE TABLE has_variant SCHEMAFULL;
DEFINE FIELD in ON TABLE has_variant TYPE record<resource>;  -- The Source Image
DEFINE FIELD out ON TABLE has_variant TYPE record<resource>; -- The Resulting Variant

-- Example: Creating the 'Target' nodes for an image
LET $source = resource:image_1;

-- Create individual task nodes for different sizes
LET $small = (CREATE resource SET type = 'image_variant', width = 300, format = 'webp', status = 'PENDING');
LET $large = (CREATE resource SET type = 'image_variant', width = 1200, format = 'webp', status = 'PENDING');

-- Link them so the scheduler knows they depend on the source
RELATE $source->has_variant->$small;
RELATE $source->has_variant->$large;

```

---

### 2. The Integrated "Work Schedule" Query

Now, your `get_work_schedule` doesn't need to know the difference between a "Document" and an "Image Resize Task." It just sees nodes that are ready to go.

A node is ready if:

1. It is `PENDING` or `DIRTY`.
2. All its incoming `depends_on` OR `has_variant` edges point to nodes that are `DONE`.

```surrealql
SELECT * FROM resource
WHERE (status = 'PENDING' OR status = 'DIRTY')
  AND count(<-depends_on[WHERE out.status != 'DONE']) = 0
  AND count(<-has_variant[WHERE out.status != 'DONE']) = 0;

```

---

### 3. Handling the Work in Rust

When `rayon` picks up an `image_variant` node, the logic is highly specific. Since the variant knows its parent, it can pull the source data, perform the resize, and save the result.

```rust
fn handle_task(res: &Resource, db: &Surreal<Client>) {
    match res.r_type {
        "image_variant" => {
            // 1. Get the parent image path (the 'in' side of the relation)
            let parent: Resource = db.query("SELECT in FROM has_variant WHERE out = $id")
                .bind(("id", &res.id)).take(0)?;

            // 2. Perform the XXHash/Resize using the parent's data
            let bytes = std::fs::read(&parent.path)?;
            let optimized = image::load_from_memory(&bytes)?
                .resize(res.width, res.height, FilterType::Lanczos3);

            // 3. Save and mark DONE
            optimized.save(&res.path)?;
            db.query("UPDATE $id SET status = 'DONE', last_rendered_hash = $h")
                .bind(("id", &res.id)).bind(("h", parent.content_hash));
        },
        "document" => {
            // Render document logic...
        }
    }
}

```

---

### 4. Visualizing the Full Parallel Pipeline

By structuring your data this way, you create a "Waterfall" effect that `rayon` can drink from:

1. **Stage 1 (I/O Bound):** Download remote images and docs. (Parallelized)
2. **Stage 2 (CPU Bound):** Compute `xxHash` for all new data. (Parallelized)
3. **Stage 3 (Heavy CPU):** The "Fan-out." One image might spark 5 different resize tasks. If you have 10 images, `rayon` now has 50 tasks to spread across your cores.
4. **Stage 4 (Composite):** Documents render only once all their specific image variants are `DONE`.

### 5. Final Architecture Benefits

* **Granularity:** You aren't waiting for one image to finish *all* its sizes before starting the next image. As soon as the source is ready, every size starts in parallel.
* **Safety:** The `has_variant` relationship acts as a lock. A 300px WebP won't try to generate until the source download is `DONE`.
* **Cleanliness:** If you delete the source `resource`, your `DELETE EVENT` from the previous step will automatically wipe out all the variant nodes and their files.


