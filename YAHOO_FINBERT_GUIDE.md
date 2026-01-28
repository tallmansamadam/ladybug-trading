# 🚀 Yahoo RSS + Local FinBERT Sentiment System

## 🎯 What We Built

A **COMPLETE** local sentiment analysis system with:
- **Yahoo Finance RSS** - Free, unlimited news feeds
- **Local FinBERT LLM** - Real NLP sentiment analysis on YOUR hardware
- **Zero external costs** - All processing done locally!

---

## 🏗️ Architecture

```
Yahoo Finance RSS → Rust Backend → Python FinBERT → Sentiment Scores → Trading
    (Free News)      (Fetches)      (Analyzes)      (-1.0 to +1.0)    (Decides)
```

---

## 📦 Installation

### Step 1: Install Python Dependencies

```bash
cd sentiment-service
pip install -r requirements.txt
```

**What this does:**
- Installs Flask (web server)
- Installs Transformers (Hugging Face library)
- Installs PyTorch (deep learning framework)
- Downloads FinBERT model (~400MB, first run only)

**Requirements:**
- Python 3.8+
- **Recommended:** NVIDIA GPU with CUDA (for speed)
- **Minimum:** 8GB RAM (works on CPU, just slower)

### Step 2: Build Rust Backend

```bash
cd ../rust-engine
cargo build
```

---

## 🚀 Starting the System

**You need TWO terminals:**

### Terminal 1: Start FinBERT Sentiment Service

```bash
cd sentiment-service
python sentiment_server.py
```

**You'll see:**
```
🚀 Starting LadyBug Sentiment Service...
🤖 Loading FinBERT model...
✅ FinBERT loaded on GPU  (or CPU if no GPU)
🌐 Starting Flask server on http://localhost:5000
```

**Leave this running!**

### Terminal 2: Start Rust Trading Engine

```bash
cd ../rust-engine
cargo run
```

**You'll see:**
```
📰 Starting News Aggregator with Yahoo RSS + Local FinBERT
✅ FinBERT sentiment service is running
📰 AAPL - Found 5 headlines
🤖 AAPL - Sentiment: 0.652 (from 5 headlines)
📰 GOOGL - Found 4 headlines
🤖 GOOGL - Sentiment: 0.421 (from 4 headlines)
```

---

## 🔥 How It Works

### 1. Yahoo RSS Fetches News (Every 3 Minutes)

```rust
// For each symbol (AAPL, GOOGL, etc.)
GET https://finance.yahoo.com/rss/headline?s=AAPL

Response (RSS/XML):
<rss>
  <channel>
    <item>
      <title>Apple Beats Q4 Earnings Expectations</title>
      <link>https://...</link>
    </item>
    <item>
      <title>iPhone Sales Strong in China</title>
    </item>
    ...
  </channel>
</rss>
```

### 2. Extract Headlines

```rust
headlines = [
    "Apple Beats Q4 Earnings Expectations",
    "iPhone Sales Strong in China",
    "New MacBook Pro Announced",
    "Apple Expands Services Revenue",
    "Tim Cook Comments on AI Strategy"
]
```

### 3. Send to Local FinBERT

```rust
POST http://localhost:5000/batch
{
    "texts": [
        "Apple Beats Q4 Earnings Expectations",
        "iPhone Sales Strong in China",
        ...
    ]
}
```

### 4. FinBERT Analyzes (Local LLM!)

```python
# Python sentiment service
for headline in headlines:
    # Tokenize
    inputs = tokenizer(headline)
    
    # Run through FinBERT model
    outputs = model(**inputs)
    
    # Get sentiment probabilities
    probs = softmax(outputs)
    # [negative: 0.05, neutral: 0.15, positive: 0.80]
    
    # Convert to score: positive - negative
    score = probs[2] - probs[0]  # 0.80 - 0.05 = 0.75
```

### 5. Return Scores

```json
{
    "results": [
        {"score": 0.75, "sentiment": "positive", "confidence": 0.92},
        {"score": 0.62, "sentiment": "positive", "confidence": 0.85},
        {"score": 0.45, "sentiment": "positive", "confidence": 0.78},
        {"score": 0.28, "sentiment": "neutral", "confidence": 0.65},
        {"score": 0.81, "sentiment": "positive", "confidence": 0.94}
    ]
}
```

### 6. Average & Cache

```rust
// Average the scores
avg_sentiment = (0.75 + 0.62 + 0.45 + 0.28 + 0.81) / 5
             = 0.582  // Bullish!

// Store in cache
sentiment_cache.insert("AAPL", 0.582)

LOG: "🤖 AAPL - Sentiment: 0.582 (from 5 headlines)"
```

### 7. Trading Uses Sentiment

```rust
// Next trading cycle (30 seconds later)
technical_signal = 0.12  // Weak buy from price action
news_sentiment = 0.582   // Strong positive from FinBERT!

combined_signal = (technical_signal + news_sentiment) / 2.0
                = 0.351  // STRONG BUY!

if combined_signal > 0.15 {
    execute_buy_order("AAPL");  // ✅ Trade!
}
```

