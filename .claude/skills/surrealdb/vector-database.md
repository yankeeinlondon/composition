# SurrealDB as Vector Database

SurrealDB provides native vector operations for semantic search, recommendations, and RAG applications. Store, index, and query vectors alongside structured data without separate infrastructure.

## Setup

### Define Vector Schema

```sql
DEFINE TABLE document SCHEMAFULL;
DEFINE FIELD title ON document TYPE string;
DEFINE FIELD content ON document TYPE string;
DEFINE FIELD embedding ON document TYPE array<float>;

-- HNSW index for approximate nearest neighbor search
-- DIMENSION must match your embedding model output size
DEFINE INDEX doc_embedding ON document FIELDS embedding
    HNSW DIMENSION 1536 DISTANCE COSINE;
```

### Distance Metrics

| Metric | Use Case |
|--------|----------|
| `COSINE` | Text embeddings (most common) |
| `EUCLIDEAN` | Spatial/geometric data |
| `MANHATTAN` | Grid-based distances |

## Storing Embeddings

### Rust

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Document {
    title: String,
    content: String,
    embedding: Vec<f32>,
}

// Generate embedding from your model (OpenAI, Mistral, etc.)
let embedding = embedding_model.embed(&content).await?;

let doc: Option<Document> = db.create("document").content(Document {
    title: "Introduction to Vectors".into(),
    content: content.clone(),
    embedding,
}).await?;
```

### TypeScript

```typescript
interface Document {
  title: string;
  content: string;
  embedding: number[];
}

const embedding = await embeddingModel.embed(content);

await db.create<Document>("document", {
  title: "Introduction to Vectors",
  content,
  embedding
});
```

## Vector Similarity Search

### Basic Search

```sql
-- Find k=5 nearest neighbors using COSINE distance
SELECT
    id, title, content,
    vector::distance::cosine(embedding, $query) AS distance
FROM document
WHERE embedding <|5, COSINE|> $query
ORDER BY distance
LIMIT 10;
```

### Rust Example

```rust
#[derive(Debug, Deserialize)]
struct SearchResult {
    id: RecordId,
    title: String,
    distance: f32,
}

let query_embedding = embedding_model.embed(&query_text).await?;

