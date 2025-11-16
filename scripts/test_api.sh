#!/bin/bash

# Тестирование Payment Analytics Backend API

BASE_URL="http://localhost:3000"

echo "🧪 Тестирование Payment Analytics Backend API"
echo "=============================================="
echo ""

# 1. Health Check
echo "1️⃣  Проверка Health Check..."
curl -s "$BASE_URL/api/health" | jq '.' || echo "❌ Health check failed"
echo ""
echo "---"
echo ""

# 2. Простой запрос
echo "2️⃣  Простой запрос: Сколько всего транзакций?"
curl -s -X POST "$BASE_URL/api/query" \
  -H "Content-Type: application/json" \
  -d '{"question": "Сколько всего транзакций?"}' | jq '.' || echo "❌ Query failed"
echo ""
echo "---"
echo ""

# 3. Запрос с фильтрацией
echo "3️⃣  Запрос с фильтрацией: Топ 5 мерчантов по объему"
curl -s -X POST "$BASE_URL/api/query" \
  -H "Content-Type: application/json" \
  -d '{"question": "Топ 5 мерчантов по объему транзакций"}' | jq '.' || echo "❌ Query failed"
echo ""
echo "---"
echo ""

# 4. Запрос по категориям
echo "4️⃣  Запрос по категориям: Объем транзакций по категориям MCC"
curl -s -X POST "$BASE_URL/api/query" \
  -H "Content-Type: application/json" \
  -d '{"question": "Объем транзакций по категориям MCC"}' | jq '.' || echo "❌ Query failed"
echo ""

echo "✅ Тестирование завершено!"
