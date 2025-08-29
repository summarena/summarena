# PAIR Reranking Test - Personalized vs Non-Personalized

Test Voyage reranker with personalized preferences against MAIR benchmark datasets to compare baseline vs personalized reranking performance.

## Setup

1. **Set API Key:**
   ```bash
   export VOYAGE_API_KEY="your_voyage_api_key"
   export OPENAI_API_KEY="your_openai_key"  # Optional: for LLM-generated personas
   ```

2. **Build:**
   ```bash
   cargo build --release
   ```

## Usage

### Basic Usage
```bash
# Test 10 random queries from acordar dataset (default)
cargo run --bin test_reranking

# Test single query by index
cargo run --bin test_reranking --single 51

# Test 5 random queries from specific dataset
cargo run --bin test_reranking --dataset fiqa --count 5
```

### Dataset Options
```bash
# List popular datasets
cargo run --bin test_reranking --list-datasets

# Discover all 120+ available datasets (requires internet)
cargo run --bin test_reranking --discover-all

# Use specific dataset
cargo run --bin test_reranking --dataset fiqa
```

### Personalization Options
```bash
# Test with custom persona
cargo run --bin test_reranking --persona "Medical researcher focused on clinical trials"

# Generate personas using LLM (requires OPENAI_API_KEY)
cargo run --bin test_reranking --generate-personas

# Adjust number of first-stage candidates for reranking
cargo run --bin test_reranking --candidate-limit 50
```

## Available Datasets

**Working datasets** (with first-stage results): `acordar`, `fiqa`

**Listed datasets** (may need first-stage results): `climate-fever`, `fever`, `news21`, `nfcorpus`, `quora`, `scidocs`

## Output

The program shows clean, results-only output with:
- **Per-query breakdown**: Each query with baseline vs personalized NDCG@10 scores
- **Improvement deltas**: Difference between personalized and baseline performance  
- **Summary statistics**: Average baseline, average improvement, and success rate percentage

### Sample Output
```
FINAL COMPARISON RESULTS
============================================================

Query 1: What margin is required to initiate and maintain a short sal...
Baseline NDCG@10: 0.5000
  Persona: Academic researcher; values peer-reviewed, methodo... → NDCG@10: 0.6309 (Δ0.1309)
  Persona: Industry professional; prefers practical, implemen... → NDCG@10: 0.6309 (Δ0.1309)
  Persona: Policy maker; needs comprehensive, policy-relevant... → NDCG@10: 0.6309 (Δ0.1309)

============================================================
SUMMARY
Average baseline NDCG@10: 0.2500
Average improvement: 0.0655
Positive improvements: 3/6 (50.0%)
```

## Examples

Test a single query with custom persona:
```bash
cargo run --bin test_reranking --single 0 --persona "Academic researcher valuing peer-reviewed sources"
```

Generate AI personas for queries:
```bash
cargo run --bin test_reranking --dataset fiqa --generate-personas --count 3
```

Run large-scale test:
```bash
cargo run --bin test_reranking --count 50 --dataset fiqa
```

## Notes

- All verbose output (API responses, loading messages, debug info) has been suppressed
- Only final comparison results and summary statistics are displayed
- Perfect for large-scale testing where you only need the key performance metrics
- The `--quiet` flag was removed since the output is now always quiet/clean