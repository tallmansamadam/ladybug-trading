import React, { useState, useEffect } from 'react';

type TradingMode = 'Conservative' | 'Volatile' | 'Hybrid';

interface TradingModeSelectorProps {
  onModeChange?: (mode: TradingMode) => void;
}

export function TradingModeSelector({ onModeChange }: TradingModeSelectorProps) {
  const [mode, setMode] = useState<TradingMode>('Conservative');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Fetch current mode from backend
    fetch('http://localhost:8080/trading-mode')
      .then(res => res.json())
      .then(data => setMode(data as TradingMode))
      .catch(err => console.error('Failed to fetch trading mode:', err));
  }, []);

  const handleModeChange = async (newMode: TradingMode) => {
    setLoading(true);
    setMode(newMode);
    
    try {
      // Update backend
      await fetch('http://localhost:8080/trading-mode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: newMode }),
      });
      
      if (onModeChange) {
        onModeChange(newMode);
      }
    } catch (err) {
      console.error('Failed to update trading mode:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="trading-mode-selector">
      <h3>⚡ Trading Strategy</h3>
      <p className="subtitle">Select volatility profile for maximum profit potential</p>
      
      <div className="modes-container">
        <label className={`mode-option ${mode === 'Conservative' ? 'active' : ''} ${loading ? 'loading' : ''}`}>
          <input
            type="radio"
            name="trading-mode"
            value="Conservative"
            checked={mode === 'Conservative'}
            onChange={() => handleModeChange('Conservative')}
            disabled={loading}
          />
          <div className="mode-content">
            <div className="mode-header">
              <span className="mode-icon">🛡️</span>
              <span className="mode-name">Conservative</span>
            </div>
            <p className="mode-desc">Stable mega-caps + major crypto</p>
            <div className="mode-stats">
              <span className="stat">📊 20 stocks</span>
              <span className="stat">₿ 3 crypto</span>
              <span className="stat">📈 3-5% volatility</span>
            </div>
            <div className="mode-examples">
              AAPL, GOOGL, MSFT, BTC, ETH
            </div>
          </div>
        </label>

        <label className={`mode-option ${mode === 'Volatile' ? 'active' : ''} ${loading ? 'loading' : ''}`}>
          <input
            type="radio"
            name="trading-mode"
            value="Volatile"
            checked={mode === 'Volatile'}
            onChange={() => handleModeChange('Volatile')}
            disabled={loading}
          />
          <div className="mode-content">
            <div className="mode-header">
              <span className="mode-icon">⚡</span>
              <span className="mode-name">Volatile</span>
              <span className="mode-badge">MAX PROFIT</span>
            </div>
            <p className="mode-desc">High beta stocks + altcoins</p>
            <div className="mode-stats">
              <span className="stat">📊 20 stocks</span>
              <span className="stat">₿ 6 crypto</span>
              <span className="stat">📈 10-30% volatility</span>
            </div>
            <div className="mode-examples">
              TSLA, GME, RIOT, MSTR, SOL, DOGE
            </div>
          </div>
        </label>

        <label className={`mode-option ${mode === 'Hybrid' ? 'active' : ''} ${loading ? 'loading' : ''}`}>
          <input
            type="radio"
            name="trading-mode"
            value="Hybrid"
            checked={mode === 'Hybrid'}
            onChange={() => handleModeChange('Hybrid')}
            disabled={loading}
          />
          <div className="mode-content">
            <div className="mode-header">
              <span className="mode-icon">🔄</span>
              <span className="mode-name">Hybrid</span>
            </div>
            <p className="mode-desc">50% stable + 50% volatile</p>
            <div className="mode-stats">
              <span className="stat">📊 20 stocks</span>
              <span className="stat">₿ 5 crypto</span>
              <span className="stat">📈 6-15% volatility</span>
            </div>
            <div className="mode-examples">
              Mix of AAPL, GOOGL, TSLA, GME, BTC, SOL
            </div>
          </div>
        </label>
      </div>

      <style jsx>{`
        .trading-mode-selector {
          background: var(--cyber-bg-card, #151b2b);
          border: 1px solid var(--cyber-border, #1e2942);
          border-radius: 8px;
          padding: 25px;
          margin-bottom: 20px;
        }

        .trading-mode-selector h3 {
          margin: 0 0 5px 0;
          font-size: 20px;
          font-weight: 700;
          color: var(--cyber-accent-cyan, #00d9ff);
          text-transform: uppercase;
          letter-spacing: 2px;
        }

        .subtitle {
          margin: 0 0 20px 0;
          font-size: 13px;
          color: var(--cyber-text-secondary, #8b95a5);
        }

        .modes-container {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 15px;
        }

        .mode-option {
          display: flex;
          align-items: flex-start;
          padding: 20px;
          background: var(--cyber-bg-secondary, #0f1419);
          border: 2px solid var(--cyber-border, #1e2942);
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.3s ease;
          position: relative;
        }

        .mode-option:hover {
          border-color: var(--cyber-accent-cyan, #00d9ff);
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 217, 255, 0.2);
        }

        .mode-option.active {
          border-color: var(--cyber-accent-cyan, #00d9ff);
          background: var(--cyber-bg-card, #151b2b);
          box-shadow: 0 0 20px rgba(0, 217, 255, 0.3);
        }

        .mode-option.active::before {
          content: '✓';
          position: absolute;
          top: 10px;
          right: 10px;
          width: 24px;
          height: 24px;
          background: var(--cyber-accent-green, #00ff88);
          color: #000;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: 700;
          font-size: 14px;
        }

        .mode-option.loading {
          opacity: 0.6;
          cursor: wait;
        }

        .mode-option input[type="radio"] {
          margin: 5px 15px 0 0;
          width: 20px;
          height: 20px;
          cursor: pointer;
          accent-color: var(--cyber-accent-cyan, #00d9ff);
        }

        .mode-content {
          flex: 1;
        }

        .mode-header {
          display: flex;
          align-items: center;
          gap: 10px;
          margin-bottom: 10px;
        }

        .mode-icon {
          font-size: 24px;
        }

        .mode-name {
          font-size: 18px;
          font-weight: 700;
          color: var(--cyber-text-primary, #e0e6ed);
        }

        .mode-badge {
          font-size: 10px;
          padding: 3px 8px;
          background: var(--cyber-accent-magenta, #ff00ff);
          color: #fff;
          border-radius: 4px;
          font-weight: 700;
          letter-spacing: 1px;
        }

        .mode-desc {
          margin: 0 0 12px 0;
          font-size: 14px;
          color: var(--cyber-text-secondary, #8b95a5);
        }

        .mode-stats {
          display: flex;
          gap: 12px;
          margin-bottom: 10px;
          flex-wrap: wrap;
        }

        .stat {
          font-size: 11px;
          color: var(--cyber-accent-cyan, #00d9ff);
          font-family: 'Roboto Mono', monospace;
          background: rgba(0, 217, 255, 0.1);
          padding: 3px 8px;
          border-radius: 3px;
        }

        .mode-examples {
          font-size: 11px;
          color: var(--cyber-text-secondary, #8b95a5);
          font-style: italic;
          margin-top: 8px;
          padding-top: 8px;
          border-top: 1px solid var(--cyber-border, #1e2942);
        }

        @media (max-width: 768px) {
          .modes-container {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
}
