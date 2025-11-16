use crate::config::Config;
use crate::utils::language::{detect_language, Language};
use anyhow::Result;
use rig::completion::CompletionRequest;
use rig::completion::request::CompletionModel;
use rig::completion::message::AssistantContent;
use rig::message::{Message, UserContent};
use rig::one_or_many::OneOrMany;
use rig::client::completion::CompletionClient;
use super::insights::{AnalysisResult, ChartType};

pub struct AnalysisClient {
    config: Config,
}

impl AnalysisClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Analyze SQL query results and generate human-readable insights
    pub async fn analyze_results(
        &self,
        question: &str,
        sql: &str,
        data: &[serde_json::Value],
        language: &Language,
    ) -> Result<AnalysisResult> {
        // Build prompt for analysis
        let prompt = build_analysis_prompt(question, sql, data, language);
        
        // Generate analysis using LLM
        let analysis_text = self.generate_analysis(&prompt).await?;
        
        // Parse structured response
        let result = parse_analysis_response(&analysis_text, data, language)?;
        
        Ok(result)
    }

    async fn generate_analysis(&self, prompt: &str) -> Result<String> {
        // Retry logic for Gemini (sometimes returns empty response)
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            attempts += 1;
            match self.call_analysis_direct(prompt).await {
                Ok(text) if !text.trim().is_empty() => {
                    return Ok(text);
                }
                Ok(_) => {
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("Empty response from LLM after {} attempts", max_attempts));
                    }
                    tracing::warn!("Empty response from LLM, retrying (attempt {}/{})", attempts, max_attempts);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(e);
                    }
                    tracing::warn!("LLM error, retrying (attempt {}/{}): {}", attempts, max_attempts, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
                }
            }
        }
    }
    
    async fn call_analysis_direct(&self, prompt: &str) -> Result<String> {
        // Direct implementation using rig-core
        match self.config.llm_provider.as_str() {
            "ollama" => {
                let client = rig::providers::ollama::Client::new();
                let model = &self.config.ollama_model;
                self.call_analysis_ollama_direct(&client, model, prompt).await
            }
            "gemini" => {
                let api_key = self.config.gemini_api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("GEMINI_API_KEY not set"))?;
                std::env::set_var("GEMINI_API_KEY", &api_key);
                let client = rig::providers::gemini::Client::from_env();
                let model = &self.config.gemini_model;
                self.call_analysis_gemini_direct(&client, model, prompt).await
            }
            _ => Err(anyhow::anyhow!("Unsupported provider for analysis")),
        }
    }
    
    async fn call_analysis_ollama_direct(
        &self,
        client: &rig::providers::ollama::Client,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        let comp_model = client.completion_model(model);
        
        let request = CompletionRequest {
            preamble: Some(
                "You are a data analyst expert. Analyze query results and provide structured insights in JSON format.".to_string(),
            ),
            chat_history: OneOrMany::one(Message::User {
                content: OneOrMany::one(UserContent::text(prompt)),
            }),
            documents: vec![],
            tools: vec![],
            temperature: Some(0.7),
            max_tokens: Some(1024),
            tool_choice: None,
            additional_params: None,
        };
        
        let response = comp_model
            .completion(request)
            .await
            .map_err(|e| anyhow::anyhow!("LLM analysis error: {}", e))?;
        
        let mut text_parts = Vec::new();
        for content in response.choice.iter() {
            if let AssistantContent::Text(text) = content {
                text_parts.push(text.text.clone());
            }
        }
        
        let text = text_parts.join(" ").trim().to_string();
        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty analysis response from LLM"));
        }
        
        Ok(text)
    }
    
    async fn call_analysis_gemini_direct(
        &self,
        client: &rig::providers::gemini::Client,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        let comp_model = client.completion_model(model);
        
        let mut additional_params = serde_json::Map::new();
        additional_params.insert(
            "generationConfig".to_string(),
            serde_json::json!({
                "temperature": 0.7,
                "maxOutputTokens": 1024,
                "topP": 0.95,
                "topK": 40
            })
        );
        
        let request = CompletionRequest {
            preamble: Some(
                "You are a data analyst expert. Analyze query results and provide structured insights in JSON format.".to_string(),
            ),
            chat_history: OneOrMany::one(Message::User {
                content: OneOrMany::one(UserContent::text(prompt)),
            }),
            documents: vec![],
            tools: vec![],
            temperature: Some(0.7),
            max_tokens: Some(1024),
            tool_choice: None,
            additional_params: Some(serde_json::Value::Object(additional_params)),
        };
        
        let response = comp_model
            .completion(request)
            .await
            .map_err(|e| anyhow::anyhow!("Gemini analysis error: {}", e))?;
        
        let mut text_parts = Vec::new();
        for content in response.choice.iter() {
            if let AssistantContent::Text(text) = content {
                text_parts.push(text.text.clone());
            }
        }
        
        let text = text_parts.join(" ").trim().to_string();
        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty analysis response from Gemini"));
        }
        
        Ok(text)
    }

}

