#!/usr/bin/env python3
"""
Простой скрипт для тестирования Payment Analytics API
Использование: python3 test_api.py
"""

import requests
import json
import sys

API_URL = "http://localhost:3000/api"

def test_health():
    """Проверка health endpoint"""
    print("🔍 Health Check:")
    try:
        response = requests.get(f"{API_URL}/health")
        response.raise_for_status()
        data = response.json()
        print(f"✅ Status: {data['status']}")
        print(f"   Database: {data['database']}")
        print(f"   LLM: {data['llm']}")
        return True
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        print("   Убедитесь, что сервер запущен: cargo run")
        return False

def test_query(question):
    """Отправка запроса к API"""
    print(f"\n📝 Вопрос: {question}")
    try:
        response = requests.post(
            f"{API_URL}/query",
            json={"question": question},
            timeout=30
        )
        response.raise_for_status()
        data = response.json()
        
        print(f"✅ SQL: {data['sql']}")
        print(f"   Результатов: {data['row_count']}")
        print(f"   Время выполнения: {data['execution_time_ms']}ms")
        print(f"\n   Данные:")
        print(json.dumps(data['data'], indent=2, ensure_ascii=False))
        return True
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        if hasattr(e, 'response'):
            print(f"   Ответ сервера: {e.response.text}")
        return False

def main():
    print("=" * 60)
    print("🧪 Тестирование Payment Analytics API")
    print("=" * 60)
    
    # Проверка health
    if not test_health():
        sys.exit(1)
    
    # Тестовые запросы
    questions = [
        "Сколько всего транзакций в базе?",
        "Топ 5 мерчантов по объему транзакций",
        "Сколько транзакций было сегодня?",
        "Объем транзакций по категориям MCC",
    ]
    
    for question in questions:
        test_query(question)
        print("\n" + "-" * 60)
    
    print("\n✅ Тестирование завершено!")

if __name__ == "__main__":
    main()

