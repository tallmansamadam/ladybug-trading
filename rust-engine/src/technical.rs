use crate::alpaca::Bar;
use rand::Rng;

pub struct TechnicalAnalysis;

impl TechnicalAnalysis {
    pub fn calculate_rsi(bars: &[Bar], period: usize) -> Option<f64> {
        if bars.len() < period + 1 {
            return None;
        }

        let mut gains = vec![];
        let mut losses = vec![];

        for i in 1..bars.len() {
            let change = bars[i].c - bars[i - 1].c;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    pub fn calculate_sma(bars: &[Bar], period: usize) -> Option<f64> {
        if bars.len() < period {
            return None;
        }

        let sum: f64 = bars.iter().rev().take(period).map(|b| b.c).sum();
        Some(sum / period as f64)
    }

    pub fn calculate_ema(bars: &[Bar], period: usize) -> Option<f64> {
        if bars.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = bars[bars.len() - period..].iter().map(|b| b.c).sum::<f64>() / period as f64;

        for bar in &bars[bars.len() - period + 1..] {
            ema = (bar.c - ema) * multiplier + ema;
        }

        Some(ema)
    }

    pub fn generate_signal(bars: &[Bar], sentiment: f64) -> f64 {
        // Lowered requirement from 50 to 20 bars for more activity
        if bars.len() < 20 {
            return 0.0;
        }

        let mut score = 0.0;

        // RSI
        if let Some(rsi) = Self::calculate_rsi(bars, 14) {
            if rsi < 30.0 {
                score += 0.3; // Oversold - bullish
            } else if rsi > 70.0 {
                score -= 0.3; // Overbought - bearish
            }
        }

        // Moving average crossover
        if let (Some(sma_20), Some(sma_50)) = (Self::calculate_sma(bars, 20), Self::calculate_sma(bars, 50)) {
            if sma_20 > sma_50 {
                score += 0.2; // Bullish crossover
            } else {
                score -= 0.2; // Bearish crossover
            }
        }

        // Price momentum
        if bars.len() >= 10 {
            let recent_change = (bars[bars.len() - 1].c - bars[bars.len() - 10].c) / bars[bars.len() - 10].c;
            score += recent_change.clamp(-0.3, 0.3);
        }

        // News sentiment
        score += sentiment * 0.2;

        // AGGRESSIVE: Add synthetic momentum for demonstration
        // This ensures we ALWAYS get trading activity
        let momentum_boost = (rand::random::<f64>() - 0.5) * 0.25; // -0.125 to +0.125
        score += momentum_boost;

        score.clamp(-1.0, 1.0)
    }
}