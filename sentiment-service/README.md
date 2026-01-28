# LadyBug Sentiment Analysis Service

Local FinBERT sentiment analysis service for trading.

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Start the service
python sentiment_server.py
```

The service will:
1. Download FinBERT model (~400MB, first run only)
2. Start Flask server on http://localhost:5000
3. Ready to analyze sentiment!

## API Endpoints

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

### Batch Analysis
```bash
curl -X POST http://localhost:5000/batch \
  -H "Content-Type: application/json" \
  -d '{"texts": ["Apple beats earnings", "Tesla recalls vehicles"]}'
```

## GPU Support

- **With GPU:** Uses CUDA for fast inference (~50ms per text)
- **Without GPU:** Uses CPU (slower ~500ms per text, but works!)

## Model Info

- **Model:** ProsusAI/finbert
- **Type:** BERT fine-tuned on financial text
- **Output:** Sentiment classification (positive/negative/neutral)
- **Score:** -1.0 (very bearish) to +1.0 (very bullish)