fn build_analysis_prompt(question: &str, sql: &str, data: &[serde_json::Value], language: &Language) -> String {
    let data_summary = if data.len() <= 10 {
        format!("Full data: {}", serde_json::to_string(data).unwrap_or_default())
    } else {
        format!(
            "First 5 rows: {}\n... and {} more rows",
            serde_json::to_string(&data[..5.min(data.len())]).unwrap_or_default(),
            data.len() - 5
        )
    };
    
    let (language_name, language_instruction) = match language {
        Language::Russian => ("Russian", "Отвечайте на русском языке. Все тексты (headline, insights, explanation, suggested_questions) должны быть на русском языке."),
        Language::English => ("English", "Respond in English. All texts (headline, insights, explanation, suggested_questions) should be in English."),
        Language::Kazakh => ("Kazakh", "Қазақ тілінде жауап беріңіз. Барлық мәтіндер (headline, insights, explanation, suggested_questions) қазақ тілінде болуы керек."),
    };
    
    format!(
        r#"You are a data analyst. Analyze the database query results and provide insights in JSON format.

{language_instruction}

USER QUESTION: {question}

SQL QUERY: {sql}

QUERY RESULTS:
{data_summary}

CRITICAL: You MUST return ONLY valid JSON, no markdown, no code blocks, no explanations outside JSON.

Required JSON structure:
{{
  "headline": "Main answer to the question in {language_name} (1-2 sentences)",
  "insights": [
    {{
      "title": "Key finding title in {language_name}",
      "description": "Detailed explanation in {language_name}",
      "significance": "High"
    }}
  ],
  "explanation": "Detailed explanation in {language_name} (2-3 sentences)",
  "suggested_questions": ["Question 1 in {language_name}", "Question 2 in {language_name}"],
  "chart_type": "Bar"
}}

Rules:
- headline: Direct answer to the question in {language_name} - MUST be a complete, human-readable sentence that answers the question directly. For COUNT queries, format as "Всего найдено X транзакций" or "Найдено X транзакций". For SUM queries, format as "Общая сумма составляет X тенге" or similar.
- insights: 2-3 key findings with significance (High/Medium/Low) in {language_name} - each insight should be meaningful and actionable. If result is a single number, provide insights about what this number means in context.
- explanation: Detailed analysis in {language_name} - MUST be comprehensive (4-6 sentences), explaining what the data means, trends, patterns, and implications. Do NOT just repeat the numbers, provide context and interpretation. For single values, explain what this number means, compare it to expectations, provide context about typical values, and explain business implications.
- suggested_questions: 2-3 follow-up questions in {language_name} that would help users explore the data further
- chart_type: One of: Bar, Line, Pie, Table, Trend (or null if not applicable). Use Table ONLY when user explicitly asks for a table or when data has multiple rows with categories. For single aggregated values (COUNT, SUM, AVG without GROUP BY), ALWAYS use null.

CRITICAL: 
1. Your explanation MUST be detailed and human-readable. Don't just list numbers - explain what they mean, provide context, identify trends, and help users understand the significance of the data.
2. If the result is a single aggregated value (like COUNT(*), SUM, AVG), focus on explaining what this number means in context, provide comparisons, and explain business implications. DO NOT suggest creating tables for single values.
3. Always provide a natural language description that answers the user's question directly and comprehensively.
4. For COUNT queries: Explain what the number represents, whether it's high or low compared to typical values, and what it means for the business.
5. For SUM/AVG queries: Explain the total/average amount, provide context about typical values, and explain what this means."#
    )
}

