-- Скрипт для заполнения базы данных тестовыми данными
-- Использование: psql -U postgres -d payment_analytics -f scripts/insert_test_data.sql
-- Или: psql postgresql://user:password@host:port/dbname -f scripts/insert_test_data.sql
-- 
-- Для указания количества транзакций используйте переменную:
-- psql -v transaction_count=1000000 -d payment_analytics -f scripts/insert_test_data.sql

-- Очищаем существующие данные (опционально)
-- TRUNCATE TABLE transactions CASCADE;

-- Создаем временную таблицу для передачи значения из psql переменной в DO блок
-- Используем условную компиляцию psql для проверки наличия переменной
-- Если переменная передана через -v transaction_count=значение, она доступна как :transaction_count
DROP TABLE IF EXISTS temp_transaction_count;
\if :{?transaction_count}
    -- Переменная передана, создаем временную таблицу с этим значением
    CREATE TEMP TABLE temp_transaction_count AS SELECT :'transaction_count'::INTEGER AS count;
\else
    -- Переменная не передана, используем значение по умолчанию
    CREATE TEMP TABLE temp_transaction_count AS SELECT 5000::INTEGER AS count;
\endif

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
    transaction_count INTEGER;
    existing_count INTEGER;
    start_id INTEGER;
    batch_size INTEGER := 50000;  -- Размер батча для больших объемов
