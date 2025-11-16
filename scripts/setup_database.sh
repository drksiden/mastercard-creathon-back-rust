#!/bin/bash

# Скрипт для создания базы данных PostgreSQL и заполнения тестовыми данными
# Использование: ./scripts/setup_database.sh [DB_NAME] [DB_USER] [DB_PASSWORD] [DB_HOST] [DB_PORT]

set -e

# Параметры по умолчанию
DB_NAME="${1:-payment_analytics}"
DB_USER="${2:-postgres}"
DB_PASSWORD="${3:-postgres}"
DB_HOST="${4:-localhost}"
DB_PORT="${5:-5432}"

echo "🗄️  Настройка базы данных PostgreSQL"
echo "=================================="
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

# Проверяем подключение к PostgreSQL
echo "📡 Проверка подключения к PostgreSQL..."
if ! psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "SELECT 1;" > /dev/null 2>&1; then
    echo "❌ Ошибка: Не удалось подключиться к PostgreSQL."
    echo "   Убедитесь, что PostgreSQL запущен и доступен."
    exit 1
fi
echo "✅ Подключение успешно"

# Создаем базу данных (если не существует)
echo ""
echo "📦 Создание базы данных '$DB_NAME'..."
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME'" | grep -q 1; then
    echo "⚠️  База данных '$DB_NAME' уже существует"
    read -p "Удалить и пересоздать? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🗑️  Удаление существующей базы данных..."
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;"
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $DB_NAME;"
        echo "✅ База данных пересоздана"
    else
        echo "ℹ️  Используем существующую базу данных"
    fi
else
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $DB_NAME;"
    echo "✅ База данных создана"
fi

# Применяем миграции
echo ""
echo "📋 Применение миграций..."

# Определяем путь к скриптам
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Применяем скрипт создания таблиц
CREATE_DB_FILE="$PROJECT_ROOT/scripts/create_database.sql"
if [ ! -f "$CREATE_DB_FILE" ]; then
    echo "❌ Ошибка: Файл не найден: $CREATE_DB_FILE"
    exit 1
fi

psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$CREATE_DB_FILE"
echo "✅ Таблицы созданы"

# Заполняем тестовыми данными
echo ""
echo "📊 Заполнение тестовыми данными..."
INSERT_DATA_FILE="$PROJECT_ROOT/scripts/insert_test_data.sql"
if [ ! -f "$INSERT_DATA_FILE" ]; then
    echo "❌ Ошибка: Файл не найден: $INSERT_DATA_FILE"
    exit 1
fi

psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$INSERT_DATA_FILE"
echo "✅ Тестовые данные добавлены"

# Проверяем количество транзакций
echo ""
echo "🔍 Проверка данных..."
TRANSACTION_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM transactions;")
echo "✅ В базе данных: $TRANSACTION_COUNT транзакций"

# Выводим информацию о подключении
echo ""
echo "✅ База данных готова к использованию!"
echo ""
echo "📝 Для подключения используйте:"
echo "   DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
echo ""
echo "   Или добавьте в .env файл:"
echo "   DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

# Очищаем переменную окружения
unset PGPASSWORD

