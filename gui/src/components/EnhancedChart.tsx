import React, { useState, useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';

interface Position {
  symbol: string;
  quantity: number;
  entry_price: number;
  current_price: number;
  pnl: number;
  pnl_percent: number;
  market_value: number;
  asset_type: 'stock' | 'crypto';
}

interface PortfolioSnapshot {
  timestamp: string;
  total_value: number;
  cash: number;
  positions_value: number;
}

interface Props {
  portfolioHistory: PortfolioSnapshot[];
  positions: Position[];
}

export function EnhancedChart({ portfolioHistory, positions }: Props) {
  const [showTotal, setShowTotal] = useState(true);
  const [showStocks, setShowStocks] = useState(true);
  const [showCrypto, setShowCrypto] = useState(true);
  const [showCash, setShowCash] = useState(true);

  const chartData = useMemo(() => {
    return portfolioHistory.map((snapshot) => {
      // Calculate stock value from current positions
      const stockValue = positions
        .filter((p) => p.asset_type === 'stock')
        .reduce((sum, p) => sum + p.market_value, 0);

      // Calculate crypto value from current positions
      const cryptoValue = positions
        .filter((p) => p.asset_type === 'crypto')
        .reduce((sum, p) => sum + p.market_value, 0);

      return {
        timestamp: new Date(snapshot.timestamp).toLocaleTimeString(),
        total: snapshot.total_value,
        stocks: stockValue,
        crypto: cryptoValue,
        cash: snapshot.cash,
      };
    });
  }, [portfolioHistory, positions]);

  const formatCurrency = (value: number) => {
    return `$${value.toLocaleString('en-US', { 
      minimumFractionDigits: 2,
      maximumFractionDigits: 2 
    })}`;
  };

  return (
    <div className="enhanced-chart-container">
      <div className="chart-header">
        <h2>Portfolio Performance</h2>
        <div className="chart-controls">
          <label className="chart-toggle">
            <input
              type="checkbox"
              checked={showTotal}
              onChange={() => setShowTotal(!showTotal)}
            />
            <span className="label-text total">Total Holdings</span>
          </label>
          <label className="chart-toggle">
            <input
              type="checkbox"
              checked={showStocks}
              onChange={() => setShowStocks(!showStocks)}
            />
            <span className="label-text stocks">Stocks</span>
          </label>
          <label className="chart-toggle">
            <input
              type="checkbox"
              checked={showCrypto}
              onChange={() => setShowCrypto(!showCrypto)}
            />
            <span className="label-text crypto">Crypto</span>
          </label>
          <label className="chart-toggle">
            <input
              type="checkbox"
              checked={showCash}
              onChange={() => setShowCash(!showCash)}
            />
            <span className="label-text cash">Cash</span>
          </label>
        </div>
      </div>

      <ResponsiveContainer width="100%" height={400}>
        <LineChart
          data={chartData}
          margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#e0e0e0" />
          <XAxis 
            dataKey="timestamp" 
            stroke="#666"
            style={{ fontSize: '12px' }}
          />
          <YAxis 
            stroke="#666"
            style={{ fontSize: '12px' }}
            tickFormatter={(value) => `$${(value / 1000).toFixed(0)}k`}
          />
          <Tooltip 
            formatter={formatCurrency}
            contentStyle={{
              backgroundColor: '#fff',
              border: '1px solid #ccc',
              borderRadius: '4px',
              padding: '10px',
            }}
          />
          <Legend 
            wrapperStyle={{ paddingTop: '20px' }}
          />

          {showTotal && (
            <Line
              type="monotone"
              dataKey="total"
              stroke="#3b82f6"
              strokeWidth={3}
              name="Total Holdings"
              dot={false}
              activeDot={{ r: 6 }}
            />
          )}
          {showStocks && (
            <Line
              type="monotone"
              dataKey="stocks"
              stroke="#10b981"
              strokeWidth={2}
              name="Stocks"
              dot={false}
              activeDot={{ r: 5 }}
            />
          )}
          {showCrypto && (
            <Line
              type="monotone"
              dataKey="crypto"
              stroke="#f59e0b"
              strokeWidth={2}
              name="Crypto"
              dot={false}
              activeDot={{ r: 5 }}
            />
          )}
          {showCash && (
            <Line
              type="monotone"
              dataKey="cash"
              stroke="#ef4444"
              strokeWidth={2}
              name="Cash"
              dot={false}
              strokeDasharray="5 5"
              activeDot={{ r: 5 }}
            />
          )}
        </LineChart>
      </ResponsiveContainer>

      <style jsx>{`
        .enhanced-chart-container {
          background: white;
          border-radius: 8px;
          padding: 20px;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .chart-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }

        .chart-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #1f2937;
        }

        .chart-controls {
          display: flex;
          gap: 20px;
        }

        .chart-toggle {
          display: flex;
          align-items: center;
          cursor: pointer;
          user-select: none;
        }

        .chart-toggle input[type="checkbox"] {
          margin-right: 8px;
          width: 18px;
          height: 18px;
          cursor: pointer;
        }

        .label-text {
          font-size: 14px;
          font-weight: 500;
        }

        .label-text.total { color: #3b82f6; }
        .label-text.stocks { color: #10b981; }
        .label-text.crypto { color: #f59e0b; }
        .label-text.cash { color: #ef4444; }

        @media (max-width: 768px) {
          .chart-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 15px;
          }

          .chart-controls {
            flex-wrap: wrap;
            gap: 10px;
          }
        }
      `}</style>
    </div>
  );
}
