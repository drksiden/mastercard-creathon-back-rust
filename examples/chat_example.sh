#!/bin/bash

# Примеры использования Chat API

API_URL="http://localhost:3000/api/chat"

echo "=== Пример 1: Первое сообщение (создание сессии) ==="
RESPONSE1=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Привет! Расскажи о себе"
  }')

echo "$RESPONSE1" | jq '.'
SESSION_ID=$(echo "$RESPONSE1" | jq -r '.session_id')
echo "Session ID: $SESSION_ID"
echo ""

echo "=== Пример 2: Продолжение разговора (с контекстом) ==="
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"Как получить данные о транзакциях?\",
    \"session_id\": \"$SESSION_ID\"
  }" | jq '.'
echo ""

echo "=== Пример 3: Вопрос на английском ==="
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello! What can you help me with?",
    "session_id": ""
  }' | jq '.'
echo ""

echo "=== Пример 4: Вопрос на казахском ==="
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Сәлем! Сен не істей аласың?",
    "session_id": ""
  }' | jq '.'
echo ""

echo "=== Пример 5: Общий вопрос с контекстом ==="
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"Спасибо за информацию!\",
    \"session_id\": \"$SESSION_ID\"
  }" | jq '.'

