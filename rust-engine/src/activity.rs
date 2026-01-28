use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::sync::Arc;
use dashmap::DashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: String,
    pub timestamp: String,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

pub struct ActivityLogger {
    logs: Arc<DashMap<String, ActivityLog>>,
    max_logs: usize,
}

impl ActivityLogger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(DashMap::new()),
            max_logs: 100,
        }
    }

    pub fn log(&self, level: LogLevel, category: &str, message: &str, symbol: Option<&str>) {
        let id = uuid::Uuid::new_v4().to_string();
        let log = ActivityLog {
            id: id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            level,
            category: category.to_string(),
            message: message.to_string(),
            symbol: symbol.map(|s| s.to_string()),
        };

        self.logs.insert(id, log);

        // Keep only the most recent logs
        if self.logs.len() > self.max_logs {
            if let Some(oldest) = self.logs.iter().min_by_key(|entry| entry.value().timestamp.clone()) {
                let key = oldest.key().clone();
                drop(oldest);
                self.logs.remove(&key);
            }
        }
    }

    pub fn get_logs(&self) -> Vec<ActivityLog> {
        let mut logs: Vec<ActivityLog> = self.logs.iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        logs
    }

    pub fn info(&self, category: &str, message: &str) {
        self.log(LogLevel::Info, category, message, None);
    }

    pub fn success(&self, category: &str, message: &str) {
        self.log(LogLevel::Success, category, message, None);
    }

    pub fn warning(&self, category: &str, message: &str) {
        self.log(LogLevel::Warning, category, message, None);
    }

    #[allow(dead_code)]
    pub fn error(&self, category: &str, message: &str) {
        self.log(LogLevel::Error, category, message, None);
    }

    pub fn trade(&self, level: LogLevel, message: &str, symbol: &str) {
        self.log(level, "Trade", message, Some(symbol));
    }

    pub fn signal(&self, message: &str, symbol: &str) {
        self.log(LogLevel::Info, "Signal", message, Some(symbol));
    }

    pub fn analysis(&self, message: &str, symbol: &str) {
        self.log(LogLevel::Info, "Analysis", message, Some(symbol));
    }
}