#!/bin/bash

# Скрипт для добавления дополнительных транзакций в существующую базу данных
# Использование: ./scripts/add_more_transactions.sh [COUNT] [DB_NAME] [DB_USER] [DB_PASSWORD] [DB_HOST] [DB_PORT]

set -e

# Параметры по умолчанию
TRANSACTION_COUNT="${1:-2000000}"  # По умолчанию 1 миллион
DB_NAME="${2:-payment_analytics}"
DB_USER="${3:-postgres}"
DB_PASSWORD="${4:-postgres}"
DB_HOST="${5:-localhost}"
DB_PORT="${6:-5433}"

echo "📊 Добавление транзакций в базу данных"
echo "======================================"
echo "Количество транзакций: $TRANSACTION_COUNT"
echo "База данных: $DB_NAME"
echo "Пользователь: $DB_USER"
echo "Хост: $DB_HOST"
echo "Порт: $DB_PORT"
echo ""

# Проверяем наличие psql
if ! command -v psql &> /dev/null; then
    echo "❌ Ошибка: psql не найден. Установите PostgreSQL."
    exit 1
fi

# Устанавливаем переменную окружения для пароля
export PGPASSWORD="$DB_PASSWORD"

# Проверяем подключение к базе данных
echo "📡 Проверка подключения к базе данных..."
if ! psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" > /dev/null 2>&1; then
    echo "❌ Ошибка: Не удалось подключиться к базе данных '$DB_NAME'."
    echo "   Убедитесь, что база данных существует и доступна."
    exit 1
fi
echo "✅ Подключение успешно"

# Проверяем существование таблицы
echo ""
echo "🔍 Проверка таблицы transactions..."
TABLE_EXISTS=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'transactions');")
if [ "$TABLE_EXISTS" != " t" ]; then
    echo "❌ Ошибка: Таблица 'transactions' не существует."
    echo "   Сначала выполните: ./scripts/setup_database.sh"
    exit 1
fi
echo "✅ Таблица существует"

# Показываем текущее количество транзакций
CURRENT_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM transactions;")
echo "📊 Текущее количество транзакций: $CURRENT_COUNT"

# Определяем путь к скриптам
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INSERT_DATA_FILE="$PROJECT_ROOT/scripts/insert_test_data.sql"

if [ ! -f "$INSERT_DATA_FILE" ]; then
    echo "❌ Ошибка: Файл не найден: $INSERT_DATA_FILE"
    exit 1
fi

# Добавляем транзакции
echo ""
echo "📈 Добавление $TRANSACTION_COUNT транзакций..."
echo "   Это может занять некоторое время..."
echo "   Для 1 миллиона записей обычно требуется 5-15 минут в зависимости от производительности сервера"

# Используем переменную PostgreSQL для передачи количества транзакций
# Передаем переменную без кавычек - psql сам обработает её как число
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    -v transaction_count="$TRANSACTION_COUNT" \
    -f "$INSERT_DATA_FILE"

# Проверяем результат
echo ""
echo "🔍 Проверка результата..."
NEW_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM transactions;")
ADDED_COUNT=$((NEW_COUNT - CURRENT_COUNT))

echo "✅ Готово!"
echo "   Было транзакций: $CURRENT_COUNT"
echo "   Добавлено транзакций: $ADDED_COUNT"
echo "   Всего транзакций: $NEW_COUNT"

# Очищаем переменную окружения
unset PGPASSWORD

