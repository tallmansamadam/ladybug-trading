"""
LadyBug Sentiment Analysis Service
Uses FinBERT for financial sentiment analysis
"""

from flask import Flask, request, jsonify
from transformers import AutoTokenizer, AutoModelForSequenceClassification
import torch
import logging
from typing import Dict, List

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)

# Global model and tokenizer (loaded once at startup)
model = None
tokenizer = None

def load_model():
    """Load FinBERT model and tokenizer"""
    global model, tokenizer
    
    logger.info("🤖 Loading FinBERT model...")
    
    model_name = "ProsusAI/finbert"
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForSequenceClassification.from_pretrained(model_name)
    
    # Move to GPU if available
    if torch.cuda.is_available():
        model = model.cuda()
        logger.info("✅ FinBERT loaded on GPU")
    else:
        logger.info("✅ FinBERT loaded on CPU (slower)")
    
    model.eval()  # Set to evaluation mode

def analyze_sentiment(text: str) -> Dict[str, float]:
    """
    Analyze sentiment of text using FinBERT
    
    Returns:
        {
            'sentiment': 'positive'|'negative'|'neutral',
            'score': float between -1.0 and 1.0,
            'confidence': float between 0.0 and 1.0
        }
    """
    # Tokenize
    inputs = tokenizer(text, return_tensors="pt", truncation=True, max_length=512)
    
    # Move to GPU if available
    if torch.cuda.is_available():
        inputs = {k: v.cuda() for k, v in inputs.items()}
    
    # Get predictions
    with torch.no_grad():
        outputs = model(**inputs)
        predictions = torch.nn.functional.softmax(outputs.logits, dim=-1)
    
    # FinBERT outputs: [negative, neutral, positive]
    probs = predictions[0].cpu().numpy()
    
    # Map to sentiment
    labels = ['negative', 'neutral', 'positive']
    sentiment_idx = probs.argmax()
    sentiment = labels[sentiment_idx]
    confidence = float(probs[sentiment_idx])
    
    # Convert to score: -1.0 (bearish) to +1.0 (bullish)
    score = float(probs[2] - probs[0])  # positive - negative
    
    return {
        'sentiment': sentiment,
        'score': score,
        'confidence': confidence
    }

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        'status': 'healthy',
        'model_loaded': model is not None,
        'gpu_available': torch.cuda.is_available()
    })

@app.route('/analyze', methods=['POST'])
def analyze():
    """
    Analyze sentiment of text
    
    Request body:
        {
            "text": "Apple beats earnings expectations"
        }
    
    Response:
        {
            "text": "Apple beats earnings expectations",
            "sentiment": "positive",
            "score": 0.85,
            "confidence": 0.92
        }
    """
    try:
        data = request.get_json()
        
        if 'text' not in data:
            return jsonify({'error': 'Missing "text" field'}), 400
        
        text = data['text']
        
        if not text.strip():
            return jsonify({'error': 'Empty text'}), 400
        
        # Analyze
        result = analyze_sentiment(text)
        result['text'] = text
        
        logger.info(f"📊 Analyzed: '{text[:50]}...' → {result['sentiment']} ({result['score']:.3f})")
        
        return jsonify(result)
    
    except Exception as e:
        logger.error(f"❌ Error: {e}")
        return jsonify({'error': str(e)}), 500

@app.route('/batch', methods=['POST'])
def batch_analyze():
    """
    Analyze multiple texts at once
    
    Request body:
        {
            "texts": [
                "Apple beats earnings",
                "Tesla recalls vehicles"
            ]
        }
    
    Response:
        {
            "results": [
                {"text": "...", "sentiment": "positive", "score": 0.85, ...},
                {"text": "...", "sentiment": "negative", "score": -0.65, ...}
            ]
        }
    """
    try:
        data = request.get_json()
        
        if 'texts' not in data:
            return jsonify({'error': 'Missing "texts" field'}), 400
        
        texts = data['texts']
        
        if not isinstance(texts, list):
            return jsonify({'error': '"texts" must be an array'}), 400
        
        results = []
        for text in texts:
            if text.strip():
                result = analyze_sentiment(text)
                result['text'] = text
                results.append(result)
        
        logger.info(f"📊 Batch analyzed {len(results)} texts")
        
        return jsonify({'results': results})
    
    except Exception as e:
        logger.error(f"❌ Error: {e}")
        return jsonify({'error': str(e)}), 500

if __name__ == '__main__':
    logger.info("🚀 Starting LadyBug Sentiment Service...")
    
    # Load model at startup
    load_model()
    
    logger.info("🌐 Starting Flask server on http://localhost:5000")
    logger.info("📡 Endpoints:")
    logger.info("   GET  /health - Health check")
    logger.info("   POST /analyze - Analyze single text")
    logger.info("   POST /batch - Analyze multiple texts")
    
    # Run Flask server
    app.run(host='0.0.0.0', port=5000, debug=False)
