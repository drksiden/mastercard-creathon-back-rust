-- migrations/001_init.sql

-- Transactions table
CREATE TABLE transactions (
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
CREATE INDEX idx_transactions_timestamp ON transactions(transaction_timestamp);
CREATE INDEX idx_transactions_merchant_id ON transactions(merchant_id);
CREATE INDEX idx_transactions_merchant_mcc ON transactions(merchant_mcc);
CREATE INDEX idx_transactions_mcc_category ON transactions(mcc_category);
CREATE INDEX idx_transactions_transaction_type ON transactions(transaction_type);
CREATE INDEX idx_transactions_card_id ON transactions(card_id);
CREATE INDEX idx_transactions_issuer_bank_name ON transactions(issuer_bank_name);
CREATE INDEX idx_transactions_merchant_city ON transactions(merchant_city);
CREATE INDEX idx_transactions_transaction_id ON transactions(transaction_id);
CREATE INDEX idx_transactions_transaction_currency ON transactions(transaction_currency);
CREATE INDEX idx_transactions_acquirer_country_iso ON transactions(acquirer_country_iso);

-- Audit log for tracking queries
CREATE TABLE query_audit_log (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(100),
    question TEXT NOT NULL,
    generated_sql TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    execution_time_ms INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Generate mock transactions
DO $$
DECLARE
    mcc_categories VARCHAR[] := ARRAY[
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
    ];
    transaction_types VARCHAR[] := ARRAY['ATM_WITHDRAWAL', 'BILL_PAYMENT', 'ECOM', 'P2P_IN', 'P2P_OUT', 'POS', 'SALARY'];
    wallet_types VARCHAR[] := ARRAY['Apple Pay', 'Google Pay', 'Samsung Pay', NULL];
    issuer_banks VARCHAR[] := ARRAY['Halyk Bank', 'Kaspi Bank', 'ForteBank', 'Jusan Bank', 'Eurasian Bank', 'Bank CenterCredit'];
    cities VARCHAR[] := ARRAY['Almaty', 'Astana', 'Shymkent', 'Karaganda', 'Aktobe', 'Taraz', 'Pavlodar', 'Oskemen'];
    currencies VARCHAR[] := ARRAY['AMD', 'BYN', 'CNY', 'EUR', 'GEL', 'KGS', 'KZT', 'TRY', 'USD', 'UZS'];
    countries VARCHAR[] := ARRAY['ARM', 'BLR', 'CHN', 'GEO', 'ITA', 'KAZ', 'KGZ', 'TUR', 'USA', 'UZB'];
    pos_modes VARCHAR[] := ARRAY['Contactless', 'ECOM', 'QR_Code', 'Swipe', NULL];
BEGIN
    -- Generate 5000 mock transactions
    FOR i IN 1..5000 LOOP
        INSERT INTO transactions (
            transaction_id,
            transaction_timestamp,
            card_id,
            expiry_date,
            issuer_bank_name,
            merchant_id,
            merchant_mcc,
            mcc_category,
            merchant_city,
            transaction_type,
            transaction_amount_kzt,
            original_amount,
            transaction_currency,
            acquirer_country_iso,
            pos_entry_mode,
            wallet_type
        ) VALUES (
            'TXN' || LPAD(i::TEXT, 10, '0'),
            NOW() - (random() * 365 * 2 || ' days')::INTERVAL,  -- Last 2 years
            floor(random() * 1000000 + 1)::INTEGER,
            TO_CHAR(CURRENT_DATE + (random() * 365 || ' days')::INTERVAL, 'MM/YY'),
            issuer_banks[floor(random() * array_length(issuer_banks, 1) + 1)],
            floor(random() * 1000 + 1)::INTEGER,
            floor(random() * 9999 + 1)::INTEGER,
            mcc_categories[floor(random() * array_length(mcc_categories, 1) + 1)],
            cities[floor(random() * array_length(cities, 1) + 1)],
            transaction_types[floor(random() * array_length(transaction_types, 1) + 1)],
            (random() * 100000 + 100)::NUMERIC(15, 2),
            CASE WHEN random() > 0.3 THEN (random() * 100000 + 100)::NUMERIC(15, 2) ELSE NULL END,
            currencies[floor(random() * array_length(currencies, 1) + 1)],
            countries[floor(random() * array_length(countries, 1) + 1)],
            pos_modes[floor(random() * (array_length(pos_modes, 1) + 1))],
            wallet_types[floor(random() * (array_length(wallet_types, 1) + 1))]
        );
    END LOOP;
END $$;
