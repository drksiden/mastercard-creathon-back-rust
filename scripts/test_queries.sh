#!/bin/bash

# Тестовый скрипт для проверки API с различными запросами
# Проверяет: таблицы, диаграммы, анализ

API_URL="http://localhost:3000/api/query"

echo "🧪 Тестирование Payment Analytics API"
echo "======================================"
echo ""

# Функция для форматированного вывода
print_test() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 Тест: $1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Функция для вывода анализа
print_analysis() {
    echo ""
    echo "📝 Анализ:"
    echo "  Headline: $(echo "$1" | jq -r '.analysis.headline // "N/A"')"
    echo "  Insights: $(echo "$1" | jq -r '.analysis.insights | length')"
    echo "  Chart Type: $(echo "$1" | jq -r '.analysis.chart_type // "N/A"')"
    echo "  Explanation: $(echo "$1" | jq -r '.analysis.explanation[0:100] // "N/A"')..."
}

# Функция для вывода таблицы
print_table() {
    local data="$1"
    local count=$(echo "$data" | jq '.data | length')
    echo ""
    echo "📋 Таблица данных ($count строк):"
    echo "$data" | jq -r '.data[0:5] | .[] | to_entries | map("\(.key): \(.value)") | join(" | ")' | head -5
    if [ "$count" -gt 5 ]; then
        echo "... и еще $((count - 5)) строк"
    fi
}

# Тест 1: Простой запрос (COUNT)
print_test "1. Простой запрос - количество транзакций"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько всего транзакций?",
    "include_analysis": true
  }')

echo "$RESPONSE" | jq '{question, row_count, execution_time_ms, cached}'
print_analysis "$RESPONSE"
echo "$RESPONSE" | jq '.data'

# Тест 2: Запрос для таблицы (TOP N)
print_test "2. Запрос для таблицы - Топ-5 категорий MCC"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Показать топ-5 категорий MCC по количеству транзакций",
    "include_analysis": true
  }')

echo "$RESPONSE" | jq '{question, row_count, execution_time_ms, chart_type: .analysis.chart_type}'
print_analysis "$RESPONSE"
print_table "$RESPONSE"

# Тест 3: Запрос для Bar диаграммы
print_test "3. Запрос для Bar диаграммы - Транзакции по типам"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько транзакций каждого типа? Покажи по типам транзакций",
    "include_analysis": true
  }')

echo "$RESPONSE" | jq '{question, row_count, chart_type: .analysis.chart_type}'
print_analysis "$RESPONSE"
echo ""
echo "📊 Данные для Bar диаграммы:"
echo "$RESPONSE" | jq '.data'

# Тест 4: Запрос для Line диаграммы (временной ряд)
print_test "4. Запрос для Line диаграммы - Динамика по дням"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Показать динамику транзакций по дням за последние 7 дней",
    "include_analysis": true
  }')

echo "$RESPONSE" | jq '{question, row_count, chart_type: .analysis.chart_type}'
print_analysis "$RESPONSE"
echo ""
echo "📈 Данные для Line диаграммы:"
echo "$RESPONSE" | jq '.data'

# Тест 5: Запрос для Pie диаграммы (доли)
print_test "5. Запрос для Pie диаграммы - Доли по валютам"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Показать распределение транзакций по валютам",
    "include_analysis": true
  }')

echo "$RESPONSE" | jq '{question, row_count, chart_type: .analysis.chart_type}'
print_analysis "$RESPONSE"
echo ""
echo "🥧 Данные для Pie диаграммы:"
echo "$RESPONSE" | jq '.data'

# Тест 6: Сложный аналитический запрос
print_test "6. Сложный запрос - Анализ по городам"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-10 городов по объему транзакций в KZT",
    "include_analysis": true
  }')

echo "$RESPONSE" | jq '{question, row_count, chart_type: .analysis.chart_type}'
print_analysis "$RESPONSE"
print_table "$RESPONSE"

# Тест 7: Запрос с кэшированием
print_test "7. Тест кэширования"
echo "Первый запрос (будет закэширован):"
RESPONSE1=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько транзакций типа POS?",
    "include_analysis": false,
    "use_cache": true
  }')
echo "$RESPONSE1" | jq '{cached, execution_time_ms}'

echo ""
echo "Второй запрос (из кэша):"
RESPONSE2=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько транзакций типа POS?",
    "include_analysis": false,
    "use_cache": true
  }')
echo "$RESPONSE2" | jq '{cached, execution_time_ms}'

# Тест 8: Проверка формата данных для фронтенда
print_test "8. Формат данных для фронтенда"
RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-5 категорий MCC",
    "include_analysis": true
  }')

echo "✅ Структура ответа:"
echo "$RESPONSE" | jq '{
  has_question: (.question != null),
  has_sql: (.sql != null),
  has_data: (.data != null),
  has_analysis: (.analysis != null),
  analysis_has_headline: (.analysis.headline != null),
  analysis_has_insights: (.analysis.insights != null),
  analysis_has_chart_type: (.analysis.chart_type != null),
  data_count: (.data | length)
}'

echo ""
echo "📊 Пример данных для визуализации:"
echo "$RESPONSE" | jq '.data[0:3]'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Тестирование завершено!"
echo ""
echo "💡 Рекомендации:"
echo "  - Для таблиц: используйте .data напрямую"
echo "  - Для диаграмм: используйте .analysis.chart_type для типа и .data для данных"
echo "  - Для текста: используйте .analysis.headline, .analysis.explanation"
echo "  - Для инсайтов: используйте .analysis.insights"

