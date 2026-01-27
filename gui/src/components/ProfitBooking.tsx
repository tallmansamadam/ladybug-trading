import React, { useState } from 'react'
import axios from 'axios'

const API_URL = 'http://localhost:8080'

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

interface ProfitBookingProps {
  positions: Position[]
  onRefresh: () => void
}

export function ProfitBooking({ positions, onRefresh }: ProfitBookingProps) {
  const [booking, setBooking] = useState<string | null>(null)
  const [expanded, setExpanded] = useState(false)
  const [filter, setFilter] = useState<'all' | 'profitable' | 'losing'>('profitable')

  const bookProfit = async (symbol: string) => {
    if (!confirm(`Book profit for ${symbol}?`)) return
    
    setBooking(symbol)
    try {
      await axios.post(`${API_URL}/book-profit/${symbol}`)
      alert(`✅ Profit booked for ${symbol}!`)
      onRefresh()
    } catch (err) {
      alert(`❌ Failed to book profit for ${symbol}`)
    } finally {
      setBooking(null)
    }
  }

  const bookAllProfits = async () => {
    const profitableCount = positions.filter(p => p.pnl > 0).length
    if (!confirm(`Book profits for ALL ${profitableCount} profitable positions?`)) return
    
    setBooking('all')
    try {
      const response = await axios.post(`${API_URL}/book-all-profits`)
      const data = response.data
      alert(`✅ Closed ${data.closed_count} positions!\n${data.failed_count > 0 ? `⚠️ ${data.failed_count} failed` : ''}`)
      onRefresh()
    } catch (err) {
      alert('❌ Failed to book profits')
    } finally {
      setBooking(null)
    }
  }

  const filteredPositions = positions.filter(p => {
    if (filter === 'profitable') return p.pnl > 0
    if (filter === 'losing') return p.pnl < 0
    return true
  })

  const profitablePositions = positions.filter(p => p.pnl > 0)
  const totalUnrealizedProfit = profitablePositions.reduce((sum, p) => sum + p.pnl, 0)

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
        marginBottom: '1rem',
        cursor: 'pointer'
      }}
        onClick={() => setExpanded(!expanded)}
      >
        <h2 style={{ margin: 0 }}>
          💰 Profit Booking 
          <span style={{ 
            fontSize: '0.875rem', 
            opacity: 0.7, 
            marginLeft: '0.5rem',
            fontWeight: 'normal' 
          }}>
            {expanded ? '▼' : '▶'} Click to {expanded ? 'collapse' : 'expand'}
          </span>
        </h2>
        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
          <div style={{ textAlign: 'right' }}>
            <div style={{ fontSize: '0.875rem', opacity: 0.7 }}>Unrealized Profit</div>
            <div style={{ 
              fontSize: '1.5rem', 
              fontWeight: 'bold', 
              color: totalUnrealizedProfit >= 0 ? '#10b981' : '#ef4444' 
            }}>
              ${totalUnrealizedProfit.toFixed(2)}
            </div>
            <div style={{ fontSize: '0.75rem', opacity: 0.6 }}>
              {profitablePositions.length} profitable positions
            </div>
          </div>
          <button
            onClick={(e) => {
              e.stopPropagation()
              bookAllProfits()
            }}
            disabled={booking !== null || profitablePositions.length === 0}
            style={{
              padding: '0.75rem 1.5rem',
              background: profitablePositions.length > 0 ? '#10b981' : '#555',
              border: 'none',
              borderRadius: '0.5rem',
              color: '#fff',
              cursor: profitablePositions.length > 0 ? 'pointer' : 'not-allowed',
              fontWeight: 'bold',
              fontSize: '1rem',
              transition: 'all 0.2s',
              opacity: booking === 'all' ? 0.5 : 1
            }}
          >
            {booking === 'all' ? '⏳ Booking...' : '💰 Book ALL Profits'}
          </button>
        </div>
      </div>

      {expanded && (
        <>
          <div style={{ 
            display: 'flex', 
            gap: '0.5rem', 
            marginBottom: '1rem',
            padding: '0.75rem',
            background: 'rgba(0,0,0,0.2)',
            borderRadius: '0.5rem'
          }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
              <input 
                type="radio" 
                name="profit-filter" 
                checked={filter === 'all'}
                onChange={() => setFilter('all')}
                style={{ cursor: 'pointer' }}
              />
              <span>All ({positions.length})</span>
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
              <input 
                type="radio" 
                name="profit-filter" 
                checked={filter === 'profitable'}
                onChange={() => setFilter('profitable')}
                style={{ cursor: 'pointer' }}
              />
              <span style={{ color: '#10b981' }}>Profitable ({profitablePositions.length})</span>
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
              <input 
                type="radio" 
                name="profit-filter" 
                checked={filter === 'losing'}
                onChange={() => setFilter('losing')}
                style={{ cursor: 'pointer' }}
              />
              <span style={{ color: '#ef4444' }}>Losing ({positions.filter(p => p.pnl < 0).length})</span>
            </label>
          </div>

          <div style={{ overflowX: 'auto', maxHeight: '400px', overflowY: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead style={{ position: 'sticky', top: 0, background: 'rgba(30, 58, 138, 0.95)', zIndex: 1 }}>
                <tr style={{ borderBottom: '2px solid rgba(255,255,255,0.2)' }}>
                  <th style={tableHeaderStyle}>Symbol</th>
                  <th style={tableHeaderStyle}>Type</th>
                  <th style={tableHeaderStyle}>Quantity</th>
                  <th style={tableHeaderStyle}>Entry</th>
                  <th style={tableHeaderStyle}>Current</th>
                  <th style={tableHeaderStyle}>P&L</th>
                  <th style={tableHeaderStyle}>P&L %</th>
                  <th style={tableHeaderStyle}>Action</th>
                </tr>
              </thead>
              <tbody>
                {filteredPositions.map((pos, idx) => (
                  <tr key={idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                    <td style={tableCellStyle}><strong>{pos.symbol}</strong></td>
                    <td style={tableCellStyle}>
                      <span style={{ 
                        padding: '0.25rem 0.5rem', 
                        borderRadius: '0.25rem',
                        background: pos.asset_type === 'crypto' ? 'rgba(245, 158, 11, 0.2)' : 'rgba(16, 185, 129, 0.2)',
                        color: pos.asset_type === 'crypto' ? '#f59e0b' : '#10b981',
                        fontSize: '0.75rem',
                        fontWeight: 'bold'
                      }}>
                        {pos.asset_type === 'crypto' ? '₿ CRYPTO' : '📊 STOCK'}
                      </span>
                    </td>
                    <td style={tableCellStyle}>{pos.quantity.toFixed(pos.asset_type === 'crypto' ? 6 : 2)}</td>
                    <td style={tableCellStyle}>${pos.entry_price.toFixed(2)}</td>
                    <td style={tableCellStyle}>${pos.current_price.toFixed(2)}</td>
                    <td style={{ 
                      ...tableCellStyle, 
                      color: pos.pnl >= 0 ? '#10b981' : '#ef4444',
                      fontWeight: 'bold'
                    }}>
                      ${pos.pnl.toFixed(2)}
                    </td>
                    <td style={{ 
                      ...tableCellStyle, 
                      color: pos.pnl_percent >= 0 ? '#10b981' : '#ef4444' 
                    }}>
                      {pos.pnl_percent >= 0 ? '+' : ''}{pos.pnl_percent.toFixed(2)}%
                    </td>
                    <td style={tableCellStyle}>
                      {pos.pnl > 0 ? (
                        <button
                          onClick={() => bookProfit(pos.symbol)}
                          disabled={booking !== null}
                          style={{
                            padding: '0.5rem 1rem',
                            background: '#10b981',
                            border: 'none',
                            borderRadius: '0.25rem',
                            color: '#fff',
                            cursor: booking === null ? 'pointer' : 'not-allowed',
                            fontSize: '0.875rem',
                            fontWeight: 'bold',
                            opacity: booking === pos.symbol ? 0.5 : 1
                          }}
                        >
                          {booking === pos.symbol ? '⏳' : '💰 Book'}
                        </button>
                      ) : (
                        <span style={{ opacity: 0.5, fontSize: '0.875rem' }}>-</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {filteredPositions.length === 0 && (
            <div style={{ 
              textAlign: 'center', 
              padding: '2rem', 
              opacity: 0.6 
            }}>
              No {filter === 'profitable' ? 'profitable' : filter === 'losing' ? 'losing' : ''} positions
            </div>
          )}
        </>
      )}
    </div>
  )
}

const tableHeaderStyle = {
  padding: '0.75rem',
  textAlign: 'left' as const,
  fontWeight: 'bold',
  opacity: 0.9,
  fontSize: '0.875rem'
}

const tableCellStyle = {
  padding: '0.75rem',
  textAlign: 'left' as const,
  fontSize: '0.875rem'
}
