use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub message: String,
    pub r#type: String,
    pub reference_id: Option<i32>,
    pub reference_type: Option<String>,
    pub is_read: bool,
    pub is_deleted: bool,
    pub priority: String,
    pub action_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationPreference {
    pub id: i32,
    pub user_id: i32,
    pub notification_type: String,
    pub enabled: bool,
    pub delivery_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationSession {
    pub id: i32,
    pub user_id: i32,
    pub session_id: String,
    pub socket_id: Option<String>,
    pub is_active: bool,
    pub last_ping: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub message: String,
    pub r#type: Option<String>,
    pub reference_id: Option<i32>,
    pub reference_type: Option<String>,
    pub priority: Option<String>,
    pub action_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationRequest {
    pub is_read: Option<bool>,
    pub is_deleted: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationFilters {
    pub r#type: Option<String>,
    pub is_read: Option<bool>,
    pub is_deleted: Option<bool>,
    pub priority: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_count: Option<i64>,
    pub unread_count: Option<i64>,
    pub high_priority_unread: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeNotificationPayload {
    pub notification: Notification,
    pub stats: NotificationStats,
}

#[derive(Debug, Deserialize)]
pub struct MarkMultipleRequest {
    pub notification_ids: Vec<i32>,
    pub is_read: Option<bool>,
    pub is_deleted: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationPreferenceUpdate {
    pub notification_type: String,
    pub enabled: bool,
    pub delivery_method: Option<String>,
}
