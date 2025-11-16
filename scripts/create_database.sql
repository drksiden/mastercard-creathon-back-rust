-- Скрипт для создания базы данных и таблиц
-- Использование: psql -U postgres -f scripts/create_database.sql

-- Создаем базу данных (если не существует)
-- Примечание: CREATE DATABASE нельзя выполнить внутри транзакции
-- Выполните это вручную: CREATE DATABASE payment_analytics;

-- Подключаемся к базе данных payment_analytics
\c payment_analytics;

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    transaction_id VARCHAR(255) NOT NULL,
    transaction_timestamp TIMESTAMP,
    card_id INTEGER,
    expiry_date VARCHAR(10),
    issuer_bank_name VARCHAR(255),
    merchant_id INTEGER,
    merchant_mcc INTEGER,
    mcc_category VARCHAR(255) CHECK (mcc_category IN (
        'Clothing & Apparel',
        'Dining & Restaurants',
        'Electronics & Software',
        'Fuel & Service Stations',
        'General Retail & Department',
        'Grocery & Food Markets',
        'Hobby, Books, Sporting Goods',
        'Home Furnishings & Supplies',
        'Pharmacies & Health',
        'Services (Other)',
        'Travel & Transportation',
        'Unknown',
        'Utilities & Bill Payments'
    )),
    merchant_city VARCHAR(255),
    transaction_type VARCHAR(50) CHECK (transaction_type IN (
        'ATM_WITHDRAWAL',
        'BILL_PAYMENT',
        'ECOM',
        'P2P_IN',
        'P2P_OUT',
        'POS',
        'SALARY'
    )),
    transaction_amount_kzt NUMERIC(15, 2),
    original_amount NUMERIC(15, 2),
    transaction_currency VARCHAR(3) CHECK (transaction_currency IN (
        'AMD', 'BYN', 'CNY', 'EUR', 'GEL', 'KGS', 'KZT', 'TRY', 'USD', 'UZS'
    )),
    acquirer_country_iso VARCHAR(3) CHECK (acquirer_country_iso IN (
        'ARM', 'BLR', 'CHN', 'GEO', 'ITA', 'KAZ', 'KGZ', 'TUR', 'USA', 'UZB'
    )),
    pos_entry_mode VARCHAR(50) CHECK (pos_entry_mode IN (
        'Contactless', 'ECOM', 'QR_Code', 'Swipe'
    ) OR pos_entry_mode IS NULL),
    wallet_type VARCHAR(50)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(transaction_timestamp);
CREATE INDEX IF NOT EXISTS idx_transactions_merchant_id ON transactions(merchant_id);
CREATE INDEX IF NOT EXISTS idx_transactions_merchant_mcc ON transactions(merchant_mcc);
CREATE INDEX IF NOT EXISTS idx_transactions_mcc_category ON transactions(mcc_category);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_type ON transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_card_id ON transactions(card_id);
CREATE INDEX IF NOT EXISTS idx_transactions_issuer_bank_name ON transactions(issuer_bank_name);
CREATE INDEX IF NOT EXISTS idx_transactions_merchant_city ON transactions(merchant_city);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_id ON transactions(transaction_id);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_currency ON transactions(transaction_currency);
CREATE INDEX IF NOT EXISTS idx_transactions_acquirer_country_iso ON transactions(acquirer_country_iso);

-- Audit log for tracking queries
CREATE TABLE IF NOT EXISTS query_audit_log (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(100),
    question TEXT NOT NULL,
    generated_sql TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    execution_time_ms INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Создаем индекс для query_audit_log
CREATE INDEX IF NOT EXISTS idx_query_audit_log_created_at ON query_audit_log(created_at);
CREATE INDEX IF NOT EXISTS idx_query_audit_log_user_id ON query_audit_log(user_id);

