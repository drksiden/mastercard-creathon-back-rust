// Язык определяется в api/query.rs, здесь не нужен

/// Определяет, является ли вопрос запросом к базе данных или обычным вопросом
/// Также проверяет наличие префиксов для явного указания SQL-запроса
pub fn is_database_query(question: &str) -> bool {
    let question_lower = question.to_lowercase().trim().to_string();
    
    // Проверяем префикс для явного указания SQL-запроса (только sql:)
    if question_lower.starts_with("sql:") {
        return true;
    }
    
    // Убираем префикс для дальнейшего анализа
    let question_clean = if question_lower.starts_with("sql:") {
        question_lower.split_once(':').map(|(_, rest)| rest.trim().to_string()).unwrap_or(question_lower)
    } else {
        question_lower
    };
    
    let question_lower = question_clean.as_str();
    
    // Ключевые слова, указывающие на SQL-запрос
    let db_keywords = [
        // Вопросы о данных
        "сколько", "how many", "қанша",
        "топ", "top", "көп",
        "объем", "volume", "көлем",
        "средний", "average", "орташа",
        "сумма", "sum", "қосынды",
        "максимум", "maximum", "максимум",
        "минимум", "minimum", "минимум",
        "показать", "show", "көрсет",
        "вывести", "display", "шығар",
        "найти", "find", "тап",
        "получить", "get", "алу",
        
        // Временные периоды
        "сегодня", "today", "бүгін",
        "вчера", "yesterday", "кеше",
        "неделя", "week", "апта",
        "месяц", "month", "ай",
        "год", "year", "жыл",
        "день", "day", "күн",
        "дни", "days", "күндер",
        "транзакции", "transactions", "транзакциялар",
        "транзакций", "транзакция",
        "за последние", "last", "соңғы",
        "за период", "for period", "кезең үшін",
        "динамик", "dynamics", "динамик", // "динамик" покрывает "динамика", "динамику", "динамики"
        "динамика", "динамику", "динамики", // Добавляем все формы
        "тренд", "trend", "тренд",
        "изменение", "change", "өзгеріс",
        "изменения", "changes", "өзгерістер",
        
        // Категории и фильтры
        "по категориям", "by category", "категориялар бойынша",
        "по городам", "by city", "қалалар бойынша",
        "по банкам", "by bank", "банктер бойынша",
        "по валютам", "by currency", "валюталар бойынша",
        "по дням", "by day", "күндер бойынша",
        "по месяцам", "by month", "айлар бойынша",
        "по годам", "by year", "жылдар бойынша",
        "мерчант", "merchant", "мерчант",
        "карт", "card", "карта",
        
        // Аналитика
        "распределение", "distribution", "таралу",
        "сравнение", "compare", "салыстыру",
        "статистика", "statistics", "статистика",
        "анализ", "analysis", "талдау",
        "график", "chart", "график",
        "диаграмма", "diagram", "диаграмма",
        
        // Конкретные поля БД
        "mcc", "mcc_category",
        "transaction_type", "тип транзакции",
        "transaction_currency", "валюта транзакции",
        "acquirer_country", "страна",
        "pos_entry_mode", "способ оплаты",
    ];
    
    // Ключевые слова, указывающие на обычный вопрос (не про БД) - ПРИОРИТЕТ
    let chat_keywords = [
        "привет", "hello", "сәлем", "hi", "hey",
        "как дела", "how are you", "қалың қалай", "how's it going",
        "кто ты", "who are you", "сен кімсің", "what are you",
        "что умеешь", "what can you do", "не істей аласың", "what can you",
        "помощь", "help", "көмек",
        "спасибо", "thanks", "рахмет", "thank you",
        "пока", "bye", "сау бол", "goodbye",
        "можешь", "can you", "can", "можешь ли", "але сен",
        "вывести", "показать", "show", "display", "көрсет", "шығар", // Но только в контексте общих вопросов
        "или нет", "or not", "немесе жоқ", // Вопросы типа "можешь или нет"
        "расскажи", "tell me", "айт", "explain", "объясни", "түсіндір",
        "что такое", "what is", "не дегеніміз", "что значит", "what does",
    ];
    
        // Проверяем на обычные вопросы (ВЫСОКИЙ ПРИОРИТЕТ)
        for keyword in &chat_keywords {
            if question_lower.contains(keyword) {
                // Если это вопрос типа "можешь вывести" или "можешь показать" БЕЗ упоминания транзакций/данных - обычный вопрос
                if (*keyword == "можешь" || *keyword == "can you" || *keyword == "can") && 
                   (question_lower.contains("вывести") || question_lower.contains("показать") || 
                    question_lower.contains("show") || question_lower.contains("display")) {
                    // Проверяем, есть ли упоминание транзакций или данных
                    if !question_lower.contains("транзакц") && !question_lower.contains("transaction") && 
                       !question_lower.contains("данн") && !question_lower.contains("data") &&
                       !question_lower.contains("баз") && !question_lower.contains("database") {
                        return false; // Это обычный вопрос, не SQL
                    }
                }
                // Для остальных ключевых слов - точно обычный вопрос
                if *keyword != "вывести" && *keyword != "показать" && *keyword != "show" && *keyword != "display" {
                    return false;
                }
            }
        }
    
    // Проверяем на SQL-запросы (требуется больше доказательств)
    let mut db_score = 0;
    for keyword in &db_keywords {
        if question_lower.contains(keyword) {
            db_score += 1;
        }
    }
    
    // Специальные паттерны для SQL-запросов (требуют комбинации с другими ключевыми словами)
    let sql_patterns = [
        "динамик", "dynamics", "тренд",
        "за последние", "last", "соңғы",
        "по дням", "by day", "күндер бойынша",
        "по месяцам", "by month", "айлар бойынша",
        "статистика", "statistics", "статистика",
        "распределение", "distribution", "таралу",
    ];
    
    let mut has_sql_pattern = false;
    for pattern in &sql_patterns {
        if question_lower.contains(pattern) {
            has_sql_pattern = true;
            db_score += 2; // Даем больший вес паттернам
            break;
        }
    }
    
    // Если вопрос содержит "транзакции" + временной период ИЛИ агрегацию - точно SQL
    if question_lower.contains("транзакц") && (
        question_lower.contains("день") ||
        question_lower.contains("месяц") ||
        question_lower.contains("год") ||
        question_lower.contains("неделя") ||
        question_lower.contains("сегодня") ||
        question_lower.contains("вчера") ||
        question_lower.contains("последние") ||
        question_lower.contains("сколько") ||
        question_lower.contains("топ") ||
        question_lower.contains("объем") ||
        question_lower.contains("средний")
    ) {
        return true;
    }
    
    // Если вопрос очень короткий (1-3 слова) - скорее всего обычный вопрос
    let word_count = question.split_whitespace().count();
    if word_count <= 3 && db_score <= 1 {
        return false;
    }
    
    // Вопросительные слова БЕЗ ключевых слов БД - обычный вопрос
    let question_words = ["что", "what", "не", "кто", "who", "кім", "как", "how", "қалай", "почему", "why", "неге"];
    let has_question_word = question_words.iter().any(|w| question_lower.contains(w));
    
    // Если есть вопросительное слово и мало ключевых слов БД - обычный вопрос
    if has_question_word && db_score <= 1 && !has_sql_pattern {
        return false;
    }
    
    // Если db_score достаточно высокий (>= 2) - SQL запрос
    if db_score >= 2 {
        return true;
    }
    
    // По умолчанию - обычный вопрос (менее агрессивная классификация)
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_queries() {
        assert!(is_database_query("Сколько транзакций было сегодня?"));
        assert!(is_database_query("Топ 10 городов по объему транзакций"));
        assert!(is_database_query("Средний чек для карт Halyk Bank"));
        assert!(is_database_query("Объем транзакций по категориям"));
        assert!(is_database_query("How many transactions were today?"));
    }

    #[test]
    fn test_chat_queries() {
        assert!(!is_database_query("Привет"));
        assert!(!is_database_query("Как дела?"));
        assert!(!is_database_query("Кто ты?"));
        assert!(!is_database_query("Что умеешь?"));
        assert!(!is_database_query("Hello"));
        assert!(!is_database_query("Спасибо"));
    }

    #[test]
    fn test_edge_cases() {
        // Вопросы, которые могут быть и тем и другим
        assert!(is_database_query("Что такое транзакции?")); // Скорее SQL, т.к. есть "транзакции"
        assert!(!is_database_query("Что такое жизнь?")); // Обычный вопрос
    }

    #[test]
    fn test_dynamics_queries() {
        assert!(is_database_query("Показать динамику транзакций по дням за последние 7 дней"));
        assert!(is_database_query("Динамика транзакций по месяцам"));
        assert!(is_database_query("Показать изменения транзакций за период"));
        assert!(is_database_query("Тренд транзакций по дням"));
    }
}