let mut results = db.query(r#"
    SELECT id, title, vector::distance::cosine(embedding, $embedding) AS distance
    FROM document
    WHERE embedding <|5, COSINE|> $embedding
    ORDER BY distance LIMIT 10
"#)
.bind(("embedding", query_embedding))
.await?;

let docs: Vec<SearchResult> = results.take(0)?;
```

### TypeScript Example

```typescript
const queryEmbedding = await embeddingModel.embed(queryText);

const [results] = await db.query<SearchResult[]>(`
  SELECT id, title, vector::distance::cosine(embedding, $embedding) AS distance
  FROM document
  WHERE embedding <|5, COSINE|> $embedding
  ORDER BY distance LIMIT 10
`, { embedding: queryEmbedding });
```

## Hybrid Search

Combine vector similarity with metadata filtering:

```sql
-- Vector + metadata filter
SELECT
    id, title, content,
    vector::distance::cosine(embedding, $embedding) AS distance
FROM document
WHERE
    embedding <|5, COSINE|> $embedding
    AND category = $category
    AND created_at > $date
ORDER BY distance
LIMIT 10;

-- Vector + full-text search
SELECT
    id, title,
    vector::distance::cosine(embedding, $embedding) AS vec_score,
    search::score(1) AS text_score
FROM document
WHERE
    embedding <|10, COSINE|> $embedding
    AND content @1@ $search_terms
ORDER BY (vec_score * 0.7 + text_score * 0.3)
LIMIT 10;
```

## RAG Application Pattern

```rust
// 1. Generate embedding for user query
let query_embedding = embedding_model.embed(&user_question).await?;

// 2. Retrieve relevant documents
let mut results = db.query(r#"
    SELECT content FROM document
    WHERE embedding <|5, COSINE|> $embedding
    ORDER BY vector::distance::cosine(embedding, $embedding)
    LIMIT 5
"#).bind(("embedding", query_embedding)).await?;

let contexts: Vec<String> = results.take(0)?;

// 3. Generate response with context
let prompt = format!(
    "Context:\n{}\n\nQuestion: {}\n\nAnswer:",
    contexts.join("\n---\n"),
    user_question
);

let response = llm.generate(&prompt).await?;
```

## Optimization

### HNSW Parameters

```sql
-- Custom HNSW parameters for better recall
DEFINE INDEX optimized_idx ON document FIELDS embedding
    HNSW DIMENSION 1536 DISTANCE COSINE
    M 16 EF_CONSTRUCTION 200;
```

| Parameter | Effect |
|-----------|--------|
| `M` | Connections per node (higher = better recall, more memory) |
| `EF_CONSTRUCTION` | Build-time search depth (higher = better index, slower build) |

### Batch Processing

```rust
const BATCH_SIZE: usize = 100;

for chunk in documents.chunks(BATCH_SIZE) {
    let mut query = db.query("BEGIN TRANSACTION");

    for doc in chunk {
        query = query.query("CREATE document CONTENT $doc").bind(("doc", doc));
    }

    query.query("COMMIT TRANSACTION").await?;
}
```

## Common Gotchas

### Dimension Mismatch

```rust
// WRONG - index dimension doesn't match embedding
DEFINE INDEX idx ON doc FIELDS embedding HNSW DIMENSION 768 DISTANCE COSINE;
// But embeddings have 1536 dimensions -> Error!

// CORRECT - validate before storing
const MAX_DIMENSIONS: usize = 16384;

fn validate_embedding(embedding: &[f32], expected: usize) -> Result<(), String> {
    if embedding.len() != expected {
        return Err(format!("Expected {} dimensions, got {}", expected, embedding.len()));
    }
    if embedding.len() > MAX_DIMENSIONS {
        return Err(format!("Exceeds max {} dimensions", MAX_DIMENSIONS));
    }
    Ok(())
}
```

### Missing Distance Metric

```sql
-- WRONG - no distance metric
SELECT * FROM document WHERE embedding <|5|> $embedding;

-- CORRECT - specify distance metric
SELECT * FROM document WHERE embedding <|5, COSINE|> $embedding;
```

### Index Not Used

Use `EXPLAIN` to verify index usage:

```sql
EXPLAIN SELECT * FROM document WHERE embedding <|5, COSINE|> $embedding;
-- Check for "Iterate Index" operation
```

### Memory with Large Results

```rust
// Process large result sets in batches
async fn process_large_results(db: &Surreal<Any>, query_embedding: Vec<f32>) {
    let batch_size = 1000;
    let mut offset = 0;

    loop {
        let mut results = db.query(r#"
            SELECT * FROM document
            WHERE embedding <|100, COSINE|> $embedding
            ORDER BY vector::distance::cosine(embedding, $embedding)
            LIMIT $limit START $offset
        "#)
        .bind(("embedding", &query_embedding))
        .bind(("limit", batch_size))
        .bind(("offset", offset))
        .await?;

        let batch: Vec<Document> = results.take(0)?;
        if batch.is_empty() { break; }

        process_batch(batch);
        offset += batch_size;
    }
}
```

## Embedding Model Tips

| Model | Dimensions | Best For |
|-------|------------|----------|
| text-embedding-3-small | 1536 | General text, cost-effective |
| text-embedding-3-large | 3072 | High accuracy needs |
| Mistral-embed | 1024 | Open-source, self-hosted |

Always normalize embeddings if your model doesn't do it automatically when using COSINE distance.

## Related

- [Graph Database](./graph-database.md) - Combine with graph traversals
- [Rust SDK](./sdk-rust.md)
- [TypeScript SDK](./sdk-typescript.md)
