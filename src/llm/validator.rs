use anyhow::{Result, bail};

pub fn validate_sql(sql: &str) -> Result<()> {
    let sql_upper = sql.to_uppercase();
    
    // Check for dangerous commands
    let dangerous = ["DROP", "DELETE", "UPDATE", "INSERT", "ALTER", "CREATE", "TRUNCATE"];
    for keyword in dangerous {
        if sql_upper.contains(keyword) {
            bail!("Only SELECT queries are allowed. Found: {}", keyword);
        }
    }
    
    // Must contain SELECT
    if !sql_upper.contains("SELECT") {
        bail!("Query must contain SELECT statement");
    }
    
    // Must end with semicolon
    if !sql.trim().ends_with(';') {
        bail!("Query must end with semicolon");
    }
    
    // Basic syntax check (простая версия)
    if sql_upper.contains("FROM") && !sql_upper.contains("SELECT") {
        bail!("Invalid SQL syntax");
    }
    
    // CRITICAL: Check for SELECT * without LIMIT or aggregation
    // This will cause system failure with millions of rows
    if sql_upper.contains("SELECT *") {
        // Check if there's LIMIT
        let has_limit = sql_upper.contains("LIMIT");
        // Check if there's aggregation
        let has_aggregation = sql_upper.contains("COUNT") || sql_upper.contains("SUM") || 
                             sql_upper.contains("AVG") || sql_upper.contains("MAX") || 
                             sql_upper.contains("MIN") || sql_upper.contains("GROUP BY");
        
        if !has_limit && !has_aggregation {
            bail!("SELECT * without LIMIT or aggregation is FORBIDDEN. Database contains millions of rows. Use aggregation (COUNT, SUM, GROUP BY) or LIMIT (max 100).");
        }
    }
    
    // Check for queries without LIMIT that might return many rows
    // If query has FROM transactions but no LIMIT and no aggregation, it's dangerous
    if sql_upper.contains("FROM TRANSACTIONS") {
        let has_limit = sql_upper.contains("LIMIT");
        let has_aggregation = sql_upper.contains("COUNT") || sql_upper.contains("SUM") || 
                             sql_upper.contains("AVG") || sql_upper.contains("MAX") || 
                             sql_upper.contains("MIN") || sql_upper.contains("GROUP BY");
        
        // If no LIMIT and no aggregation, this is dangerous
        if !has_limit && !has_aggregation {
            bail!("Query without LIMIT or aggregation is FORBIDDEN. Database contains millions of rows. Use aggregation (COUNT, SUM, GROUP BY) or LIMIT (max 100).");
        }
        
        // If LIMIT is too large, reject it
        if has_limit {
            // Try to extract LIMIT value
            if let Some(limit_pos) = sql_upper.find("LIMIT") {
                let after_limit = &sql_upper[limit_pos + 5..];
                // Find the number after LIMIT
                let limit_str = after_limit.trim().split_whitespace().next().unwrap_or("");
                if let Ok(limit_val) = limit_str.parse::<u32>() {
                    if limit_val > 1000 {
                        bail!("LIMIT value too large ({}). Maximum allowed is 1000. Use aggregation instead.", limit_val);
                    }
                }
            }
        }
    }
    
    Ok(())
}

