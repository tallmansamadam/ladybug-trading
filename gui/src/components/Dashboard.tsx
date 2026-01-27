import { useState, useEffect } from 'react'
import axios from 'axios'
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'

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

  const totalPnL = tradeHistory.reduce((sum, t) => sum + t.pnl, 0)
  const winningTrades = tradeHistory.filter(t => t.pnl > 0).length
  const totalTrades = tradeHistory.filter(t => t.action === 'SELL').length
  const winRate = totalTrades > 0 ? ((winningTrades / totalTrades) * 100).toFixed(1) : '0'

  const currentValue = portfolioHistory.length > 0 
    ? portfolioHistory[portfolioHistory.length - 1].total_value 
    : 100000

  const startValue = portfolioHistory.length > 0 
    ? portfolioHistory[0].total_value 
    : 100000

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

        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', 
          gap: '1rem', 
          marginBottom: '1.5rem' 
        }}>
          <StatCard title="Portfolio Value" value={`$${currentValue.toFixed(2)}`} color="#10b981" />
          <StatCard title="Total Return" value={`${totalReturn}%`} color={parseFloat(totalReturn) >= 0 ? '#10b981' : '#ef4444'} />
          <StatCard title="Total P&L" value={`$${totalPnL.toFixed(2)}`} color={totalPnL >= 0 ? '#10b981' : '#ef4444'} />
          <StatCard title="Win Rate" value={`${winRate}%`} color="#8b5cf6" />
          <StatCard title="Active Positions" value={status?.active_positions.toString() || '0'} color="#06b6d4" />
          <StatCard title="Status" value={connected ? '🟢 Live' : '🔴 Offline'} color={connected ? '#10b981' : '#ef4444'} />
        </div>

        <div style={{
          background: 'rgba(255,255,255,0.1)',
          backdropFilter: 'blur(10px)',
          borderRadius: '1rem',
          padding: '1rem',
          marginBottom: '1.5rem',
          display: 'flex',
          gap: '1rem',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <Button onClick={() => toggleTrading(true)} disabled={status?.trading_enabled} color="#10b981">
              Start Trading
            </Button>
            <Button onClick={() => toggleTrading(false)} disabled={!status?.trading_enabled} color="#ef4444">
              Stop Trading
            </Button>
            <Button onClick={generateTestData} color="#8b5cf6">
              🧪 Generate Test Data
            </Button>
          </div>
          <div style={{ fontSize: '0.875rem', opacity: 0.9 }}>
            {status?.trading_enabled ? '🟢 Trading Active' : '🟡 Trading Paused'}
          </div>
        </div>

        <div style={{
          background: 'rgba(255,255,255,0.1)',
          backdropFilter: 'blur(10px)',
          borderRadius: '1rem',
          padding: '1rem',
          marginBottom: '1.5rem',
          display: 'flex',
          gap: '0.5rem'
        }}>
          <ViewButton 
            active={selectedView === 'portfolio'} 
            onClick={() => setSelectedView('portfolio')}
          >
            📈 Portfolio History
          </ViewButton>
          <ViewButton 
            active={selectedView === 'trades'} 
            onClick={() => setSelectedView('trades')}
          >
            💰 Trade Analysis
          </ViewButton>
          <ViewButton 
            active={selectedView === 'positions'} 
            onClick={() => setSelectedView('positions')}
          >
            📊 Position Breakdown
          </ViewButton>
        </div>

        <div style={{
          background: 'rgba(255,255,255,0.1)',
          backdropFilter: 'blur(10px)',
          borderRadius: '1rem',
          padding: '1.5rem',
          marginBottom: '1.5rem'
        }}>
          {selectedView === 'portfolio' && (
            <>
              <h2 style={{ margin: '0 0 1rem 0' }}>Portfolio Value Over Time</h2>
              {chartData.length > 0 ? (
                <ResponsiveContainer width="100%" height={400}>
                  <LineChart data={chartData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
                    <XAxis dataKey="time" stroke="#fff" />
                    <YAxis stroke="#fff" />
                    <Tooltip 
                      contentStyle={{ background: 'rgba(0,0,0,0.8)', border: 'none', borderRadius: '0.5rem' }}
                      labelStyle={{ color: '#fff' }}
                    />
                    <Legend />
                    <Line type="monotone" dataKey="value" stroke="#10b981" strokeWidth={3} name="Total Value" />
                    <Line type="monotone" dataKey="cash" stroke="#60a5fa" strokeWidth={2} name="Cash" />
                    <Line type="monotone" dataKey="positions" stroke="#f59e0b" strokeWidth={2} name="Positions" />
                  </LineChart>
                </ResponsiveContainer>
              ) : (
                <EmptyChart />
              )}
            </>
          )}

          {selectedView === 'trades' && (
            <>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <h2 style={{ margin: 0 }}>Trade P&L Analysis</h2>
                <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                  {symbols.map(symbol => (
                    <SymbolButton
                      key={symbol}
                      active={selectedSymbol === symbol}
                      onClick={() => setSelectedSymbol(symbol)}
                    >
                      {symbol}
                    </SymbolButton>
                  ))}
                </div>
              </div>
              {tradePnLData.length > 0 ? (
                <ResponsiveContainer width="100%" height={400}>
                  <BarChart data={tradePnLData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
                    <XAxis dataKey="time" stroke="#fff" />
                    <YAxis stroke="#fff" />
                    <Tooltip 
                      contentStyle={{ background: 'rgba(0,0,0,0.8)', border: 'none', borderRadius: '0.5rem' }}
                      labelStyle={{ color: '#fff' }}
                    />
                    <Legend />
                    <Bar dataKey="pnl" fill="#8b5cf6" name="Profit/Loss" />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <EmptyChart />
              )}
            </>
          )}

          {selectedView === 'positions' && (
            <>
              <h2 style={{ margin: '0 0 1rem 0' }}>Performance by Symbol</h2>
              {symbolPerfData.length > 0 ? (
                <ResponsiveContainer width="100%" height={400}>
                  <BarChart data={symbolPerfData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
                    <XAxis dataKey="symbol" stroke="#fff" />
                    <YAxis stroke="#fff" />
                    <Tooltip 
                      contentStyle={{ background: 'rgba(0,0,0,0.8)', border: 'none', borderRadius: '0.5rem' }}
                      labelStyle={{ color: '#fff' }}
                    />
                    <Legend />
                    <Bar dataKey="pnl" name="Total P&L" fill="#8884d8">
                      {symbolPerfData.map((entry, index) => (
                        <Bar key={`cell-${index}`} dataKey="pnl" fill={entry.pnl >= 0 ? '#10b981' : '#ef4444'} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <EmptyChart />
              )}
            </>
          )}
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>
          
          <Panel title="💼 Active Positions">
            {positions.length === 0 ? (
              <EmptyState>No active positions</EmptyState>
            ) : (
              positions.map((pos, i) => (
                <div key={i} style={{
                  padding: '1rem',
                  background: 'rgba(0,0,0,0.2)',
                  borderRadius: '0.5rem',
                  marginBottom: '0.75rem',
                  borderLeft: `4px solid ${pos.pnl >= 0 ? '#10b981' : '#ef4444'}`
                }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <div style={{ fontWeight: 'bold', fontSize: '1.25rem' }}>{pos.symbol}</div>
                    <div style={{ fontSize: '1.125rem' }}>${pos.current_price.toFixed(2)}</div>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.875rem', opacity: 0.8, marginTop: '0.5rem' }}>
                    <span>{pos.quantity} @ ${pos.entry_price.toFixed(2)}</span>
                    <span style={{ fontWeight: 'bold', color: pos.pnl >= 0 ? '#10b981' : '#ef4444' }}>
                      {pos.pnl >= 0 ? '+' : ''}${pos.pnl.toFixed(2)}
                    </span>
                  </div>
                </div>
              ))
            )}
          </Panel>

          <Panel title="📜 Recent Activity">
            <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
              {logs.length === 0 ? (
                <EmptyState>No activity yet</EmptyState>
              ) : (
                logs.slice(0, 10).map(log => (
                  <div key={log.id} style={{
                    padding: '0.75rem',
                    background: 'rgba(0,0,0,0.2)',
                    borderRadius: '0.5rem',
                    marginBottom: '0.5rem',
                    fontSize: '0.875rem'
                  }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.25rem' }}>
                      <span style={{ fontWeight: 'bold' }}>{log.symbol || log.category}</span>
                      <span style={{ opacity: 0.6 }}>{new Date(log.timestamp).toLocaleTimeString()}</span>
                    </div>
                    <div>{log.message}</div>
                  </div>
                ))
              )}
            </div>
          </Panel>
        </div>
      </div>
    </div>
  )
}

function StatCard({ title, value, color }: any) {
  return (
    <div style={{
      background: 'rgba(255,255,255,0.1)',
      backdropFilter: 'blur(10px)',
      borderRadius: '0.75rem',
      padding: '1rem',
      borderLeft: `4px solid ${color}`
    }}>
      <div style={{ fontSize: '0.75rem', opacity: 0.7, marginBottom: '0.25rem' }}>{title}</div>
      <div style={{ fontSize: '1.5rem', fontWeight: 'bold' }}>{value}</div>
    </div>
  )
}

function Button({ onClick, disabled, color, children }: any) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        background: disabled ? '#6b7280' : color,
        border: 'none',
        padding: '0.75rem 1.5rem',
        borderRadius: '0.5rem',
        color: 'white',
        fontWeight: 'bold',
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.5 : 1
      }}
    >
      {children}
    </button>
  )
}

function ViewButton({ active, onClick, children }: any) {
  return (
    <button
      onClick={onClick}
      style={{
        background: active ? 'rgba(255,255,255,0.3)' : 'rgba(255,255,255,0.1)',
        border: 'none',
        padding: '0.75rem 1.5rem',
        borderRadius: '0.5rem',
        color: 'white',
        fontWeight: 'bold',
        cursor: 'pointer',
        transition: 'all 0.2s'
      }}
    >
      {children}
    </button>
  )
}

function SymbolButton({ active, onClick, children }: any) {
  return (
    <button
      onClick={onClick}
      style={{
        background: active ? '#8b5cf6' : 'rgba(255,255,255,0.1)',
        border: 'none',
        padding: '0.5rem 1rem',
        borderRadius: '0.375rem',
        color: 'white',
        fontSize: '0.875rem',
        fontWeight: 'bold',
        cursor: 'pointer'
      }}
    >
      {children}
    </button>
  )
}

function Panel({ title, children }: any) {
  return (
    <div style={{
      background: 'rgba(255,255,255,0.1)',
      backdropFilter: 'blur(10px)',
      borderRadius: '1rem',
      padding: '1.5rem'
    }}>
      <h2 style={{ margin: '0 0 1rem 0', fontSize: '1.25rem' }}>{title}</h2>
      {children}
    </div>
  )
}

function EmptyState({ children }: any) {
  return (
    <div style={{ textAlign: 'center', padding: '3rem', opacity: 0.6 }}>
      {children}
    </div>
  )
}

function EmptyChart() {
  return (
    <div style={{ 
      height: '400px', 
      display: 'flex', 
      alignItems: 'center', 
      justifyContent: 'center',
      opacity: 0.6 
    }}>
      No data available. Click "Generate Test Data" to see charts.
    </div>
  )
}