fn parse_analysis_response(text: &str, data: &[serde_json::Value], language: &Language) -> Result<AnalysisResult> {
    tracing::debug!("Raw analysis response: {}", text);
    
    // Try to extract JSON from the response
    let json_text = extract_json_from_text(text);
    tracing::debug!("Extracted JSON: {}", json_text);
    
    // Parse JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| {
            tracing::error!("JSON parse error: {} for text: {}", e, json_text);
            anyhow::anyhow!("Failed to parse analysis JSON: {}. Raw text: {}", e, text)
        })?;
    
    // Extract fields with better error handling
    let headline = parsed["headline"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| parsed["summary"].as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| {
            // Fallback: generate headline from data
            let (found_records, analysis_complete) = match language {
                Language::Russian => ("Найдено {} записей", "Анализ завершен"),
                Language::English => ("Found {} records", "Analysis complete"),
                Language::Kazakh => ("{} жазба табылды", "Талдау аяқталды"),
            };
            if let Some(first_row) = data.first() {
                if let Some(obj) = first_row.as_object() {
                    if let Some(count) = obj.values().next() {
                        return found_records.replace("{}", &count.to_string());
                    }
                }
            }
            analysis_complete.to_string()
        });
    
    let insights = parsed["insights"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let title = v["title"].as_str()?;
                    let description = v["description"].as_str()?;
                    let significance_str = v["significance"].as_str().unwrap_or("Low");
                    
                    Some(super::insights::Insight {
                        title: title.to_string(),
                        description: description.to_string(),
                        significance: match significance_str {
                            "High" => super::insights::InsightSignificance::High,
                            "Medium" => super::insights::InsightSignificance::Medium,
                            _ => super::insights::InsightSignificance::Low,
                        },
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    
    let explanation = parsed["explanation"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Fallback explanation
            let (result_contains, value, first_values) = match language {
                Language::Russian => (
                    "Результат запроса содержит {} строк данных. {}",
                    "Значение: {:?}",
                    "Первые значения: {:?}",
                ),
                Language::English => (
                    "Query result contains {} rows of data. {}",
                    "Value: {:?}",
                    "First values: {:?}",
                ),
                Language::Kazakh => (
                    "Сұрау нәтижесі {} жол деректерді қамтиды. {}",
                    "Мән: {:?}",
                    "Алғашқы мәндер: {:?}",
                ),
            };
            format!("{}{}", 
                result_contains.replace("{}", &data.len().to_string()),
                if data.len() == 1 {
                    value.replace("{:?}", &format!("{:?}", data[0]))
                } else {
                    first_values.replace("{:?}", &format!("{:?}", &data[..data.len().min(3)]))
                }
            )
        });
    
    let suggested_questions = parsed["suggested_questions"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| {
            match language {
                Language::Russian => vec![
                    "Показать детализацию".to_string(),
                    "Сравнить с другими периодами".to_string(),
                ],
                Language::English => vec![
                    "Show details".to_string(),
                    "Compare with other periods".to_string(),
                ],
                Language::Kazakh => vec![
                    "Толық мәліметтерді көрсету".to_string(),
                    "Басқа кезеңдермен салыстыру".to_string(),
                ],
            }
        });
    
    let chart_type = parsed["chart_type"]
        .as_str()
        .and_then(|s| match s {
            "Bar" => Some(ChartType::Bar),
            "Line" => Some(ChartType::Line),
            "Pie" => Some(ChartType::Pie),
            "Table" => Some(ChartType::Table),
            "Trend" => Some(ChartType::Trend),
            _ => None,
        });
    
    Ok(AnalysisResult {
        headline,
        insights,
        explanation,
        suggested_questions,
        chart_type,
        data: data.to_vec(),
    })
}

fn extract_json_from_text(text: &str) -> String {
    // Try to find JSON object in the text
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return text[start..=end].to_string();
        }
    }
    
    // If no JSON found, return the text as-is (will fail parsing, but that's ok)
    text.to_string()
}

