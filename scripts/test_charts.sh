#!/bin/bash

# Специальный тест для проверки данных диаграмм

API_URL="http://localhost:3000/api/query"

echo "📊 Тестирование данных для диаграмм"
echo "===================================="
echo ""

test_chart() {
    local name="$1"
    local question="$2"
    local expected_type="$3"
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    RESPONSE=$(curl -s -X POST "$API_URL" \
      -H "Content-Type: application/json" \
      -d "{
        \"question\": \"$question\",
        \"include_analysis\": true
      }")
    
    CHART_TYPE=$(echo "$RESPONSE" | jq -r '.analysis.chart_type // "null"')
    ROW_COUNT=$(echo "$RESPONSE" | jq '.data | length')
    
    echo "Вопрос: $question"
    echo "Рекомендуемый тип диаграммы: $CHART_TYPE (ожидался: $expected_type)"
    echo "Количество строк данных: $ROW_COUNT"
    echo ""
    
    if [ "$CHART_TYPE" != "null" ] && [ "$CHART_TYPE" != "" ]; then
        echo "✅ Тип диаграммы определен: $CHART_TYPE"
    else
        echo "⚠️  Тип диаграммы не определен"
    fi
    
    echo ""
    echo "📋 Данные (первые 5 строк):"
    echo "$RESPONSE" | jq '.data[0:5]'
    
    echo ""
    echo "📝 Headline:"
    echo "$RESPONSE" | jq -r '.analysis.headline'
    
    echo ""
    echo "💡 Первый инсайт:"
    echo "$RESPONSE" | jq -r '.analysis.insights[0].title // "Нет инсайтов"'
    echo "$RESPONSE" | jq -r '.analysis.insights[0].description // ""'
}

# Тест Bar диаграммы
test_chart "Bar Chart - Транзакции по типам" \
  "Сколько транзакций каждого типа? Покажи POS, ATM_WITHDRAWAL, ECOM" \
  "Bar"

# Тест Line диаграммы
test_chart "Line Chart - Динамика по дням" \
  "Показать количество транзакций по дням за последнюю неделю" \
  "Line"

# Тест Pie диаграммы
test_chart "Pie Chart - Распределение по валютам" \
  "Показать долю каждой валюты в общем объеме транзакций" \
  "Pie"

# Тест Table
test_chart "Table - Детальная таблица" \
  "Показать топ-10 городов с количеством и суммой транзакций" \
  "Table"

# Тест Trend
test_chart "Trend Chart - Тренд по месяцам" \
  "Показать тренд транзакций по месяцам за последний год" \
  "Trend"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Тестирование завершено!"
echo ""
echo "📊 Формат данных для диаграмм:"
echo "  - Bar/Line: массив объектов с ключами для осей"
echo "  - Pie: массив объектов с названием и значением"
echo "  - Table: массив объектов с колонками"
echo ""
echo "💡 Использование на фронтенде:"
echo "  const chartType = response.analysis.chart_type;"
echo "  const chartData = response.data;"
echo "  // Используйте Recharts/Chart.js для построения"

