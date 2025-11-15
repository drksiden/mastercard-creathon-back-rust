use crate::config::Config;
use anyhow::Result;

pub struct LLMClient {
    provider: LLMProvider,
}

enum LLMProvider {
    Ollama {
        client: reqwest::Client,
        url: String,
        model: String,
    },
    OpenAI {
        // Реализуете позже если нужно
        api_key: String,
    },
}

impl LLMClient {
    pub async fn new(config: &Config) -> Result<Self> {
        let provider = match config.llm_provider.as_str() {
            "ollama" => LLMProvider::Ollama {
                client: reqwest::Client::new(),
                url: config.ollama_url.clone(),
                model: config.ollama_model.clone(),
            },
            "openai" => {
                let api_key = config.openai_api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
                LLMProvider::OpenAI { api_key }
            },
            _ => return Err(anyhow::anyhow!("Unknown LLM provider")),
        };
        
        Ok(Self { provider })
    }
    
    pub async fn generate_sql(&self, question: &str) -> Result<String> {
        let prompt = super::prompts::build_sql_generation_prompt(question);
        
        let raw_response = match &self.provider {
            LLMProvider::Ollama { client, url, model } => {
                self.call_ollama(client, url, model, &prompt).await?
            }
            LLMProvider::OpenAI { .. } => {
                // TODO: Implement OpenAI fallback
                return Err(anyhow::anyhow!("OpenAI not implemented yet"));
            }
        };
        
        let cleaned = super::prompts::clean_sql_response(&raw_response);
        
        // Validate SQL
        super::validator::validate_sql(&cleaned)?;
        
        Ok(cleaned)
    }
    
    async fn call_ollama(
        &self,
        client: &reqwest::Client,
        url: &str,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        #[derive(serde::Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
            options: Options,
        }
        
        #[derive(serde::Serialize)]
        struct Options {
            temperature: f32,
            top_p: f32,
            num_predict: i32,
        }
        
        #[derive(serde::Deserialize)]
        struct OllamaResponse {
            response: String,
        }
        
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: Options {
                temperature: 0.1,  // Low temperature for deterministic SQL
                top_p: 0.9,
                num_predict: 512,
            },
        };
        
        let response = client
            .post(format!("{}/api/generate", url))
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;
        
        Ok(response.response)
    }
}

