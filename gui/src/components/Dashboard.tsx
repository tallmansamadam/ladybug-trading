import { useState, useEffect } from 'react'
import axios from 'axios'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'
import { TradingModeSelector } from './TradingModeSelector'
import { ProfitBooking } from './ProfitBooking'
import { EnhancedTradeAnalysis } from './EnhancedTradeAnalysis'

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
  
  // Portfolio chart controls
  const [showTotalValue, setShowTotalValue] = useState(true)
  const [showCash, setShowCash] = useState(true)
  const [showPositions, setShowPositions] = useState(true)
  const [chartZoom, setChartZoom] = useState(100)  // percentage

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

  // CORRECT CALCULATIONS
  const unrealizedPnL = positions.reduce((sum, p) => sum + p.pnl, 0)
  const realizedPnL = tradeHistory
    .filter(t => t.action === 'SELL')
    .reduce((sum, t) => sum + t.pnl, 0)
  const totalPnL = unrealizedPnL + realizedPnL

  const closedTrades = tradeHistory.filter(t => t.action === 'SELL')
  const winningTrades = closedTrades.filter(t => t.pnl > 0).length
  const winRate = closedTrades.length > 0 
    ? ((winningTrades / closedTrades.length) * 100).toFixed(1) 
    : '0'

  const latestSnapshot = portfolioHistory.length > 0 
    ? portfolioHistory[portfolioHistory.length - 1]
    : { total_value: 100000, cash: 100000, positions_value: 0 }
  
  const currentValue = latestSnapshot.total_value
  const currentCash = latestSnapshot.cash
  const currentPositionsValue = latestSnapshot.positions_value

  const startValue = portfolioHistory.length > 0 
    ? portfolioHistory[0].total_value 
    : 100000

  const totalReturn = ((currentValue - startValue) / startValue * 100).toFixed(2)

  // Apply zoom to chart data
  const totalPoints = portfolioHistory.length
  const pointsToShow = Math.max(10, Math.floor(totalPoints * (chartZoom / 100)))
  const startIndex = Math.max(0, totalPoints - pointsToShow)
  const zoomedHistory = portfolioHistory.slice(startIndex)

  const chartData = zoomedHistory.map(p => ({
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

  // Split positions
  const stockPositions = positions.filter(p => p.asset_type === 'stock')
  const cryptoPositions = positions.filter(p => p.asset_type === 'crypto')

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
            subtitle={`${winningTrades} wins / ${closedTrades.length} closed`}
            color={parseFloat(winRate) >= 50 ? '#10b981' : '#ef4444'} 
          />
          <StatCard 
            title="Active Positions" 
            value={positions.length.toString()} 
            subtitle={`Stocks: ${stockPositions.length} | Crypto: ${cryptoPositions.length}`}
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
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem', flexWrap: 'wrap', gap: '1rem' }}>
              <h2 style={{ margin: 0 }}>Portfolio Value Over Time</h2>
              
              <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', flexWrap: 'wrap' }}>
                {/* Line toggles */}
                <div style={{ display: 'flex', gap: '0.75rem', background: 'rgba(0,0,0,0.2)', padding: '0.5rem 1rem', borderRadius: '0.5rem' }}>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                    <input 
                      type="checkbox" 
                      checked={showTotalValue} 
                      onChange={(e) => setShowTotalValue(e.target.checked)}
                      style={{ cursor: 'pointer' }}
                    />
                    <span style={{ color: '#10b981' }}>● Total Value</span>
                  </label>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                    <input 
                      type="checkbox" 
                      checked={showCash} 
                      onChange={(e) => setShowCash(e.target.checked)}
                      style={{ cursor: 'pointer' }}
                    />
                    <span style={{ color: '#06b6d4' }}>● Cash</span>
                  </label>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                    <input 
                      type="checkbox" 
                      checked={showPositions} 
                      onChange={(e) => setShowPositions(e.target.checked)}
                      style={{ cursor: 'pointer' }}
                    />
                    <span style={{ color: '#f59e0b' }}>● Holdings</span>
                  </label>
                </div>

                {/* Zoom controls */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', background: 'rgba(0,0,0,0.2)', padding: '0.5rem 1rem', borderRadius: '0.5rem' }}>
                  <span style={{ fontSize: '0.875rem', opacity: 0.8 }}>Zoom:</span>
                  <button 
                    onClick={() => setChartZoom(Math.min(200, chartZoom + 25))}
                    style={{ ...zoomButtonStyle }}
                  >
                    🔍+
                  </button>
                  <span style={{ fontSize: '0.875rem', fontWeight: 'bold', minWidth: '50px', textAlign: 'center' }}>
                    {chartZoom}%
                  </span>
                  <button 
                    onClick={() => setChartZoom(Math.max(25, chartZoom - 25))}
                    style={{ ...zoomButtonStyle }}
                  >
                    🔍-
                  </button>
                  <button 
                    onClick={() => setChartZoom(100)}
                    style={{ ...zoomButtonStyle, padding: '0.25rem 0.75rem' }}
                  >
                    Reset
                  </button>
                </div>
              </div>
            </div>

            <ResponsiveContainer width="100%" height={350}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
                <XAxis dataKey="time" stroke="#fff" />
                <YAxis stroke="#fff" />
                <Tooltip 
                  contentStyle={{ background: 'rgba(0,0,0,0.8)', border: 'none', borderRadius: '0.5rem' }}
                  labelStyle={{ color: '#fff' }}
                />
                <Legend />
                {showTotalValue && <Line type="monotone" dataKey="value" stroke="#10b981" name="Total Value" strokeWidth={2} />}
                {showCash && <Line type="monotone" dataKey="cash" stroke="#06b6d4" name="Cash" strokeWidth={2} />}
                {showPositions && <Line type="monotone" dataKey="positions" stroke="#f59e0b" name="Holdings" strokeWidth={2} />}
              </LineChart>
            </ResponsiveContainer>
            <div style={{ fontSize: '0.75rem', opacity: 0.6, marginTop: '0.5rem', textAlign: 'center' }}>
              Showing {pointsToShow} of {totalPoints} data points
            </div>
          </div>
        )}

        {/* Profit Booking Module - Always visible */}
        <ProfitBooking positions={positions} onRefresh={fetchData} />

        {/* Enhanced Trade Analysis */}
        {selectedView === 'trades' && (
          <EnhancedTradeAnalysis tradeHistory={tradeHistory} />
        )}

        {selectedView === 'positions' && (
          <div style={cardStyle}>
            <h2 style={{ margin: '0 0 1.5rem 0' }}>🔥 Active Positions ({positions.length} total)</h2>
            
            {/* Stock Positions */}
            <h3 style={{ marginTop: '1rem', color: '#10b981', borderBottom: '2px solid #10b981', paddingBottom: '0.5rem' }}>
              📊 Stocks ({stockPositions.length})
            </h3>
            {stockPositions.length > 0 ? (
              <div style={{ overflowX: 'auto', marginBottom: '2rem' }}>
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
                    {stockPositions.map((pos, idx) => (
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
            ) : (
              <div style={{ textAlign: 'center', padding: '2rem', opacity: 0.6 }}>
                No stock positions. Trading will create positions when signals trigger.
              </div>
            )}

            {/* Crypto Positions */}
            <h3 style={{ marginTop: '2rem', color: '#f59e0b', borderBottom: '2px solid #f59e0b', paddingBottom: '0.5rem' }}>
              ₿ Crypto ({cryptoPositions.length})
            </h3>
            {cryptoPositions.length > 0 ? (
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
                    {cryptoPositions.map((pos, idx) => (
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
            ) : (
              <div style={{ textAlign: 'center', padding: '2rem', opacity: 0.6 }}>
                No crypto positions. Trading will create positions when signals trigger.
              </div>
            )}

            {positions.length === 0 && (
              <div style={{ textAlign: 'center', padding: '3rem', opacity: 0.6, fontSize: '1.1rem' }}>
                📭 No active positions yet.<br/>
                <span style={{ fontSize: '0.9rem' }}>Click "Generate Test Data" to create sample positions or wait for trading signals.</span>
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

        <style>{`
          .symbol-dropdown option:hover {
            background-color: #7c3aed !important;
          }
          .symbol-dropdown:hover {
            border-color: #10b981;
          }
        `}</style>
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

const zoomButtonStyle = {
  padding: '0.25rem 0.5rem',
  background: 'rgba(255,255,255,0.2)',
  border: '1px solid rgba(255,255,255,0.3)',
  borderRadius: '0.25rem',
  color: '#fff',
  cursor: 'pointer',
  fontSize: '0.875rem',
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
