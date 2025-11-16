use serde_json::Value;
use crate::api::models::{ChartData, ChartDataset};

/// Форматирует данные в таблицу (Markdown)
pub fn format_as_table(data: &[Value]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    
    // Получаем все ключи из первой строки
    if let Some(first_obj) = data[0].as_object() {
        let keys: Vec<String> = first_obj.keys().map(|k| k.clone()).collect();
        
        if keys.is_empty() {
            return String::new();
        }
        
        // Формируем заголовок
        result.push_str("| ");
        for key in &keys {
            result.push_str(key);
            result.push_str(" | ");
        }
        result.push_str("\n");
        
        // Разделитель
        result.push_str("|");
        for _ in &keys {
            result.push_str(" --- |");
        }
        result.push_str("\n");

        // Формируем строки данных
        for row in data {
            if let Some(obj) = row.as_object() {
                result.push_str("| ");
                for key in &keys {
                    let value = obj.get(key)
                        .and_then(|v| {
                            if v.is_number() {
                                Some(format!("{:.2}", v.as_f64().unwrap_or(0.0)))
                            } else {
                                v.as_str().map(|s| s.to_string())
                            }
                        })
                        .unwrap_or_else(|| "N/A".to_string());
                    
                    result.push_str(&value);
                    result.push_str(" | ");
                }
                result.push_str("\n");
            }
        }
    }

    result
}

/// Форматирует данные в CSV
pub fn format_as_csv(data: &[Value]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    
    if let Some(first_obj) = data[0].as_object() {
        let keys: Vec<String> = first_obj.keys().map(|k| k.clone()).collect();
        
        // Заголовок
        result.push_str(&keys.join(","));
        result.push_str("\n");
        
        // Данные
        for row in data {
            if let Some(obj) = row.as_object() {
                let values: Vec<String> = keys.iter()
                    .map(|key| {
                        let value = obj.get(key)
                            .and_then(|v| {
                                if v.is_number() {
                                    Some(format!("{}", v.as_f64().unwrap_or(0.0)))
                                } else {
                                    v.as_str().map(|s| format!("\"{}\"", s.replace("\"", "\"\"")))
                                }
                            })
                            .unwrap_or_else(|| "".to_string());
                        value
                    })
                    .collect();
                result.push_str(&values.join(","));
                result.push_str("\n");
            }
        }
    }

    result
}

// Генерация изображений диаграмм реализована только в telegram_bot
// Основной бэкенд возвращает только данные для диаграмм (ChartData)

