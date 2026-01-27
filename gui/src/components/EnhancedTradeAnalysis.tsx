import React from 'react'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'

interface TradeRecord {
  id: string
  timestamp: string
  symbol: string
  action: string
  quantity: number
  price: number
  pnl: number
}

interface EnhancedTradeAnalysisProps {
  tradeHistory: TradeRecord[]
}

export function EnhancedTradeAnalysis({ tradeHistory }: EnhancedTradeAnalysisProps) {
  const [selectedSymbol, setSelectedSymbol] = React.useState<string>('ALL')
  const [expanded, setExpanded] = React.useState(false)

  // Calculate P&L by symbol
  const symbolPnL = tradeHistory
    .filter(t => t.action === 'SELL')
    .reduce((acc, t) => {
      acc[t.symbol] = (acc[t.symbol] || 0) + t.pnl
      return acc
    }, {} as Record<string, number>)

  const symbolPerfData = Object.entries(symbolPnL)
    .map(([symbol, pnl]) => ({ symbol, pnl }))
    .sort((a, b) => b.pnl - a.pnl) // Sort by P&L descending

  const symbols = ['ALL', ...Object.keys(symbolPnL).sort()]

  const filteredTrades = selectedSymbol === 'ALL'
    ? tradeHistory.filter(t => t.action === 'SELL')
    : tradeHistory.filter(t => t.symbol === selectedSymbol && t.action === 'SELL')

  const displayData = selectedSymbol === 'ALL' 
    ? symbolPerfData 
    : [{ symbol: selectedSymbol, pnl: symbolPnL[selectedSymbol] || 0 }]

  return (
    <div style={{
      background: 'rgba(255,255,255,0.1)',
      backdropFilter: 'blur(10px)',
      borderRadius: '1rem',
      padding: '1.5rem',
      marginBottom: '1.5rem',
      border: '1px solid rgba(255,255,255,0.2)'
    }}>
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center', 
        marginBottom: '1rem' 
      }}>
        <h2 style={{ margin: 0 }}>📈 P&L by Symbol</h2>
        <button
          onClick={() => setExpanded(!expanded)}
          style={{
            padding: '0.5rem 1rem',
            background: 'rgba(255,255,255,0.2)',
            border: '1px solid rgba(255,255,255,0.3)',
            borderRadius: '0.5rem',
            color: '#fff',
            cursor: 'pointer',
            fontSize: '0.875rem'
          }}
        >
          {expanded ? '▲ Collapse Filters' : '▼ Expand Filters'}
        </button>
      </div>

      {expanded && (
        <div style={{
          marginBottom: '1.5rem',
          padding: '1rem',
          background: 'rgba(0,0,0,0.2)',
          borderRadius: '0.5rem',
          maxHeight: '300px',
          overflowY: 'auto'
        }}>
          <div style={{ fontSize: '0.875rem', fontWeight: 'bold', marginBottom: '0.75rem', opacity: 0.9 }}>
            Filter by Symbol:
          </div>
          <div style={{ 
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))',
            gap: '0.5rem'
          }}>
            {symbols.map(symbol => {
              const pnl = symbol === 'ALL' ? null : symbolPnL[symbol]
              const isSelected = selectedSymbol === symbol
              
              return (
                <label 
                  key={symbol}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.5rem',
                    padding: '0.5rem',
                    background: isSelected ? 'rgba(124, 58, 237, 0.3)' : 'rgba(255,255,255,0.05)',
                    borderRadius: '0.375rem',
                    cursor: 'pointer',
                    border: `2px solid ${isSelected ? '#7c3aed' : 'transparent'}`,
                    transition: 'all 0.2s'
                  }}
                  onMouseEnter={(e) => {
                    if (!isSelected) {
                      e.currentTarget.style.background = 'rgba(255,255,255,0.1)'
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (!isSelected) {
                      e.currentTarget.style.background = 'rgba(255,255,255,0.05)'
                    }
                  }}
                >
                  <input 
                    type="radio"
                    name="symbol-filter"
                    value={symbol}
                    checked={isSelected}
                    onChange={(e) => setSelectedSymbol(e.target.value)}
                    style={{ cursor: 'pointer' }}
                  />
                  <div style={{ flex: 1, fontSize: '0.875rem' }}>
                    <div style={{ fontWeight: 'bold' }}>{symbol}</div>
                    {pnl !== null && (
                      <div style={{ 
                        fontSize: '0.75rem',
                        color: pnl >= 0 ? '#10b981' : '#ef4444'
                      }}>
                        ${pnl.toFixed(2)}
                      </div>
                    )}
                  </div>
                </label>
              )
            })}
          </div>
        </div>
      )}

      <ResponsiveContainer width="100%" height={350}>
        <BarChart data={displayData}>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
          <XAxis dataKey="symbol" stroke="#fff" />
          <YAxis stroke="#fff" />
          <Tooltip
            contentStyle={{ 
              background: 'rgba(0,0,0,0.8)', 
              border: 'none', 
              borderRadius: '0.5rem',
              padding: '0.75rem'
            }}
            labelStyle={{ color: '#fff', fontWeight: 'bold' }}
            formatter={(value: number) => [`$${value.toFixed(2)}`, 'P&L']}
          />
          <Legend />
          <Bar dataKey="pnl" name="Total P&L">
            {displayData.map((entry, index) => (
              <Bar 
                key={`cell-${index}`} 
                dataKey="pnl" 
                fill={entry.pnl >= 0 ? '#10b981' : '#ef4444'} 
              />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>

      {filteredTrades.length > 0 && (
        <div style={{ marginTop: '1.5rem' }}>
          <h3 style={{ fontSize: '1rem', opacity: 0.9, marginBottom: '0.75rem' }}>
            Recent Trades {selectedSymbol !== 'ALL' && `(${selectedSymbol})`}
          </h3>
          <div style={{ 
            maxHeight: '200px', 
            overflowY: 'auto',
            background: 'rgba(0,0,0,0.2)',
            borderRadius: '0.5rem',
            padding: '0.75rem'
          }}>
            {filteredTrades.slice(-10).reverse().map((trade) => (
              <div
                key={trade.id}
                style={{
                  padding: '0.5rem',
                  marginBottom: '0.5rem',
                  background: 'rgba(255,255,255,0.05)',
                  borderRadius: '0.375rem',
                  borderLeft: `3px solid ${trade.pnl >= 0 ? '#10b981' : '#ef4444'}`,
                  fontSize: '0.875rem'
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.25rem' }}>
                  <span style={{ fontWeight: 'bold' }}>{trade.symbol}</span>
                  <span style={{ opacity: 0.7 }}>
                    {new Date(trade.timestamp).toLocaleString()}
                  </span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.75rem' }}>
                  <span>
                    {trade.action} {trade.quantity.toFixed(2)} @ ${trade.price.toFixed(2)}
                  </span>
                  <span style={{ 
                    fontWeight: 'bold',
                    color: trade.pnl >= 0 ? '#10b981' : '#ef4444'
                  }}>
                    {trade.pnl >= 0 ? '+' : ''}${trade.pnl.toFixed(2)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {filteredTrades.length === 0 && (
        <div style={{ 
          textAlign: 'center', 
          padding: '2rem', 
          opacity: 0.6 
        }}>
          No trades yet for {selectedSymbol === 'ALL' ? 'any symbol' : selectedSymbol}
        </div>
      )}
    </div>
  )
}