BEGIN
    -- Получаем количество транзакций из временной таблицы
    -- Значение было установлено перед этим блоком через временную таблицу
    SELECT count INTO transaction_count FROM temp_transaction_count;
    
    -- Проверяем, что transaction_count валидное число
    IF transaction_count IS NULL OR transaction_count <= 0 THEN
        RAISE NOTICE 'Значение transaction_count невалидно, используем значение по умолчанию: 5000';
        transaction_count := 5000;
    END IF;
    
    -- Получаем текущее количество транзакций и максимальный ID
    SELECT COALESCE(MAX(id), 0), COUNT(*) INTO start_id, existing_count FROM transactions;
    
    RAISE NOTICE 'Текущее количество транзакций: %', existing_count;
    RAISE NOTICE 'Будет добавлено транзакций: %', transaction_count;
    RAISE NOTICE 'Начальный ID: %', start_id;
    
    RAISE NOTICE 'Генерация % транзакций...', transaction_count;
    
    -- Для больших объемов (более 10000) используем оптимизированный batch insert
    IF transaction_count > 10000 THEN
        RAISE NOTICE 'Используется оптимизированный метод для больших объемов (батчи по %)...', batch_size;
        
        -- Генерируем транзакции батчами
        DECLARE
            batch_num INTEGER := 0;
            total_batches INTEGER := (transaction_count + batch_size - 1) / batch_size;
            current_batch_size INTEGER;
            batch_start_id INTEGER;
        BEGIN
            WHILE batch_num < total_batches LOOP
                current_batch_size := LEAST(batch_size, transaction_count - batch_num * batch_size);
                
                IF current_batch_size <= 0 THEN
                    EXIT;
                END IF;
                
                batch_start_id := start_id + batch_num * batch_size;
                
                -- Используем INSERT с SELECT для батча (более эффективно)
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
                )
                SELECT
                    'TXN' || LPAD((batch_start_id + row_number() OVER ())::TEXT, 10, '0'),
                    NOW() - (random() * 365 * 2 || ' days')::INTERVAL,
                    floor(random() * 1000000 + 1)::INTEGER,
                    TO_CHAR(CURRENT_DATE + (random() * 365 || ' days')::INTERVAL, 'MM/YY'),
                    issuer_banks[1 + floor(random() * array_length(issuer_banks, 1))::INTEGER],
                    floor(random() * 1000 + 1)::INTEGER,
                    floor(random() * 9999 + 1)::INTEGER,
                    mcc_categories[1 + floor(random() * array_length(mcc_categories, 1))::INTEGER],
                    cities[1 + floor(random() * array_length(cities, 1))::INTEGER],
                    transaction_types[1 + floor(random() * array_length(transaction_types, 1))::INTEGER],
                    (random() * 100000 + 100)::NUMERIC(15, 2),
                    CASE WHEN random() > 0.3 THEN (random() * 100000 + 100)::NUMERIC(15, 2) ELSE NULL END,
                    currencies[1 + floor(random() * array_length(currencies, 1))::INTEGER],
                    countries[1 + floor(random() * array_length(countries, 1))::INTEGER],
                    CASE 
                        WHEN random() > 0.2 THEN pos_modes[1 + floor(random() * (array_length(pos_modes, 1) - 1))::INTEGER]
                        ELSE NULL
                    END,
                    CASE 
                        WHEN random() > 0.25 THEN wallet_types[1 + floor(random() * (array_length(wallet_types, 1) - 1))::INTEGER]
                        ELSE NULL
                    END
                FROM generate_series(1, current_batch_size);
                
                batch_num := batch_num + 1;
                
                -- Показываем прогресс каждые 5 батчей или в конце
                IF batch_num % 5 = 0 OR batch_num = total_batches THEN
                    RAISE NOTICE 'Обработано батчей: %/% (добавлено транзакций: ~%)', 
                        batch_num, total_batches, batch_num * batch_size;
                END IF;
            END LOOP;
        END;
    ELSE
        -- Для небольших объемов используем цикл
        FOR i IN 1..transaction_count LOOP
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
                'TXN' || LPAD((start_id + i)::TEXT, 10, '0'),
                NOW() - (random() * 365 * 2 || ' days')::INTERVAL,
                floor(random() * 1000000 + 1)::INTEGER,
                TO_CHAR(CURRENT_DATE + (random() * 365 || ' days')::INTERVAL, 'MM/YY'),
                issuer_banks[1 + floor(random() * array_length(issuer_banks, 1))::INTEGER],
                floor(random() * 1000 + 1)::INTEGER,
                floor(random() * 9999 + 1)::INTEGER,
                mcc_categories[1 + floor(random() * array_length(mcc_categories, 1))::INTEGER],
                cities[1 + floor(random() * array_length(cities, 1))::INTEGER],
                transaction_types[1 + floor(random() * array_length(transaction_types, 1))::INTEGER],
                (random() * 100000 + 100)::NUMERIC(15, 2),
                CASE WHEN random() > 0.3 THEN (random() * 100000 + 100)::NUMERIC(15, 2) ELSE NULL END,
                currencies[1 + floor(random() * array_length(currencies, 1))::INTEGER],
                countries[1 + floor(random() * array_length(countries, 1))::INTEGER],
                CASE 
                    WHEN random() > 0.2 THEN pos_modes[1 + floor(random() * (array_length(pos_modes, 1) - 1))::INTEGER]
                    ELSE NULL
                END,
                CASE 
                    WHEN random() > 0.25 THEN wallet_types[1 + floor(random() * (array_length(wallet_types, 1) - 1))::INTEGER]
                    ELSE NULL
                END
            );
            
            -- Показываем прогресс каждые 1000 транзакций
            IF i % 1000 = 0 THEN
                RAISE NOTICE 'Сгенерировано % транзакций...', i;
            END IF;
        END LOOP;
    END IF;
    
    RAISE NOTICE 'Генерация завершена! Добавлено транзакций: %', transaction_count;
END $$;

-- Показываем статистику
SELECT 
    'Статистика по транзакциям' as info,
    COUNT(*) as total_transactions,
    COUNT(DISTINCT transaction_type) as transaction_types,
    COUNT(DISTINCT mcc_category) as mcc_categories,
    COUNT(DISTINCT transaction_currency) as currencies,
    COUNT(DISTINCT acquirer_country_iso) as countries,
    MIN(transaction_timestamp) as earliest_transaction,
    MAX(transaction_timestamp) as latest_transaction,
    SUM(transaction_amount_kzt) as total_amount_kzt
FROM transactions;
