use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct UserSafety {
    pub user_id: String,
    pub warnings: u32,
    pub violations: Vec<Violation>,
    pub banned_until: Option<DateTime<Utc>>,
    pub last_violation: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub violation_type: ViolationType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViolationType {
    JailbreakAttempt,
    InappropriateLanguage,
    SystemAbuse,
    RepeatedViolations,
}

pub struct UserSafetyManager {
    users: Arc<RwLock<HashMap<String, UserSafety>>>,
    max_warnings: u32,
    ban_duration_hours: i64,
}

impl UserSafetyManager {
    pub fn new(max_warnings: u32, ban_duration_hours: i64) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            max_warnings,
            ban_duration_hours,
        }
    }

    pub async fn check_user(&self, user_id: &str) -> Result<(), String> {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            // Проверяем, не забанен ли пользователь
            if let Some(banned_until) = user.banned_until {
                if banned_until > Utc::now() {
                    let remaining = (banned_until - Utc::now()).num_minutes();
                    return Err(format!("Вы временно заблокированы. Разблокировка через {} минут.", remaining));
                }
            }
        }
        Ok(())
    }

    pub async fn record_violation(
        &self,
        user_id: &str,
        violation_type: ViolationType,
        message: String,
    ) -> Option<String> {
        let mut users = self.users.write().await;
        let user = users.entry(user_id.to_string())
            .or_insert_with(|| UserSafety {
                user_id: user_id.to_string(),
                warnings: 0,
                violations: Vec::new(),
                banned_until: None,
                last_violation: None,
            });

        // Добавляем нарушение
        user.violations.push(Violation {
            violation_type: violation_type.clone(),
            message: message.clone(),
            timestamp: Utc::now(),
        });
        user.last_violation = Some(Utc::now());

        // Увеличиваем предупреждения для серьезных нарушений
        match violation_type {
            ViolationType::JailbreakAttempt | ViolationType::SystemAbuse => {
                user.warnings += 2;
            }
            ViolationType::InappropriateLanguage | ViolationType::RepeatedViolations => {
                user.warnings += 1;
            }
        }

        // Если превышен лимит предупреждений - бан
        if user.warnings >= self.max_warnings {
            let ban_until = Utc::now() + chrono::Duration::hours(self.ban_duration_hours);
            user.banned_until = Some(ban_until);
            return Some(format!(
                "Вы получили слишком много предупреждений и временно заблокированы на {} часов. Разблокировка: {}",
                self.ban_duration_hours,
                ban_until.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        }

        // Предупреждение
        if user.warnings > 0 {
            return Some(format!(
                "⚠️ Предупреждение {}/{}: {}. При повторных нарушениях вы можете быть временно заблокированы.",
                user.warnings,
                self.max_warnings,
                message
            ));
        }

        None
    }

    pub async fn check_message_safety(&self, user_id: &str, message: &str) -> (bool, Option<String>) {
        // Проверяем, не забанен ли пользователь
        if let Err(ban_msg) = self.check_user(user_id).await {
            return (false, Some(ban_msg));
        }

        let message_lower = message.to_lowercase();

        // Проверка на jailbreak попытки
        let jailbreak_keywords = [
            "ignore previous", "forget all", "you are now", "you are a",
            "act as", "pretend to be", "roleplay as", "disregard instructions",
            "forget your instructions", "ignore your instructions",
            "you must", "you have to", "system:", "assistant:",
        ];

        for keyword in &jailbreak_keywords {
            if message_lower.contains(keyword) {
                let ban_msg = self.record_violation(
                    user_id,
                    ViolationType::JailbreakAttempt,
                    format!("Обнаружена попытка jailbreak: {}", keyword),
                ).await;
                return (false, ban_msg);
            }
        }

        // Проверка на нецензурную лексику (базовая)
        let inappropriate_keywords: [&str; 0] = [
            // Добавить при необходимости
        ];

        for keyword in &inappropriate_keywords {
            if message_lower.contains(keyword) {
                let ban_msg = self.record_violation(
                    user_id,
                    ViolationType::InappropriateLanguage,
                    "Использование недопустимой лексики".to_string(),
                ).await;
                return (false, ban_msg);
            }
        }

        // Проверка на злоупотребление системой (множественные запросы без LIMIT)
        // Это обрабатывается на уровне валидации SQL

        (true, None)
    }

    pub async fn clear_warnings(&self, user_id: &str) {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.warnings = 0;
            user.banned_until = None;
        }
    }
}

