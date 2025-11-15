// Mock data generation utilities
// This module can be used to generate additional test data if needed

use crate::error::AppError;
use sqlx::PgPool;

#[allow(dead_code)]
pub async fn generate_additional_transactions(
    pool: &PgPool,
    count: i32,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO transactions (merchant_id, merchant_name, amount, status, created_at)
        SELECT 
            m.id,
            m.name,
            (random() * 10000 + 100)::DECIMAL(10,2),
            (ARRAY['completed', 'declined', 'pending'])[floor(random() * 3 + 1)],
            NOW() - (random() * 365 * 3 || ' days')::INTERVAL
        FROM merchants m
        CROSS JOIN generate_series(1, $1)
        "#,
    )
    .bind(count)
    .execute(pool)
    .await?;
    
    Ok(())
}

