-- Notifications table
CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    type VARCHAR(50) NOT NULL DEFAULT 'general', -- 'order', 'favorite', 'cart', 'promo', 'system', 'general'
    reference_id INTEGER, -- ID referensi (order_id, product_id, dll)
    reference_type VARCHAR(50), -- 'order', 'product', 'user', dll
    is_read BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal', -- 'high', 'normal', 'low'
    action_url VARCHAR(500), -- URL untuk redirect ketika notif diklik
    metadata JSONB, -- Data tambahan dalam format JSON
    expires_at TIMESTAMP, -- Kapan notif expires (opsional)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Notification preferences per user
CREATE TABLE notification_preferences (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    delivery_method VARCHAR(20) NOT NULL DEFAULT 'app', -- 'app', 'email', 'sms'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, notification_type, delivery_method)
);

-- Real-time notification sessions (untuk WebSocket)
CREATE TABLE notification_sessions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    socket_id VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_ping TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes untuk performance
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_is_deleted ON notifications(is_deleted);
CREATE INDEX idx_notifications_type ON notifications(type);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read, is_deleted);
CREATE INDEX idx_notification_preferences_user_id ON notification_preferences(user_id);
CREATE INDEX idx_notification_sessions_user_id ON notification_sessions(user_id, is_active);

-- Triggers untuk auto-update timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_notifications_updated_at BEFORE UPDATE ON notifications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_notification_preferences_updated_at BEFORE UPDATE ON notification_preferences
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_notification_sessions_updated_at BEFORE UPDATE ON notification_sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default notification preferences untuk existing users
INSERT INTO notification_preferences (user_id, notification_type, enabled, delivery_method)
SELECT u.id, nt.type, true, 'app'
FROM users u
CROSS JOIN (
    VALUES 
        ('order'),
        ('favorite'),
        ('cart'),
        ('promo'),
        ('system'),
        ('general')
) AS nt(type)
ON CONFLICT (user_id, notification_type, delivery_method) DO NOTHING;
