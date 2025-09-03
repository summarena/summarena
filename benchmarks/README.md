# Document Ranking Benchmark with Personalization

This repository evaluates how personalized user preferences affect document ranking quality. It provides tools to collect human rankings via a web interface and compare them against both original dataset labels and modern rerankers like Voyage.

## What This Does

**Core Problem**: Traditional IR evaluation assumes universal relevance, but users have different preferences. This toolkit measures how much personalization changes rankings and whether rerankers can adapt to personalized preferences.

**Key Capabilities**:
1. **Generate personalized preferences** using LLMs for realistic user personas
2. **Collect human rankings** through an intuitive web interface
3. **Analyze ranking differences** between human/original labels statistically
4. **Benchmark rerankers** against personalized human judgments

## Repository Structure

```
benchmarks/
├── src/
│   ├── bin/
│   │   ├── ui.rs                    # Web ranking interface
│   │   ├── prepare_preferences.rs   # Generate user preferences
│   │   ├── ranking_analysis.rs      # Analyze ranking differences
│   │   └── test_voyage_reranker.rs  # Benchmark rerankers
│   ├── web_ui.rs                    # Web server logic
│   ├── llm.rs                       # LLM preference generation
│   ├── lib.rs                       # Data loading utilities
│   └── types.rs                     # Data structures
├── static/index.html                # Drag-and-drop ranking UI
├── data/                            # Dataset files (generated)
└── Cargo.toml                       # Dependencies
```

## How to Use This Repo

### Step 1: Setup Your Dataset
Place your dataset files in `data/`:
```bash
data/{dataset}_queries.jsonl    # Queries with relevance labels
data/{dataset}_docs.jsonl       # Document content
```

### Step 2: Generate Personalized Preferences
```bash
# Creates personalized user preferences for 100 random queries
cargo run --bin prepare_preferences -- --dataset fiqa
# Output: data/fiqa_preferences.jsonl
```

### Step 3: Collect Human Rankings
```bash
# Launch web interface for human annotation
cargo run --bin ui -- --dataset fiqa
# Visit http://127.0.0.1:3000 to rank documents
# Output: data/fiqa_human_rankings.jsonl
```

### Step 4: Analyze Results
```bash
# Compare human vs original rankings
cargo run --bin ranking_analysis -- --dataset fiqa
# Output: data/fiqa_ranking_analysis_results.json

# Test reranker performance
cargo run --bin test_voyage_reranker -- --dataset fiqa
# Output: Console metrics + data/fiqa_voyage_comparison.json
```

## Example Data Formats

### Input: Queries (`fiqa_queries.jsonl`)
```json
{
  "qid": "FiQA_query_123",
  "query": "How to start investing with limited income?",
  "labels": [
    {"id": "doc_abc", "score": 2},
    {"id": "doc_def", "score": 1},
    {"id": "doc_xyz", "score": 0}
  ]
}
```

### Input: Documents (`fiqa_docs.jsonl`)
```json
{
  "id": "doc_abc",
  "content": "For beginners with limited income, consider starting with index funds which offer diversification at low cost..."
}
```

### Generated: Preferences (`fiqa_preferences.jsonl`)
```json
{
  "qid": "FiQA_query_123",
  "query": "How to start investing with limited income?",
  "preference": "I am a recent college graduate with student loans looking for beginner-friendly investment advice",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Generated: Human Rankings (`fiqa_human_rankings.jsonl`)
```json
{
  "qid": "FiQA_query_123",
  "rankings": [
    {"doc_id": "doc_xyz", "rank": 1},
    {"doc_id": "doc_abc", "rank": 2},
    {"doc_id": "doc_def", "rank": 3}
  ],
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Analysis Output: Ranking Analysis (`fiqa_ranking_analysis_results.json`)
```json
{
  "qid": "FiQA_query_123",
  "query": "How to start investing with limited income?",
  "kendall_tau": 0.256,
  "average_rank_change": 2.1,
  "docs_moved_up": 2,
  "docs_moved_down": 3,
  "top3_overlap": 0.667
}
```

### Reranker Output: Example Console Results
```
=== Query FiQA_query_123 ===
Using preference: 'I am a recent college graduate with student loans...'

--- Voyage vs Human Rankings ---
NDCG@3: 0.841
NDCG@10: 0.892
Kendall Tau: 0.634

--- Voyage vs MAIR Original ---
NDCG@3: 0.734
NDCG@10: 0.798
Kendall Tau: 0.445
```

**Key Insight**: Voyage performs better on personalized human rankings (NDCG@10: 0.892) than original MAIR labels (0.798), showing personalization creates meaningful evaluation differences.
