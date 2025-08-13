use actix_web::{get, post, put, delete, web, HttpResponse, Responder, Result};
use sqlx::PgPool;
use crate::models::notification::*;
use crate::models::user::Claims;
use crate::middleware::AuthMiddleware;
use chrono::Utc;

// Get notifications for authenticated user
#[get("")]
async fn get_notifications(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    query: web::Query<NotificationFilters>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let filters = query.into_inner();
    
    let limit = filters.limit.unwrap_or(20);
    let offset = filters.offset.unwrap_or(0);

    let mut query_builder = String::new();
    let mut conditions = vec!["user_id = $1".to_string()];
    let mut param_count = 1;

    // Build dynamic WHERE conditions
    if let Some(notification_type) = &filters.r#type {
        param_count += 1;
        conditions.push(format!("type = ${}", param_count));
    }

    if let Some(is_read) = filters.is_read {
        param_count += 1;
        conditions.push(format!("is_read = ${}", param_count));
    }

    if let Some(is_deleted) = filters.is_deleted {
        param_count += 1;
        conditions.push(format!("is_deleted = ${}", param_count));
    } else {
        // Default: don't show deleted notifications
        conditions.push("is_deleted = false".to_string());
    }

    if let Some(priority) = &filters.priority {
        param_count += 1;
        conditions.push(format!("priority = ${}", param_count));
    }

    // Check for expired notifications
    conditions.push("(expires_at IS NULL OR expires_at > NOW())".to_string());

    query_builder = format!(
        "SELECT * FROM notifications WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        conditions.join(" AND "),
        param_count + 1,
        param_count + 2
    );

    let mut query = sqlx::query_as::<_, Notification>(&query_builder)
        .bind(user_id);

    // Bind dynamic parameters
    if let Some(notification_type) = &filters.r#type {
        query = query.bind(notification_type);
    }
    if let Some(is_read) = filters.is_read {
        query = query.bind(is_read);
    }
    if let Some(is_deleted) = filters.is_deleted {
        query = query.bind(is_deleted);
    }
    if let Some(priority) = &filters.priority {
        query = query.bind(priority);
    }

    query = query.bind(limit).bind(offset);

    let notifications = match query
        .fetch_all(pool.get_ref())
        .await {
            Ok(notifications) => notifications,
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Failed to fetch notifications"));
            }
        };

    Ok(HttpResponse::Ok().json(notifications))
}

// Get notification statistics
#[get("/stats")]
async fn get_notification_stats(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    let stats = match sqlx::query_as!(
        NotificationStats,
        r#"
        SELECT 
            COUNT(*) as total_count,
            COUNT(*) FILTER (WHERE is_read = false) as unread_count,
            COUNT(*) FILTER (WHERE is_read = false AND priority = 'high') as high_priority_unread
        FROM notifications 
        WHERE user_id = $1 
            AND is_deleted = false 
            AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch notification stats"));
        }
    };

    Ok(HttpResponse::Ok().json(stats))
}

// Create notification (admin only or system)
#[post("")]
async fn create_notification(
    pool: web::Data<PgPool>,
    notification_data: web::Json<CreateNotificationRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let data = notification_data.into_inner();

    let notification = match sqlx::query_as::<_, Notification>(
        r#"
        INSERT INTO notifications (
            user_id, title, message, type, reference_id, reference_type, 
            priority, action_url, metadata, expires_at, created_at, updated_at
        ) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW()) 
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&data.title)
    .bind(&data.message)
    .bind(data.r#type.unwrap_or_else(|| "general".to_string()))
    .bind(data.reference_id)
    .bind(data.reference_type)
    .bind(data.priority.unwrap_or_else(|| "normal".to_string()))
    .bind(data.action_url)
    .bind(data.metadata)
    .bind(data.expires_at)
    .fetch_one(pool.get_ref())
    .await {
        Ok(notification) => notification,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to create notification"));
        }
    };

    // TODO: Send real-time notification via WebSocket
    
    Ok(HttpResponse::Created().json(notification))
}

// Update single notification (mark as read/deleted)
#[put("/{id}")]
async fn update_notification(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    update_data: web::Json<UpdateNotificationRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let notification_id = path.into_inner();
    let data = update_data.into_inner();

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut param_count = 2;

    if let Some(is_read) = data.is_read {
        updates.push(format!("is_read = ${}", param_count));
        param_count += 1;
    }

    if let Some(is_deleted) = data.is_deleted {
        updates.push(format!("is_deleted = ${}", param_count));
        param_count += 1;
    }

    if updates.is_empty() {
        return Ok(HttpResponse::BadRequest().json("No fields to update"));
    }

    updates.push("updated_at = NOW()".to_string());

    let query_str = format!(
        "UPDATE notifications SET {} WHERE id = $1 AND user_id = ${} RETURNING *",
        updates.join(", "),
        param_count
    );

    let mut query = sqlx::query_as::<_, Notification>(&query_str)
        .bind(notification_id)
        .bind(user_id);

    if let Some(is_read) = data.is_read {
        query = query.bind(is_read);
    }
    if let Some(is_deleted) = data.is_deleted {
        query = query.bind(is_deleted);
    }

    let notification = match query
        .fetch_optional(pool.get_ref())
        .await {
            Ok(Some(notif)) => notif,
            Ok(None) => return Ok(HttpResponse::NotFound().json("Notification not found")),
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Failed to update notification"));
            }
        };

    Ok(HttpResponse::Ok().json(notification))
}

