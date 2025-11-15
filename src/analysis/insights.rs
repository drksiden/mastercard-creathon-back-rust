use serde::{Deserialize, Serialize};

/// Structured analysis result with text, insights, and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Main answer to the user's question (headline)
    pub headline: String,
    
    /// Key insights and findings
    pub insights: Vec<Insight>,
    
    /// Detailed explanation
    pub explanation: String,
    
    /// Suggested next questions
    pub suggested_questions: Vec<String>,
    
    /// Chart type recommendation (if applicable)
    pub chart_type: Option<ChartType>,
    
    /// Raw data for tables/charts
    pub data: Vec<serde_json::Value>,
}

/// Individual insight or finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub title: String,
    pub description: String,
    pub significance: InsightSignificance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSignificance {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Table,
    Trend,
}

