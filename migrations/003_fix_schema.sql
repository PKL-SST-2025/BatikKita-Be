-- Fix schema mismatches for backend code compatibility

-- Add missing columns to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS name VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS password VARCHAR(255);

-- Update existing users to have name and password from existing columns
UPDATE users SET 
    name = COALESCE(first_name || ' ' || last_name, username),
    password = password_hash
WHERE name IS NULL OR password IS NULL;

-- Make the new columns not null
ALTER TABLE users ALTER COLUMN name SET NOT NULL;
ALTER TABLE users ALTER COLUMN password SET NOT NULL;

-- Add missing columns to products table
ALTER TABLE products ADD COLUMN IF NOT EXISTS stock INTEGER DEFAULT 0;
ALTER TABLE products ADD COLUMN IF NOT EXISTS image_url VARCHAR(500);
ALTER TABLE products ADD COLUMN IF NOT EXISTS additional_images TEXT[];
ALTER TABLE products ADD COLUMN IF NOT EXISTS original_price DECIMAL(12, 2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS sold_count INTEGER DEFAULT 0;
ALTER TABLE products ADD COLUMN IF NOT EXISTS size_options TEXT[];
ALTER TABLE products ADD COLUMN IF NOT EXISTS color_options TEXT[];

-- Update existing products
UPDATE products SET 
    stock = stock_quantity,
    original_price = CASE WHEN discount_price IS NOT NULL THEN price ELSE NULL END,
    price = CASE WHEN discount_price IS NOT NULL THEN discount_price ELSE price END
WHERE stock IS NULL;

-- Add missing columns to user_addresses table
ALTER TABLE user_addresses ADD COLUMN IF NOT EXISTS label VARCHAR(100) DEFAULT 'Home';
ALTER TABLE user_addresses ADD COLUMN IF NOT EXISTS full_name VARCHAR(100);
ALTER TABLE user_addresses ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
ALTER TABLE user_addresses ADD COLUMN IF NOT EXISTS province VARCHAR(100);

-- Update existing addresses
UPDATE user_addresses SET 
    province = state,
    full_name = 'User Name'
WHERE province IS NULL OR full_name IS NULL;

-- Create missing tables that the backend expects

-- Cart items table (separate from cart table)
CREATE TABLE IF NOT EXISTS cart_items (
    id SERIAL PRIMARY KEY,
    cart_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    size VARCHAR(50),
    color VARCHAR(50),
    price_at_time DECIMAL(12, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Carts table (separate user carts)
CREATE TABLE IF NOT EXISTS carts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- Reviews table (rename from product_reviews)
CREATE TABLE IF NOT EXISTS reviews (
    id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(product_id, user_id)
);

-- Coupons table
CREATE TABLE IF NOT EXISTS coupons (
    id SERIAL PRIMARY KEY,
    code VARCHAR(50) UNIQUE NOT NULL,
    discount_type VARCHAR(20) NOT NULL, -- 'percentage' or 'fixed'
    discount_value DECIMAL(12, 2) NOT NULL,
    minimum_amount DECIMAL(12, 2) DEFAULT 0,
    usage_limit INTEGER,
    used_count INTEGER DEFAULT 0,
    expires_at TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Add missing columns to orders table
ALTER TABLE orders ADD COLUMN IF NOT EXISTS final_amount DECIMAL(12, 2);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS shipping_address TEXT;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS billing_address TEXT;

-- Update existing orders
UPDATE orders SET 
    final_amount = total_amount - discount_amount + shipping_cost + tax_amount,
    shipping_address = 'Default Address',
    billing_address = 'Default Address'
WHERE final_amount IS NULL;

-- Add missing columns to order_items table
ALTER TABLE order_items ADD COLUMN IF NOT EXISTS product_name VARCHAR(255);
ALTER TABLE order_items ADD COLUMN IF NOT EXISTS product_image VARCHAR(500);
ALTER TABLE order_items ADD COLUMN IF NOT EXISTS size VARCHAR(50);
ALTER TABLE order_items ADD COLUMN IF NOT EXISTS color VARCHAR(50);
ALTER TABLE order_items ADD COLUMN IF NOT EXISTS price_at_time DECIMAL(12, 2);

-- Update existing order_items
UPDATE order_items SET 
    price_at_time = unit_price,
    product_name = 'Product Name',
    product_image = 'default.jpg'
WHERE price_at_time IS NULL;

-- Migrate existing cart data to new structure
-- First create carts for each user
INSERT INTO carts (user_id, created_at, updated_at)
SELECT user_id, MIN(created_at), MAX(updated_at)
FROM cart
GROUP BY user_id
ON CONFLICT (user_id) DO NOTHING;

-- Then migrate cart items
INSERT INTO cart_items (cart_id, product_id, quantity, price_at_time, created_at, updated_at)
SELECT c.id, cart.product_id, cart.quantity, 
       COALESCE(p.price, 100000), cart.created_at, cart.updated_at
FROM cart
JOIN carts c ON c.user_id = cart.user_id
LEFT JOIN products p ON p.id = cart.product_id;

-- Migrate product_reviews to reviews if table exists
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'product_reviews') THEN
        INSERT INTO reviews (product_id, user_id, rating, comment, is_verified, created_at)
        SELECT product_id, user_id, rating, comment, is_verified_purchase, created_at
        FROM product_reviews
        ON CONFLICT (product_id, user_id) DO NOTHING;
    END IF;
END $$;

-- Create indexes for new tables
CREATE INDEX IF NOT EXISTS idx_cart_items_cart_id ON cart_items(cart_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_product_id ON cart_items(product_id);
CREATE INDEX IF NOT EXISTS idx_carts_user_id ON carts(user_id);
CREATE INDEX IF NOT EXISTS idx_reviews_product_id ON reviews(product_id);
CREATE INDEX IF NOT EXISTS idx_reviews_user_id ON reviews(user_id);
CREATE INDEX IF NOT EXISTS idx_coupons_code ON coupons(code);

-- Add triggers for updated_at on new tables
CREATE TRIGGER update_cart_items_updated_at BEFORE UPDATE ON cart_items
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_carts_updated_at BEFORE UPDATE ON carts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
