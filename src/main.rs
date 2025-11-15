mod api;
mod config;
mod db;
mod error;
mod llm;
mod state;
mod utils;

use anyhow::Result;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    utils::logger::init();
    
    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded");
    
    // Initialize database
    let db_pool = db::pool::create_pool(&config.database_url).await?;
    tracing::info!("Database connected");
    
    // Run migrations only if tables don't exist (optional)
    // Check if transactions table exists
    let table_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'transactions'
        )"
    )
    .fetch_one(&db_pool)
    .await
    .unwrap_or(false);
    
    if !table_exists {
        tracing::info!("Tables not found, running migrations...");
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await?;
        tracing::info!("Migrations completed");
    } else {
        tracing::info!("Tables already exist, skipping migrations");
    }
    
    // Initialize LLM client
    let llm_client = llm::client::LLMClient::new(&config).await?;
    tracing::info!("LLM client initialized: {}", config.llm_provider);
    
    // Warm up LLM (optional, don't fail if LLM is not available)
    tracing::info!("Warming up LLM...");
    match llm_client.generate_sql("How many transactions are there?").await {
        Ok(_) => tracing::info!("LLM ready!"),
        Err(e) => tracing::warn!("LLM warm-up failed (will continue anyway): {}", e),
    }
    
    // Create application state
    let state = state::AppState::new(db_pool, llm_client, config.clone());
    
    // Build router
    let app = Router::new()
        .nest("/api", api::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Server running on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

