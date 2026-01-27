import { useState, useEffect } from 'react'
import axios from 'axios'
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'
import { TradingModeSelector } from './TradingModeSelector'

const API_URL = 'http://localhost:8080'

interface SystemStatus {
  running: boolean
  trading_enabled: boolean
  active_positions: number
}

interface Position {
  symbol: string
  quantity: number
  entry_price: number
  current_price: number
  pnl: number
  pnl_percent: number
  market_value: number
  asset_type: string
}

interface ActivityLog {
  id: string
  timestamp: string
  level: 'info' | 'success' | 'warning' | 'error'
  category: string
  message: string
  symbol?: string
}

interface PortfolioSnapshot {
  timestamp: string
  total_value: number
  cash: number
  positions_value: number
}

interface TradeRecord {
  id: string
  timestamp: string
  symbol: string
  action: string
  quantity: number
  price: number
  pnl: number
}

export default function Dashboard() {
  const [status, setStatus] = useState<SystemStatus | null>(null)
  const [positions, setPositions] = useState<Position[]>([])
  const [logs, setLogs] = useState<ActivityLog[]>([])
  const [portfolioHistory, setPortfolioHistory] = useState<PortfolioSnapshot[]>([])
  const [tradeHistory, setTradeHistory] = useState<TradeRecord[]>([])
  const [connected, setConnected] = useState(false)
  const [selectedView, setSelectedView] = useState<'portfolio' | 'trades' | 'positions'>('portfolio')
  const [selectedSymbol, setSelectedSymbol] = useState<string>('ALL')

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 3000)
    return () => clearInterval(interval)
  }, [])

  const fetchData = async () => {
    try {
      const [statusRes, positionsRes, logsRes, portfolioRes, tradesRes] = await Promise.all([
        axios.get(`${API_URL}/status`),
        axios.get(`${API_URL}/positions`),
        axios.get(`${API_URL}/logs`),
        axios.get(`${API_URL}/portfolio/history`),
        axios.get(`${API_URL}/trades/history`)
      ])
      setStatus(statusRes.data)
      setPositions(positionsRes.data)
      setLogs(logsRes.data)
      setPortfolioHistory(portfolioRes.data)
      setTradeHistory(tradesRes.data)
      setConnected(true)
    } catch (err) {
      setConnected(false)
    }
  }

  const toggleTrading = async (enabled: boolean) => {
    try {
      await axios.post(`${API_URL}/toggle`, { enabled })
      fetchData()
    } catch (err) {
      console.error(err)
    }
  }

  const generateTestData = async () => {
    try {
      await axios.post(`${API_URL}/test/generate`)
      setTimeout(fetchData, 1000)
    } catch (err) {
      console.error(err)
    }
  }

  const symbols = ['ALL', ...new Set(tradeHistory.map(t => t.symbol))]

  const filteredTrades = selectedSymbol === 'ALL' 
    ? tradeHistory 
    : tradeHistory.filter(t => t.symbol === selectedSymbol)

  // FIXED: Calculate P&L from current positions, not trade history
  const unrealizedPnL = positions.reduce((sum, p) => sum + p.pnl, 0)
  
  // Realized P&L from closed trades only
  const realizedPnL = tradeHistory
    .filter(t => t.action === 'SELL')
    .reduce((sum, t) => sum + t.pnl, 0)
  
  // Total P&L = unrealized + realized
  const totalPnL = unrealizedPnL + realizedPnL

  // FIXED: Win rate only from CLOSED positions (SELL trades)
  const closedTrades = tradeHistory.filter(t => t.action === 'SELL')
  const winningTrades = closedTrades.filter(t => t.pnl > 0).length
  const winRate = closedTrades.length > 0 
    ? ((winningTrades / closedTrades.length) * 100).toFixed(1) 
    : '0'

  // FIXED: Portfolio value = cash + positions value
  const latestSnapshot = portfolioHistory.length > 0 
    ? portfolioHistory[portfolioHistory.length - 1]
    : { total_value: 100000, cash: 100000, positions_value: 0 }
  
  const currentValue = latestSnapshot.total_value
  const currentCash = latestSnapshot.cash
  const currentPositionsValue = latestSnapshot.positions_value

  // Starting value
  const startValue = portfolioHistory.length > 0 
    ? portfolioHistory[0].total_value 
    : 100000

  // FIXED: Total return based on actual portfolio change
  const totalReturn = ((currentValue - startValue) / startValue * 100).toFixed(2)

  const chartData = portfolioHistory.map(p => ({
    time: new Date(p.timestamp).toLocaleTimeString(),
    value: p.total_value,
    cash: p.cash,
    positions: p.positions_value
  }))

  const tradePnLData = filteredTrades
    .filter(t => t.action === 'SELL')
    .map(t => ({
      symbol: t.symbol,
      pnl: t.pnl,
      time: new Date(t.timestamp).toLocaleTimeString()
    }))

  const symbolPnL = tradeHistory
    .filter(t => t.action === 'SELL')
    .reduce((acc, t) => {
      acc[t.symbol] = (acc[t.symbol] || 0) + t.pnl
      return acc
    }, {} as Record<string, number>)

  const symbolPerfData = Object.entries(symbolPnL).map(([symbol, pnl]) => ({
    symbol,
    pnl
  }))

  return (
    <div style={{
      minHeight: '100vh',
      background: 'linear-gradient(135deg, #1e3a8a 0%, #7c3aed 100%)',
      color: '#fff',
      padding: '1.5rem',
      fontFamily: 'system-ui'
    }}>
      <div style={{ maxWidth: '1800px', margin: '0 auto' }}>
        
        <header style={{ textAlign: 'center', marginBottom: '1.5rem' }}>
          <h1 style={{ fontSize: '2.5rem', margin: 0 }}>🐞 LadyBug Trading Dashboard</h1>
          <p style={{ opacity: 0.9 }}>Real-time Portfolio Analytics</p>
        </header>

        {/* ADDED: Trading Mode Selector */}
        <TradingModeSelector onModeChange={(mode) => console.log('Mode changed to:', mode)} />

        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', 
          gap: '1rem', 
          marginBottom: '1.5rem' 
        }}>
          <StatCard 
            title="Portfolio Value" 
            value={`$${currentValue.toFixed(2)}`} 
            subtitle={`Cash: $${currentCash.toFixed(2)} | Holdings: $${currentPositionsValue.toFixed(2)}`}
            color="#10b981" 
          />
          <StatCard 
            title="Total Return" 
            value={`${totalReturn}%`} 
            subtitle={`From $${startValue.toFixed(2)}`}
            color={parseFloat(totalReturn) >= 0 ? '#10b981' : '#ef4444'} 
          />
          <StatCard 
            title="Total P&L" 
            value={`$${totalPnL.toFixed(2)}`} 
            subtitle={`Unrealized: $${unrealizedPnL.toFixed(2)} | Realized: $${realizedPnL.toFixed(2)}`}
            color={totalPnL >= 0 ? '#10b981' : '#ef4444'} 
          />
          <StatCard 
            title="Win Rate" 
            value={`${winRate}%`} 
            subtitle={`${winningTrades} wins / ${closedTrades.length} trades`}
            color={parseFloat(winRate) >= 50 ? '#10b981' : '#ef4444'} 
          />
          <StatCard 
            title="Active Positions" 
            value={positions.length.toString()} 
            subtitle={`Stocks: ${positions.filter(p => p.asset_type === 'stock').length} | Crypto: ${positions.filter(p => p.asset_type === 'crypto').length}`}
            color="#06b6d4" 
          />
          <StatCard 
            title="Status" 
            value={connected ? '🟢 Live' : '🔴 Offline'} 
            subtitle={status?.trading_enabled ? 'Trading Active' : 'Trading Paused'}
            color={connected ? '#10b981' : '#ef4444'} 
          />
        </div>

        <div style={{
          display: 'flex',
          gap: '0.5rem',
          marginBottom: '1.5rem',
          flexWrap: 'wrap'
        }}>
          <button
            onClick={() => toggleTrading(!status?.trading_enabled)}
            style={{
              padding: '0.75rem 1.5rem',
              background: status?.trading_enabled ? '#ef4444' : '#10b981',
              border: 'none',
              borderRadius: '0.5rem',
              color: '#fff',
              cursor: 'pointer',
              fontWeight: 'bold'
            }}
          >
            {status?.trading_enabled ? 'Stop Trading' : 'Start Trading'}
          </button>
          <button
            onClick={generateTestData}
            style={{
              padding: '0.75rem 1.5rem',
              background: '#8b5cf6',
              border: 'none',
              borderRadius: '0.5rem',
              color: '#fff',
              cursor: 'pointer',
              fontWeight: 'bold'
            }}
          >
            🧪 Generate Test Data
          </button>
          <div style={{ marginLeft: 'auto', display: 'flex', gap: '0.5rem' }}>
            <button onClick={() => setSelectedView('portfolio')} style={{
              ...tabStyle,
              background: selectedView === 'portfolio' ? '#7c3aed' : 'rgba(255,255,255,0.1)'
            }}>
              📊 Portfolio History
            </button>
            <button onClick={() => setSelectedView('trades')} style={{
              ...tabStyle,
              background: selectedView === 'trades' ? '#7c3aed' : 'rgba(255,255,255,0.1)'
            }}>
              📈 Trade Analysis
            </button>
            <button onClick={() => setSelectedView('positions')} style={{
              ...tabStyle,
              background: selectedView === 'positions' ? '#7c3aed' : 'rgba(255,255,255,0.1)'
            }}>
              📍 Position Breakdown
            </button>
          </div>
        </div>

        {selectedView === 'portfolio' && (
          <div style={cardStyle}>
            <h2 style={{ margin: '0 0 1rem 0' }}>Portfolio Value Over Time</h2>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
                <XAxis dataKey="time" stroke="#fff" />
                <YAxis stroke="#fff" />
                <Tooltip 
                  contentStyle={{ background: 'rgba(0,0,0,0.8)', border: 'none', borderRadius: '0.5rem' }}
                  labelStyle={{ color: '#fff' }}
                />
                <Legend />
                <Line type="monotone" dataKey="value" stroke="#10b981" name="Total Value" strokeWidth={2} />
                <Line type="monotone" dataKey="cash" stroke="#06b6d4" name="Cash" strokeWidth={2} />
                <Line type="monotone" dataKey="positions" stroke="#f59e0b" name="Positions" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        )}

        {selectedView === 'trades' && (
          <div style={cardStyle}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
              <h2 style={{ margin: 0 }}>P&L by Symbol</h2>
              <select
                value={selectedSymbol}
                onChange={(e) => setSelectedSymbol(e.target.value)}
                style={{
                  padding: '0.5rem',
                  background: 'rgba(255,255,255,0.1)',
                  border: '1px solid rgba(255,255,255,0.2)',
                  borderRadius: '0.5rem',
                  color: '#fff'
                }}
              >
                {symbols.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </div>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={symbolPerfData}>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
                <XAxis dataKey="symbol" stroke="#fff" />
                <YAxis stroke="#fff" />
                <Tooltip
                  contentStyle={{ background: 'rgba(0,0,0,0.8)', border: 'none', borderRadius: '0.5rem' }}
                  labelStyle={{ color: '#fff' }}
                />
                <Legend />
                <Bar dataKey="pnl" fill="#10b981" name="Total P&L">
                  {symbolPerfData.map((entry, index) => (
                    <Bar key={`cell-${index}`} dataKey="pnl" fill={entry.pnl >= 0 ? '#10b981' : '#ef4444'} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}

        {selectedView === 'positions' && (
          <div style={cardStyle}>
            <h2 style={{ margin: '0 0 1rem 0' }}>🔥 Active Positions ({positions.length} total)</h2>
            
            {/* Stock Positions */}
            <h3 style={{ marginTop: '1rem', color: '#10b981' }}>📊 Stocks ({positions.filter(p => p.asset_type === 'stock').length})</h3>
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ borderBottom: '2px solid rgba(255,255,255,0.2)' }}>
                    <th style={tableHeaderStyle}>Symbol</th>
                    <th style={tableHeaderStyle}>Quantity</th>
                    <th style={tableHeaderStyle}>Entry Price</th>
                    <th style={tableHeaderStyle}>Current Price</th>
                    <th style={tableHeaderStyle}>Market Value</th>
                    <th style={tableHeaderStyle}>P&L</th>
                    <th style={tableHeaderStyle}>P&L %</th>
                  </tr>
                </thead>
                <tbody>
                  {positions.filter(p => p.asset_type === 'stock').map((pos, idx) => (
                    <tr key={idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                      <td style={tableCellStyle}><strong>{pos.symbol}</strong></td>
                      <td style={tableCellStyle}>{pos.quantity.toFixed(2)}</td>
                      <td style={tableCellStyle}>${pos.entry_price.toFixed(2)}</td>
                      <td style={tableCellStyle}>${pos.current_price.toFixed(2)}</td>
                      <td style={tableCellStyle}>${pos.market_value.toFixed(2)}</td>
                      <td style={{ ...tableCellStyle, color: pos.pnl >= 0 ? '#10b981' : '#ef4444', fontWeight: 'bold' }}>
                        ${pos.pnl.toFixed(2)}
                      </td>
                      <td style={{ ...tableCellStyle, color: pos.pnl_percent >= 0 ? '#10b981' : '#ef4444' }}>
                        {pos.pnl_percent.toFixed(2)}%
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Crypto Positions */}
            <h3 style={{ marginTop: '2rem', color: '#f59e0b' }}>₿ Crypto ({positions.filter(p => p.asset_type === 'crypto').length})</h3>
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ borderBottom: '2px solid rgba(255,255,255,0.2)' }}>
                    <th style={tableHeaderStyle}>Symbol</th>
                    <th style={tableHeaderStyle}>Quantity</th>
                    <th style={tableHeaderStyle}>Entry Price</th>
                    <th style={tableHeaderStyle}>Current Price</th>
                    <th style={tableHeaderStyle}>Market Value</th>
                    <th style={tableHeaderStyle}>P&L</th>
                    <th style={tableHeaderStyle}>P&L %</th>
                  </tr>
                </thead>
                <tbody>
                  {positions.filter(p => p.asset_type === 'crypto').map((pos, idx) => (
                    <tr key={idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                      <td style={tableCellStyle}><strong>{pos.symbol}</strong></td>
                      <td style={tableCellStyle}>{pos.quantity.toFixed(6)}</td>
                      <td style={tableCellStyle}>${pos.entry_price.toFixed(2)}</td>
                      <td style={tableCellStyle}>${pos.current_price.toFixed(2)}</td>
                      <td style={tableCellStyle}>${pos.market_value.toFixed(2)}</td>
                      <td style={{ ...tableCellStyle, color: pos.pnl >= 0 ? '#10b981' : '#ef4444', fontWeight: 'bold' }}>
                        ${pos.pnl.toFixed(2)}
                      </td>
                      <td style={{ ...tableCellStyle, color: pos.pnl_percent >= 0 ? '#10b981' : '#ef4444' }}>
                        {pos.pnl_percent.toFixed(2)}%
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {positions.length === 0 && (
              <div style={{ textAlign: 'center', padding: '2rem', opacity: 0.6 }}>
                No active positions. Click "Generate Test Data" to create sample positions.
              </div>
            )}
          </div>
        )}

        <div style={cardStyle}>
          <h2 style={{ margin: '0 0 1rem 0' }}>📋 Recent Activity</h2>
          <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
            {logs.slice(-20).reverse().map((log) => (
              <div
                key={log.id}
                style={{
                  padding: '0.75rem',
                  marginBottom: '0.5rem',
                  background: 'rgba(255,255,255,0.05)',
                  borderRadius: '0.5rem',
                  borderLeft: `3px solid ${logColor(log.level)}`
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.875rem', marginBottom: '0.25rem' }}>
                  <span style={{ opacity: 0.7 }}>
                    {new Date(log.timestamp).toLocaleTimeString()}
                  </span>
                  <span style={{ fontWeight: 'bold', color: logColor(log.level) }}>
                    {log.category}
                  </span>
                </div>
                <div>{log.message}</div>
                {log.symbol && <div style={{ fontSize: '0.875rem', opacity: 0.7, marginTop: '0.25rem' }}>Symbol: {log.symbol}</div>}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

function StatCard({ title, value, subtitle, color }: { title: string; value: string; subtitle?: string; color: string }) {
  return (
    <div style={{
      background: 'rgba(255,255,255,0.1)',
      backdropFilter: 'blur(10px)',
      borderRadius: '1rem',
      padding: '1.25rem',
      border: '1px solid rgba(255,255,255,0.2)'
    }}>
      <div style={{ fontSize: '0.875rem', opacity: 0.8, marginBottom: '0.5rem' }}>{title}</div>
      <div style={{ fontSize: '1.75rem', fontWeight: 'bold', color }}>{value}</div>
      {subtitle && <div style={{ fontSize: '0.75rem', opacity: 0.6, marginTop: '0.5rem' }}>{subtitle}</div>}
    </div>
  )
}

const cardStyle = {
  background: 'rgba(255,255,255,0.1)',
  backdropFilter: 'blur(10px)',
  borderRadius: '1rem',
  padding: '1.5rem',
  marginBottom: '1.5rem',
  border: '1px solid rgba(255,255,255,0.2)'
}

const tabStyle = {
  padding: '0.75rem 1.5rem',
  border: 'none',
  borderRadius: '0.5rem',
  color: '#fff',
  cursor: 'pointer',
  fontWeight: 'bold',
  transition: 'all 0.2s'
}

const tableHeaderStyle = {
  padding: '0.75rem',
  textAlign: 'left' as const,
  fontWeight: 'bold',
  opacity: 0.8
}

const tableCellStyle = {
  padding: '0.75rem',
  textAlign: 'left' as const
}

function logColor(level: string) {
  switch (level) {
    case 'success': return '#10b981'
    case 'error': return '#ef4444'
    case 'warning': return '#f59e0b'
    default: return '#06b6d4'
  }
}
