use crate::config::Config;
use anyhow::Result;
use rig::providers::ollama::{Client as OllamaClient, ClientBuilder};
use rig::providers::gemini::Client as GeminiClient;
use rig::completion::CompletionRequest;
use rig::completion::request::CompletionModel;
use rig::completion::message::AssistantContent;
use rig::message::{Message, UserContent};
use rig::one_or_many::OneOrMany;
use rig::client::completion::CompletionClient;
use std::sync::Arc;

pub struct LLMClient {
    provider: LLMProvider,
}

enum LLMProvider {
    Ollama {
        client: Arc<OllamaClient>,
        model: String,
    },
    OpenAI {
        // TODO: Реализовать позже если нужно
        #[allow(dead_code)]
        api_key: String,
    },
    Gemini {
        client: Arc<GeminiClient>,
        model: String,
    },
}

impl LLMClient {
    pub async fn new(config: &Config) -> Result<Self> {
        let provider = match config.llm_provider.as_str() {
            "ollama" => {
                // Создаем Ollama клиент через rig-core
                let client = if config.ollama_url == "http://localhost:11434" {
                    // Используем дефолтный URL
                    OllamaClient::new()
                } else {
                    // Используем кастомный URL
                    ClientBuilder::new()
                        .base_url(&config.ollama_url)
                        .build()
                };
                
                LLMProvider::Ollama {
                    client: Arc::new(client),
                    model: config.ollama_model.clone(),
                }
            }
            "openai" => {
                let api_key = config.openai_api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
                LLMProvider::OpenAI { api_key }
            }
            "gemini" => {
                let api_key = config.gemini_api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("GEMINI_API_KEY or LLM_API_KEY not set"))?;
                // Создаем Gemini клиент через rig-core
                // rig-core использует переменную окружения GEMINI_API_KEY
                std::env::set_var("GEMINI_API_KEY", &api_key);
                let client = GeminiClient::from_env();
                LLMProvider::Gemini {
                    client: Arc::new(client),
                    model: config.gemini_model.clone(),
                }
            }
            _ => return Err(anyhow::anyhow!("Unknown LLM provider: {}. Supported: ollama, openai, gemini", config.llm_provider)),
        };
        
