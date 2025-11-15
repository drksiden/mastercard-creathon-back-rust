use crate::error::AppError;
use sqlx::{PgPool, Row, Column, TypeInfo};

pub async fn execute_query(pool: &PgPool, sql: &str) -> Result<Vec<serde_json::Value>, AppError> {
    // Execute query and get rows
    let rows = sqlx::query(sql).fetch_all(pool).await?;
    
    // Convert rows to JSON using a simpler approach
    let mut results = Vec::new();
    for row in rows {
        let mut map = serde_json::Map::new();
        for column in row.columns() {
            let column_name = column.name();
            let value: serde_json::Value = match column.type_info().name() {
                "INT4" => {
                    if let Ok(v) = row.try_get::<i32, _>(column_name) {
                        serde_json::Value::Number(v.into())
                    } else {
                        serde_json::Value::Null
                    }
                }
                "INT8" => {
                    if let Ok(v) = row.try_get::<i64, _>(column_name) {
                        serde_json::Value::Number(v.into())
                    } else {
                        serde_json::Value::Null
                    }
                }
                "NUMERIC" | "DECIMAL" => {
                    // Try as string first, then as f64
                    if let Ok(v) = row.try_get::<String, _>(column_name) {
                        serde_json::Value::String(v)
                    } else if let Ok(v) = row.try_get::<f64, _>(column_name) {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from(0))
                        )
                    } else {
                        serde_json::Value::Null
                    }
                }
                "FLOAT4" => {
                    if let Ok(v) = row.try_get::<f32, _>(column_name) {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(v as f64).unwrap_or(serde_json::Number::from(0))
                        )
                    } else {
                        serde_json::Value::Null
                    }
                }
                "FLOAT8" => {
                    if let Ok(v) = row.try_get::<f64, _>(column_name) {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from(0))
                        )
                    } else {
                        serde_json::Value::Null
                    }
                }
                "BOOL" => {
                    if let Ok(v) = row.try_get::<bool, _>(column_name) {
                        serde_json::Value::Bool(v)
                    } else {
                        serde_json::Value::Null
                    }
                }
                "TIMESTAMP" | "TIMESTAMPTZ" => {
                    if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(column_name) {
                        serde_json::Value::String(v.to_rfc3339())
                    } else if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(column_name) {
                        serde_json::Value::String(v.to_string())
                    } else {
                        serde_json::Value::Null
                    }
                }
                _ => {
                    // Try as string for everything else
                    if let Ok(v) = row.try_get::<String, _>(column_name) {
                        serde_json::Value::String(v)
                    } else {
                        serde_json::Value::Null
                    }
                }
            };
            map.insert(column_name.to_string(), value);
        }
        results.push(serde_json::Value::Object(map));
    }
    
    Ok(results)
}