/// Преобразует данные в формат для диаграмм
pub fn format_as_chart_data(data: &[Value], chart_type: &str) -> Option<ChartData> {
    if data.is_empty() {
        return None;
    }

    // Определяем тип диаграммы на основе данных
    let detected_type = if chart_type == "auto" {
        detect_chart_type(data)
    } else {
        chart_type.to_string()
    };

    match detected_type.as_str() {
        "bar" | "line" | "trend" => {
            // Для bar/line нужны категории и значения
            if let Some(first_obj) = data[0].as_object() {
                // Ищем ключи для категорий и значений
                let keys: Vec<&String> = first_obj.keys().collect();
                
                // Первый нечисловой ключ - категория, первый числовой - значение
                // Приоритет: временные ключи (date, month, time) для категорий
                let category_key = keys.iter()
                    .find(|k| {
                        let k_lower = k.to_lowercase();
                        k_lower.contains("date") || k_lower.contains("month") || k_lower.contains("time") ||
                        k_lower.contains("день") || k_lower.contains("месяц") || k_lower.contains("год")
                    })
                    .or_else(|| {
                        keys.iter()
                            .find(|k| {
                                first_obj.get(**k)
                                    .and_then(|v| v.as_str().or_else(|| {
                                        // Также проверяем timestamp как строку
                                        v.as_i64().or_else(|| v.as_f64().map(|x| x as i64))
                                            .map(|_| "timestamp")
                                    }))
                                    .is_some()
                            })
                    });
                
                let value_key = keys.iter()
                    .find(|k| {
                        first_obj.get(**k)
                            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                            .is_some()
                    });

                if let (Some(cat_key), Some(val_key)) = (category_key, value_key) {
                    let labels: Vec<String> = data.iter()
                        .filter_map(|row| {
                            let obj = row.as_object()?;
                            let val = obj.get(&**cat_key)?;
                            // Обрабатываем timestamp как строку или число
                            if let Some(s) = val.as_str() {
                                Some(s.to_string())
                            } else if let Some(ts) = val.as_i64() {
                                // Преобразуем timestamp в дату
                                Some(format!("{}", ts))
                            } else if let Some(ts) = val.as_f64() {
                                Some(format!("{}", ts as i64))
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    let values: Vec<f64> = data.iter()
                        .filter_map(|row| {
                            row.as_object()?.get(&**val_key)
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                        })
                        .collect();

                    if !labels.is_empty() && !values.is_empty() {
                        // Для trend используем line тип
                        let final_type = if detected_type == "trend" { 
                            "line".to_string() 
                        } else { 
                            detected_type.to_string() 
                        };
                        return Some(ChartData {
                            chart_type: final_type,
                            labels,
                            datasets: vec![ChartDataset {
                                label: val_key.to_string(),
                                data: values,
                                background_color: None,
                            }],
                            title: None,
                        });
                    }
                }
            }
        }
        "pie" => {
            // Для pie нужны метки и значения
            if let Some(first_obj) = data[0].as_object() {
                let keys: Vec<&String> = first_obj.keys().collect();
                
                let label_key = keys.iter()
                    .find(|k| {
                        first_obj.get(**k)
                            .and_then(|v| v.as_str())
                            .is_some()
                    });
                
                let value_key = keys.iter()
                    .find(|k| {
                        first_obj.get(**k)
                            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                            .is_some()
                    });

                if let (Some(lab_key), Some(val_key)) = (label_key, value_key) {
                    let labels: Vec<String> = data.iter()
                        .filter_map(|row| row.as_object()?.get(&**lab_key)?.as_str().map(|s| s.to_string()))
                        .collect();
                    
                    let values: Vec<f64> = data.iter()
                        .filter_map(|row| {
                            row.as_object()?.get(&**val_key)
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))
                        })
                        .collect();

                    if !labels.is_empty() && !values.is_empty() {
                        return Some(ChartData {
                            chart_type: "pie".to_string(),
                            labels,
                            datasets: vec![ChartDataset {
                                label: val_key.to_string(),
                                data: values,
                                background_color: None,
                            }],
                            title: None,
                        });
                    }
                }
            }
        }
        _ => {}
    }

    None
}

/// Автоматически определяет тип диаграммы на основе данных
fn detect_chart_type(data: &[Value]) -> String {
    if data.is_empty() {
        return "table".to_string();
    }

    // Если данных мало (1-5 строк) - таблица
    if data.len() <= 5 {
        return "table".to_string();
    }

    // Если есть временные данные - line chart или trend
    if let Some(first_obj) = data[0].as_object() {
        let keys: Vec<&String> = first_obj.keys().collect();
        let has_time_key = keys.iter().any(|k| {
            let k_lower = k.to_lowercase();
            k_lower.contains("date") || k_lower.contains("time") || k_lower.contains("день") || 
            k_lower.contains("месяц") || k_lower.contains("month") || k_lower.contains("год") || 
            k_lower.contains("year") || k_lower.contains("timestamp") || k_lower.contains("day")
        });
        
        if has_time_key {
            // Если данных много (более 10), это скорее всего trend
            if data.len() > 10 {
                return "trend".to_string();
            }
            return "line".to_string();
        }
    }

    // Если данных много и есть категории - bar chart
    if data.len() > 5 && data.len() <= 20 {
        return "bar".to_string();
    }

    // По умолчанию - таблица
    "table".to_string()
}

