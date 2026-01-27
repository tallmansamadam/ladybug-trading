# LadyBug Components - Quick Integration Guide

## ✅ Files Already Installed

The components are already in your project:
- `gui/src/components/PositionsPnLChart.tsx` ✅
- `gui/src/components/EnhancedChart.tsx` ✅

## 🚀 Quick Start

### 1. Install Dependencies (if not already installed)
```powershell
cd C:\Users\frank\Documents\scripts\trading-scripts\ladybug\gui
npm install recharts
```

### 2. Import in Your Dashboard

Edit `gui/src/components/Dashboard.tsx`:

```typescript
import { PositionsPnLChart } from './PositionsPnLChart';
import { EnhancedChart } from './EnhancedChart';

// In your Dashboard component:
function Dashboard() {
  const [positions, setPositions] = useState([]);
  const [portfolioHistory, setPortfolioHistory] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch positions
        const posRes = await fetch('http://localhost:8080/positions');
        const posData = await posRes.json();
        setPositions(posData);

        // Fetch portfolio history
        const histRes = await fetch('http://localhost:8080/portfolio/history');
        const histData = await histRes.json();
        setPortfolioHistory(histData);
      } catch (error) {
        console.error('Error fetching data:', error);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 5000); // Refresh every 5 seconds

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="dashboard">
      <h1>LadyBug Trading Dashboard</h1>
      
      {/* P&L Bar Chart */}
      <PositionsPnLChart positions={positions} />
      
      {/* Portfolio Line Chart */}
      <EnhancedChart 
        portfolioHistory={portfolioHistory}
        positions={positions}
      />
    </div>
  );
}
```

### 3. Start the Backend

```powershell
cd ..\rust-engine
cargo run
```

### 4. Start the Frontend

```powershell
cd ..\gui
npm start
```

### 5. Generate Test Data (Optional)

```powershell
Invoke-RestMethod -Uri "http://localhost:8080/test/generate" -Method Post
```

## 📊 Component Details

### PositionsPnLChart
- Shows vertical bars for each position
- Green bars = profit
- Red bars = loss
- Displays summary stats at top
- Sorted by performance

### EnhancedChart
- 4 toggleable lines:
  - Total Holdings (blue)
  - Stocks (green)
  - Crypto (orange)
  - Cash (red, dashed)
- Individual checkboxes to show/hide lines
- Interactive tooltips

## ✅ Verification

Check files exist:
```powershell
ls C:\Users\frank\Documents\scripts\trading-scripts\ladybug\gui\src\components\*.tsx
```

You should see:
- Dashboard.tsx
- EnhancedChart.tsx ✅
- PositionsPnLChart.tsx ✅

## 🎉 You're Ready!

The components are already installed and ready to use. Just:
1. Install recharts (`npm install recharts`)
2. Import the components in your Dashboard
3. Start the backend and frontend
4. Enjoy your beautiful trading visualizations!
