import React, { useMemo } from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Cell,
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

interface Props {
  positions: Position[];
}

export function PositionsPnLChart({ positions }: Props) {
  const chartData = useMemo(() => {
    return positions.map((position) => ({
      symbol: position.symbol,
      pnl: position.pnl,
      pnl_percent: position.pnl_percent,
      market_value: position.market_value,
      asset_type: position.asset_type,
    })).sort((a, b) => b.pnl - a.pnl); // Sort by P&L descending
  }, [positions]);

  const formatCurrency = (value: number) => {
    return `$${value.toLocaleString('en-US', { 
      minimumFractionDigits: 2,
      maximumFractionDigits: 2 
    })}`;
  };

  const formatPercent = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
  };

  const getBarColor = (pnl: number) => {
    return pnl >= 0 ? '#10b981' : '#ef4444'; // Green for profit, red for loss
  };

  // Calculate total P&L
  const totalPnL = positions.reduce((sum, p) => sum + p.pnl, 0);
  const stockPnL = positions.filter(p => p.asset_type === 'stock').reduce((sum, p) => sum + p.pnl, 0);
  const cryptoPnL = positions.filter(p => p.asset_type === 'crypto').reduce((sum, p) => sum + p.pnl, 0);

  return (
    <div className="pnl-chart-container">
      <div className="chart-header">
        <h2>Positions P&L</h2>
        <div className="pnl-summary">
          <div className="pnl-item total">
            <span className="label">Total P&L:</span>
            <span className={`value ${totalPnL >= 0 ? 'positive' : 'negative'}`}>
              {formatCurrency(totalPnL)}
            </span>
          </div>
          <div className="pnl-item stocks">
            <span className="label">Stocks:</span>
            <span className={`value ${stockPnL >= 0 ? 'positive' : 'negative'}`}>
              {formatCurrency(stockPnL)}
            </span>
          </div>
          <div className="pnl-item crypto">
            <span className="label">Crypto:</span>
            <span className={`value ${cryptoPnL >= 0 ? 'positive' : 'negative'}`}>
              {formatCurrency(cryptoPnL)}
            </span>
          </div>
        </div>
      </div>

      <ResponsiveContainer width="100%" height={400}>
        <BarChart
          data={chartData}
          margin={{ top: 20, right: 30, left: 20, bottom: 60 }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#e0e0e0" />
          <XAxis 
            dataKey="symbol" 
            angle={-45}
            textAnchor="end"
            height={80}
            stroke="#666"
            style={{ fontSize: '12px' }}
          />
          <YAxis 
            stroke="#666"
            style={{ fontSize: '12px' }}
            tickFormatter={(value) => `$${(value / 1000).toFixed(1)}k`}
          />
          <Tooltip 
            formatter={(value: number, name: string, props: any) => {
              if (name === 'pnl') {
                return [
                  formatCurrency(value),
                  `P&L (${formatPercent(props.payload.pnl_percent)})`
                ];
              }
              return [formatCurrency(value), name];
            }}
            contentStyle={{
              backgroundColor: '#fff',
              border: '1px solid #ccc',
              borderRadius: '4px',
              padding: '10px',
            }}
            labelFormatter={(label) => `${label}`}
          />
          <Legend 
            wrapperStyle={{ paddingTop: '20px' }}
            formatter={(value) => value === 'pnl' ? 'Profit & Loss' : value}
          />
          <Bar dataKey="pnl" name="P&L" radius={[8, 8, 0, 0]}>
            {chartData.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={getBarColor(entry.pnl)} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>

      <div className="chart-footer">
        <div className="legend-custom">
          <div className="legend-item">
            <div className="color-box positive"></div>
            <span>Profit</span>
          </div>
          <div className="legend-item">
            <div className="color-box negative"></div>
            <span>Loss</span>
          </div>
        </div>
      </div>

      <style jsx>{`
        .pnl-chart-container {
          background: white;
          border-radius: 8px;
          padding: 20px;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
          margin-bottom: 20px;
        }

        .chart-header {
          margin-bottom: 20px;
        }

        .chart-header h2 {
          margin: 0 0 15px 0;
          font-size: 24px;
          font-weight: 600;
          color: #1f2937;
        }

        .pnl-summary {
          display: flex;
          gap: 30px;
          flex-wrap: wrap;
        }

        .pnl-item {
          display: flex;
          flex-direction: column;
          gap: 5px;
        }

        .pnl-item .label {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        .pnl-item .value {
          font-size: 20px;
          font-weight: 700;
        }

        .pnl-item .value.positive {
          color: #10b981;
        }

        .pnl-item .value.negative {
          color: #ef4444;
        }

        .pnl-item.total .value {
          font-size: 24px;
        }

        .chart-footer {
          margin-top: 20px;
          display: flex;
          justify-content: center;
        }

        .legend-custom {
          display: flex;
          gap: 30px;
        }

        .legend-item {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .color-box {
          width: 20px;
          height: 20px;
          border-radius: 4px;
        }

        .color-box.positive {
          background-color: #10b981;
        }

        .color-box.negative {
          background-color: #ef4444;
        }

        .legend-item span {
          font-size: 14px;
          color: #4b5563;
          font-weight: 500;
        }

        @media (max-width: 768px) {
          .pnl-summary {
            flex-direction: column;
            gap: 15px;
          }

          .chart-header h2 {
            font-size: 20px;
          }
        }
      `}</style>
    </div>
  );
}
