pub fn build_sql_generation_prompt(question: &str) -> String {
    let schema = get_database_schema();
    let rules = get_sql_rules();
    let examples = get_few_shot_examples();
    
    format!(
        r#"You are an expert PostgreSQL database architect for a payment processing system.

{schema}

{rules}

{examples}

USER QUESTION: {question}

Generate ONLY the SQL query, no explanations or markdown formatting.

SQL QUERY:"#
    )
}

fn get_database_schema() -> &'static str {
    r#"DATABASE SCHEMA:

Table: transactions
├─ id: SERIAL PRIMARY KEY
├─ transaction_id: VARCHAR(255) NOT NULL (unique transaction identifier)
├─ transaction_timestamp: TIMESTAMP (when transaction occurred)
├─ card_id: INTEGER (card identifier)
├─ expiry_date: VARCHAR(10) (card expiry date, format MM/YY)
├─ issuer_bank_name: VARCHAR(255) (bank that issued the card)
├─ merchant_id: INTEGER (merchant identifier)
├─ merchant_mcc: INTEGER (Merchant Category Code)
├─ mcc_category: VARCHAR(255) (category name for MCC, e.g., 'Retail', 'Restaurants', 'Gas Stations')
├─ merchant_city: VARCHAR(255) (city where merchant is located)
├─ transaction_type: VARCHAR(50) (e.g., 'Purchase', 'Refund', 'Authorization', 'Reversal')
├─ transaction_amount_kzt: NUMERIC(15, 2) (amount in KZT)
├─ original_amount: NUMERIC(15, 2) (original amount if currency conversion occurred, nullable)
├─ transaction_currency: VARCHAR(3) (currency code: KZT, USD, EUR, etc.)
├─ acquirer_country_iso: VARCHAR(3) (ISO country code where transaction was acquired)
├─ pos_entry_mode: VARCHAR(50) (e.g., 'Chip', 'Contactless', 'Magnetic Stripe')
└─ wallet_type: VARCHAR(50) (e.g., 'Apple Pay', 'Google Pay', 'Samsung Pay', or NULL)

INDEXES:
- transactions(transaction_timestamp)
- transactions(merchant_id)
- transactions(merchant_mcc)
- transactions(mcc_category)
- transactions(transaction_type)
- transactions(card_id)
- transactions(issuer_bank_name)
- transactions(merchant_city)
- transactions(transaction_id)"#
}

fn get_sql_rules() -> &'static str {
    r#"RULES:
1. Generate ONLY SELECT statements (no INSERT, UPDATE, DELETE, DROP)
2. Use proper PostgreSQL syntax (not MySQL or other dialects)
3. For date ranges with transaction_timestamp:
   - Use: transaction_timestamp >= '2024-01-01' AND transaction_timestamp < '2025-01-01'
   - For "this year": EXTRACT(YEAR FROM transaction_timestamp) = EXTRACT(YEAR FROM CURRENT_DATE)
   - For "last month": DATE_TRUNC('month', transaction_timestamp) = DATE_TRUNC('month', CURRENT_DATE - INTERVAL '1 month')
   - For "today": DATE(transaction_timestamp) = CURRENT_DATE
   - For "last 7 days": transaction_timestamp >= CURRENT_DATE - INTERVAL '7 days'
4. For text fields (issuer_bank_name, mcc_category, merchant_city, etc.): Use ILIKE '%text%' for case-insensitive partial matching
5. For transaction amounts: Use transaction_amount_kzt for KZT amounts, or original_amount for original currency
6. For "top N": Add ORDER BY and LIMIT N
7. For aggregations: Use appropriate functions (SUM, AVG, COUNT, etc.)
8. For percentage calculations: Cast to FLOAT and multiply by 100
9. Always include proper WHERE clauses for filters
10. When filtering by currency: Use transaction_currency = 'KZT' (or other currency code)
11. When grouping by time periods: Use DATE_TRUNC('day', transaction_timestamp), DATE_TRUNC('month', transaction_timestamp), etc.
12. End query with semicolon"#
}

fn get_few_shot_examples() -> &'static str {
    r#"EXAMPLES:

Q: "Total transactions in 2024"
A: SELECT COUNT(*) as total_transactions FROM transactions WHERE transaction_timestamp >= '2024-01-01' AND transaction_timestamp < '2025-01-01';

Q: "Top 5 merchants by transaction volume in KZT"
A: SELECT merchant_id, SUM(transaction_amount_kzt) as total_volume_kzt FROM transactions WHERE transaction_type = 'Purchase' GROUP BY merchant_id ORDER BY total_volume_kzt DESC LIMIT 5;

Q: "Average transaction amount for Halyk Bank cards in Almaty"
A: SELECT AVG(transaction_amount_kzt) as average_amount FROM transactions WHERE issuer_bank_name ILIKE '%halyk%' AND merchant_city ILIKE '%almaty%' AND transaction_type = 'Purchase';

Q: "Transaction volume by MCC category last month"
A: SELECT mcc_category, SUM(transaction_amount_kzt) as total_volume, COUNT(*) as transaction_count FROM transactions WHERE DATE_TRUNC('month', transaction_timestamp) = DATE_TRUNC('month', CURRENT_DATE - INTERVAL '1 month') AND transaction_type = 'Purchase' GROUP BY mcc_category ORDER BY total_volume DESC;

Q: "Transactions by wallet type today"
A: SELECT wallet_type, COUNT(*) as transaction_count, SUM(transaction_amount_kzt) as total_amount FROM transactions WHERE DATE(transaction_timestamp) = CURRENT_DATE GROUP BY wallet_type ORDER BY transaction_count DESC;

Q: "Top 10 cities by transaction count"
A: SELECT merchant_city, COUNT(*) as transaction_count, SUM(transaction_amount_kzt) as total_volume FROM transactions WHERE transaction_type = 'Purchase' GROUP BY merchant_city ORDER BY transaction_count DESC LIMIT 10;

Q: "Refund rate by transaction type"
A: SELECT transaction_type, COUNT(*) as total_count, (COUNT(CASE WHEN transaction_type = 'Refund' THEN 1 END)::FLOAT / COUNT(*) * 100) as refund_rate_percent FROM transactions GROUP BY transaction_type;

Q: "Daily transaction volume for last 7 days"
A: SELECT DATE(transaction_timestamp) as date, SUM(transaction_amount_kzt) as daily_volume, COUNT(*) as transaction_count FROM transactions WHERE transaction_timestamp >= CURRENT_DATE - INTERVAL '7 days' AND transaction_type = 'Purchase' GROUP BY DATE(transaction_timestamp) ORDER BY date DESC;"#
}

pub fn clean_sql_response(raw_sql: &str) -> String {
    raw_sql
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("```")
                && !trimmed.is_empty()
                && !trimmed.to_lowercase().starts_with("note:")
                && !trimmed.to_lowercase().starts_with("explanation:")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .trim_end_matches(';')
        .to_string()
        + ";"
}

