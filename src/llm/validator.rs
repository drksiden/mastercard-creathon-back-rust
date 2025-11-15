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
    
    Ok(())
}