        Ok(Self { provider })
    }
    
    pub async fn generate_sql(
        &self,
        question: &str,
        previous_queries: &[&crate::query_context::QueryContext],
    ) -> Result<String> {
        let prompt = super::prompts::build_sql_generation_prompt(question, previous_queries);
        
        let raw_response = match &self.provider {
            LLMProvider::Ollama { client, model } => {
                self.call_ollama_with_rig(client, model, &prompt).await?
            }
            LLMProvider::OpenAI { .. } => {
                // TODO: Implement OpenAI fallback
                return Err(anyhow::anyhow!("OpenAI not implemented yet"));
            }
            LLMProvider::Gemini { client, model } => {
                self.call_gemini_with_rig(client, model, &prompt).await?
            }
        };
        
        let cleaned = super::prompts::clean_sql_response(&raw_response);
        
        // Validate SQL - if validation fails, try to clean and retry once
        match super::validator::validate_sql(&cleaned) {
            Ok(_) => Ok(cleaned),
            Err(e) => {
                tracing::warn!("SQL validation failed: {}. Attempting to fix...", e);
                
                // Try to extract SELECT statement if LLM added extra text
                let sql_upper = cleaned.to_uppercase();
                if let Some(select_pos) = sql_upper.find("SELECT") {
                    let extracted = &cleaned[select_pos..];
                    // Find the last semicolon
                    if let Some(semicolon_pos) = extracted.rfind(';') {
                        let fixed = &extracted[..=semicolon_pos];
                        match super::validator::validate_sql(fixed) {
                            Ok(_) => {
                                tracing::info!("Successfully fixed SQL by extracting SELECT statement");
                                Ok(fixed.to_string())
                            }
                            Err(e2) => {
                                tracing::error!("Failed to fix SQL: {}", e2);
                                Err(anyhow::anyhow!("Invalid SQL generated: {}. Please rephrase your question.", e))
                            }
                        }
                    } else {
                        Err(anyhow::anyhow!("Invalid SQL generated: {}. Please rephrase your question.", e))
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid SQL generated: {}. Please rephrase your question.", e))
                }
            }
        }
    }
    
    /// Generate chat response for regular conversation
    pub async fn generate_chat_response(
        &self,
        message: &str,
        history: &[(crate::chat::session::MessageRole, String)],
        language: &crate::utils::language::Language,
    ) -> Result<String> {
        let prompt = super::prompts::build_chat_prompt(message, history, language);
        
        let raw_response = match &self.provider {
            LLMProvider::Ollama { client, model } => {
                self.call_chat_ollama(client, model, &prompt).await?
            }
            LLMProvider::OpenAI { .. } => {
                return Err(anyhow::anyhow!("OpenAI not implemented yet"));
            }
            LLMProvider::Gemini { client, model } => {
                self.call_chat_gemini(client, model, &prompt).await?
            }
        };
        
        Ok(raw_response.trim().to_string())
    }
    
    async fn call_chat_ollama(
        &self,
        client: &Arc<OllamaClient>,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        let comp_model = client.as_ref().completion_model(model);
        
        let request = CompletionRequest {
            preamble: Some(
                "You are a friendly and helpful assistant for payment transaction analytics.".to_string(),
            ),
            chat_history: OneOrMany::one(Message::User {
                content: OneOrMany::one(UserContent::text(prompt)),
            }),
            documents: vec![],
            tools: vec![],
            temperature: Some(0.7),
            max_tokens: Some(512),
            tool_choice: None,
            additional_params: None,
        };
        
        let response = comp_model
            .completion(request)
            .await
            .map_err(|e| anyhow::anyhow!("Ollama API error: {}", e))?;
        
        let mut text_parts = Vec::new();
        for content in response.choice.iter() {
            if let AssistantContent::Text(text) = content {
                text_parts.push(text.text.clone());
            }
        }
        
        Ok(text_parts.join(" ").trim().to_string())
    }
    
    async fn call_chat_gemini(
        &self,
        client: &Arc<GeminiClient>,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        let comp_model = client.as_ref().completion_model(model);
        
        let mut additional_params = serde_json::Map::new();
        additional_params.insert(
            "generationConfig".to_string(),
            serde_json::json!({
                "temperature": 0.7,
                "maxOutputTokens": 512,
                "topP": 0.95,
                "topK": 40
            })
        );
        
        let request = CompletionRequest {
            preamble: Some(
                "You are a friendly and helpful assistant for payment transaction analytics.".to_string(),
            ),
            chat_history: OneOrMany::one(Message::User {
                content: OneOrMany::one(UserContent::text(prompt)),
            }),
            documents: vec![],
            tools: vec![],
            temperature: Some(0.7),
            max_tokens: Some(512),
            tool_choice: None,
            additional_params: Some(serde_json::Value::Object(additional_params)),
        };
        
        let response = comp_model
            .completion(request)
            .await
            .map_err(|e| anyhow::anyhow!("Gemini API error: {}", e))?;
        
        let mut text_parts = Vec::new();
        for content in response.choice.iter() {
            if let AssistantContent::Text(text) = content {
                text_parts.push(text.text.clone());
            }
        }
        
        Ok(text_parts.join(" ").trim().to_string())
    }
    
    async fn call_ollama_with_rig(
        &self,
        client: &Arc<OllamaClient>,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        // Получаем ссылку из Arc и создаем completion модель
        let comp_model = client.as_ref().completion_model(model);
        
        // Создаем запрос на генерацию через rig-core
        let request = CompletionRequest {
            preamble: Some(
                "You are an expert PostgreSQL database architect. Generate ONLY SQL queries, no explanations."
                    .to_string(),
            ),
            chat_history: OneOrMany::one(Message::User {
                content: OneOrMany::one(UserContent::text(prompt)),
            }),
            documents: vec![],
            tools: vec![],
            temperature: Some(0.1),  // Low temperature for deterministic SQL
            max_tokens: Some(1024),
            tool_choice: None,
            additional_params: None,
        };
        
        // Отправляем запрос через rig-core
        let response = comp_model
            .completion(request)
            .await
            .map_err(|e| anyhow::anyhow!("LLM error: {}", e))?;
        
        // Извлекаем текст из choice (OneOrMany<AssistantContent>)
        // OneOrMany реализует IntoIterator, можно итерировать напрямую
        let mut text_parts = Vec::new();
        for content in response.choice.iter() {
            // AssistantContent может быть Text, ToolCall, Reasoning и т.д.
            if let AssistantContent::Text(text) = content {
                // Text имеет поле text: String
                text_parts.push(text.text.clone());
            }
        }
        
        let text = text_parts.join(" ").trim().to_string();
        
        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty response from LLM"));
        }
        
        Ok(text)
    }
    
    async fn call_gemini_with_rig(
        &self,
        client: &Arc<GeminiClient>,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        // Получаем ссылку из Arc и создаем completion модель
        let comp_model = client.as_ref().completion_model(model);
        
        // Создаем запрос на генерацию через rig-core
        // Gemini API требует generationConfig в additional_params
        let mut additional_params = serde_json::Map::new();
        additional_params.insert(
            "generationConfig".to_string(),
            serde_json::json!({
                "temperature": 0.1,
                "maxOutputTokens": 512,
                "topP": 0.95,
                "topK": 40
            })
        );
        
        let request = CompletionRequest {
            preamble: Some(
                "You are an expert PostgreSQL database architect. Generate ONLY SQL queries, no explanations."
                    .to_string(),
            ),
            chat_history: OneOrMany::one(Message::User {
                content: OneOrMany::one(UserContent::text(prompt)),
            }),
            documents: vec![],
            tools: vec![],
            temperature: Some(0.1),  // Low temperature for deterministic SQL
            max_tokens: Some(512),
            tool_choice: None,
            additional_params: Some(serde_json::Value::Object(additional_params)),
        };
        
        // Отправляем запрос через rig-core
        let response = comp_model
            .completion(request)
            .await
            .map_err(|e| anyhow::anyhow!("Gemini API error: {}", e))?;
        
        // Извлекаем текст из choice (OneOrMany<AssistantContent>)
        let mut text_parts = Vec::new();
        for content in response.choice.iter() {
            if let AssistantContent::Text(text) = content {
                text_parts.push(text.text.clone());
            }
        }
        
        let text = text_parts.join(" ").trim().to_string();
        
        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty response from Gemini API"));
        }
        
        Ok(text)
    }
}