// Mark multiple notifications
#[put("/bulk")]
async fn mark_multiple_notifications(
    pool: web::Data<PgPool>,
    update_data: web::Json<MarkMultipleRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let data = update_data.into_inner();

    if data.notification_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json("No notification IDs provided"));
    }

    let mut updates = Vec::new();
    let mut param_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send>> = Vec::new();
    let mut param_count = 1;

    // Add user_id as first parameter
    param_values.push(Box::new(user_id));
    param_count += 1;

    if let Some(is_read) = data.is_read {
        updates.push(format!("is_read = ${}", param_count));
        param_values.push(Box::new(is_read));
        param_count += 1;
    }

    if let Some(is_deleted) = data.is_deleted {
        updates.push(format!("is_deleted = ${}", param_count));
        param_values.push(Box::new(is_deleted));
        param_count += 1;
    }

    if updates.is_empty() {
        return Ok(HttpResponse::BadRequest().json("No fields to update"));
    }

    updates.push("updated_at = NOW()".to_string());

    // Create placeholders for IDs
    let id_placeholders: Vec<String> = data.notification_ids.iter()
        .enumerate()
        .map(|(i, _)| format!("${}", param_count + i))
        .collect();

    let query_str = format!(
        "UPDATE notifications SET {} WHERE user_id = $1 AND id = ANY(ARRAY[{}])",
        updates.join(", "),
        id_placeholders.join(", ")
    );

    let result = match sqlx::query(&query_str)
        .bind(user_id)
        .bind(data.is_read)
        .bind(data.is_deleted)
        .bind(&data.notification_ids)
        .execute(pool.get_ref())
        .await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Failed to update notifications"));
            }
        };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "updated_count": result.rows_affected(),
        "message": "Notifications updated successfully"
    })))
}

// Delete notification (soft delete)
#[delete("/{id}")]
async fn delete_notification(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let notification_id = path.into_inner();

    let result = match sqlx::query!(
        "UPDATE notifications SET is_deleted = true, updated_at = NOW() 
         WHERE id = $1 AND user_id = $2",
        notification_id,
        user_id
    )
    .execute(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to delete notification"));
        }
    };

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json("Notification deleted successfully"))
    } else {
        Ok(HttpResponse::NotFound().json("Notification not found"))
    }
}

// Mark all as read
#[put("/mark-all-read")]
async fn mark_all_as_read(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    let result = match sqlx::query!(
        "UPDATE notifications SET is_read = true, updated_at = NOW() 
         WHERE user_id = $1 AND is_read = false AND is_deleted = false",
        user_id
    )
    .execute(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to mark notifications as read"));
        }
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "updated_count": result.rows_affected(),
        "message": "All notifications marked as read"
    })))
}

// Get notification preferences
#[get("/preferences")]
async fn get_notification_preferences(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    let preferences = match sqlx::query_as::<_, NotificationPreference>(
        "SELECT * FROM notification_preferences WHERE user_id = $1 ORDER BY notification_type"
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await {
        Ok(preferences) => preferences,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch notification preferences"));
        }
    };

    Ok(HttpResponse::Ok().json(preferences))
}

// Update notification preferences
#[put("/preferences")]
async fn update_notification_preferences(
    pool: web::Data<PgPool>,
    preference_data: web::Json<NotificationPreferenceUpdate>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let data = preference_data.into_inner();

    let delivery_method = data.delivery_method.unwrap_or_else(|| "app".to_string());

    let preference = match sqlx::query_as::<_, NotificationPreference>(
        r#"
        INSERT INTO notification_preferences (user_id, notification_type, enabled, delivery_method, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        ON CONFLICT (user_id, notification_type, delivery_method) 
        DO UPDATE SET enabled = $3, updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&data.notification_type)
    .bind(data.enabled)
    .bind(&delivery_method)
    .fetch_one(pool.get_ref())
    .await {
        Ok(preference) => preference,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to update notification preference"));
        }
    };

    Ok(HttpResponse::Ok().json(preference))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notifications")
            .wrap(AuthMiddleware)
            .service(get_notifications)
            .service(get_notification_stats)
            .service(create_notification)
            .service(update_notification)
            .service(mark_multiple_notifications)
            .service(delete_notification)
            .service(mark_all_as_read)
            .service(get_notification_preferences)
            .service(update_notification_preferences)
    );
}