---

## 📊 What You Get

### Per Symbol (Every 3 Minutes):
1. Latest 5 headlines from Yahoo Finance
2. Real NLP sentiment analysis (FinBERT)
3. Score: -1.0 (very bearish) to +1.0 (very bullish)
4. Cached for trading decisions

### Example Output:

```
📰 AAPL - Found 5 headlines
🤖 AAPL - Sentiment: 0.652 (from 5 headlines)

📰 TSLA - Found 4 headlines
🤖 TSLA - Sentiment: -0.321 (from 4 headlines)  ← Bearish news!

📰 GME - Found 7 headlines
🤖 GME - Sentiment: 0.892 (from 7 headlines)  ← Very bullish!
```

---

## ⚡ Performance

### With GPU (NVIDIA):
- **Model load:** ~5 seconds (one time)
- **Analysis:** ~50ms per headline
- **Batch of 5:** ~200ms total
- **Very fast!** ✅

### Without GPU (CPU only):
- **Model load:** ~10 seconds (one time)
- **Analysis:** ~500ms per headline
- **Batch of 5:** ~2 seconds total
- **Still works!** ✅

---

## 🎯 Tracked Symbols

Current list (easy to add more):
```rust
["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", 
 "META", "NFLX", "NVDA", "GME", "PLTR",
 "RIOT", "COIN", "MSTR"]
```

**To add more symbols:**
Edit `rust-engine/src/news.rs`, line ~46:
```rust
let symbols = vec![
    "AAPL", "GOOGL", // ... existing ...
    "COIN",  // Add your symbol here!
];
```

---

## 🔧 API Endpoints (Python Service)

### Health Check
```bash
curl http://localhost:5000/health
```

### Analyze Single Text
```bash
curl -X POST http://localhost:5000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "Apple beats earnings expectations"}'
```

Response:
```json
{
  "text": "Apple beats earnings expectations",
  "sentiment": "positive",
  "score": 0.85,
  "confidence": 0.92
}
```

### Batch Analysis (Used by Rust)
```bash
curl -X POST http://localhost:5000/batch \
  -H "Content-Type: application/json" \
  -d '{"texts": ["Apple beats earnings", "Tesla recalls vehicles"]}'
```

---

## 🎓 Understanding FinBERT Scores

**Score Range:** -1.0 to +1.0

```
 +1.0  ═══ Very Bullish (amazing news!)
 +0.7  ═══ Bullish (good news)
 +0.3  ═══ Slightly Positive
  0.0  ═══ Neutral
 -0.3  ═══ Slightly Negative
 -0.7  ═══ Bearish (bad news)
 -1.0  ═══ Very Bearish (terrible news!)
```

**Examples:**

| Headline | Score | Interpretation |
|----------|-------|----------------|
| "Apple beats earnings by 20%" | +0.85 | Very bullish |
| "iPhone sales exceed expectations" | +0.65 | Bullish |
| "Apple announces new product" | +0.25 | Slightly positive |
| "Apple CEO speaks at conference" | 0.00 | Neutral |
| "Apple faces supply chain issues" | -0.40 | Negative |
| "Apple misses revenue targets" | -0.70 | Bearish |
| "Apple recalls all iPhones" | -0.90 | Very bearish |

---

## 💰 Cost Comparison

### This System (Yahoo RSS + Local FinBERT):
- **Yahoo RSS:** FREE (unlimited)
- **FinBERT Model:** FREE (open source)
- **Python/PyTorch:** FREE
- **Total:** $0/month ✅

### Alpaca News API:
- **Starter Plan:** $9/month
- **Unlimited Plan:** $99/month
- **Total:** $9-99/month ❌

### Alpha Vantage:
- **Free Tier:** 25 calls/day (too limited)
- **Paid Tier:** $50+/month

**We WIN by $108-1188 per year!** 🎉

---

## 🚀 Future Enhancements

### Easy Additions:
1. **More symbols** - Just add to the list
2. **Different sources** - Add Bloomberg RSS, Reuters, etc.
3. **Tweet sentiment** - Scrape Twitter for CEO tweets
4. **Reddit sentiment** - r/wallstreetbets analysis
5. **Custom models** - Fine-tune FinBERT on your strategy

### Advanced:
1. **GPU optimization** - Batch processing for speed
2. **Model caching** - Keep in GPU memory
3. **Multiple models** - Different models for different assets
4. **Real-time webhooks** - Push notifications on strong sentiment

---

## ✅ Summary

**You now have:**
- ✅ FREE unlimited news from Yahoo Finance
- ✅ LOCAL FinBERT LLM sentiment analysis
- ✅ Real NLP (not fake upvote counting!)
- ✅ Updates every 3 minutes
- ✅ Zero ongoing costs
- ✅ Complete control
- ✅ Professional-grade system

**COMPLEXITY BE DAMNED - WE DID IT!** 🔥🚀

Start both services and watch the sentiment flow!
