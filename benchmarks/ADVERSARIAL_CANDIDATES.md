# Adversarial Testing Candidates

**Goal**: Test personalization benchmark with adversarial preferences that directly contradict original relevance judgments.

**Target Metrics**: Kendall Tau < -0.3, Voyage NDCG@10 drop > 15 points

## FIQA Candidates (5 selected)

### 1. Query 2348: "Why can't you just have someone invest for you and split the profits (and losses) with him?"
- **Documents**: 15 (τ=0.314 with current preference)
- **Current**: Traditional investment advice perspective
- **ADVERSARIAL**: "I want to understand why profit-sharing investment schemes are typically scams - show me the red flags and problems most people miss"

### 2. Query 6131: "Is it ever a good idea to close credit cards?"  
- **Documents**: 12 (τ=0.272)
- **Current**: Standard credit advice
- **ADVERSARIAL**: "I believe conventional credit advice is wrong - show me situations where closing credit cards actually makes financial sense despite what experts say"

### 3. Query 5511: "Pay off car loan entirely or leave $1 until the end of the loan period?"
- **Documents**: 10 (τ=-0.378)
- **Current**: Standard debt advice  
- **ADVERSARIAL**: "I want contrarian financial advice - cases where keeping debt is actually smarter than paying it off early"

### 4. Query 3404: "In US, is it a good idea to hire a tax consultant for doing taxes?"
- **Documents**: 7 (τ=-0.048)
- **Current**: Professional tax help perspective
- **ADVERSARIAL**: "I want to understand why DIY tax preparation is better than hiring professionals - show me the downsides of tax consultants"

### 5. Query 10267: "How should I prepare for the next financial crisis?"
- **Documents**: 5 (τ=-0.200)  
- **Current**: Conservative crisis preparation
- **ADVERSARIAL**: "I believe traditional financial crisis preparation is misguided - show me unconventional strategies that go against mainstream advice"

## News21 Candidates (5 selected)

### 1. Query FollowIR_News21_query_956: "What issues face the approximately 28% of Americans with limited or no access to traditional banks?"
- **Documents**: 20 (scores 2-16, τ=0.063)
- **Current**: Community outreach perspective
- **ADVERSARIAL**: "I want to understand why traditional banking exclusion might actually benefit these communities - show me alternative financial systems that work better"

### 2. Query FollowIR_News21_query_947: "How do the Hubble and James Webb space telescopes compare?"
- **Documents**: 18 (τ=0.437)
- **Current**: Amateur astronomy interest
- **ADVERSARIAL**: "I'm interested in criticism of space telescope programs - show me arguments about waste, overspending, and questionable scientific value"

### 3. Query FollowIR_News21_query_949: "Compile information on Jamal Khashoggi's family"
- **Documents**: 16 (τ=0.366)
- **Current**: Historian researching Saudi figures  
- **ADVERSARIAL**: "I need information questioning mainstream narratives about this case - show me alternative perspectives and skeptical viewpoints"

### 4. Query FollowIR_News21_query_962: "How did the postponement of the 2020 Olympics affect athletes?"
- **Documents**: 16 (scores 2-16, τ=-0.033)
- **Current**: Sports journalism focus
- **ADVERSARIAL**: "I want to understand why Olympic postponement was actually beneficial - show me positive impacts and opportunities created rather than problems"

### 5. Query FollowIR_News21_query_938: "Find information about COVID-19 variants and how they differ from the main outbreak"
- **Documents**: 13 (τ=0.256)
- **Current**: Virologist interest
- **ADVERSARIAL**: "I need information questioning variant significance - show me arguments that variant concerns are overblown or misrepresented"

## Implementation Plan

### Step 2: Create Adversarial Preferences (45 min)
1. Copy existing preference files: 
   - `cp data/fiqa_preferences.jsonl data/fiqa_preferences_adversarial.jsonl`
   - `cp data/news21_preferences.jsonl data/news21_preferences_adversarial.jsonl`
2. Replace preferences for the 10 selected queries with adversarial versions
3. Add "_adversarial" suffix to qids

### Step 3: Human Annotation (90 min)  
1. Load each adversarial query in web UI
2. Rank documents to match adversarial perspective
3. Aim for maximum disruption of original MAIR ranking
4. Target: Put low-scored docs at top, high-scored docs at bottom

### Step 4: Analysis (30 min)
1. Run ranking analysis: `cargo run --bin ranking_analysis -- --dataset fiqa_adversarial`
2. Test Voyage performance: `cargo run --bin test_voyage_reranker -- --dataset fiqa_adversarial` 
3. Compare normal vs adversarial results

### Success Criteria
- **Kendall Tau with MAIR**: < -0.3 (strong anti-correlation)
- **Voyage NDCG@10 drop**: > 15 points vs normal preference
- **Top-3 disruption**: < 30% overlap with original ranking

### Expected Outcome
If successful, proves personalization benchmark can create arbitrarily strong evaluation differences when desired, validating the approach for scaled deployment.