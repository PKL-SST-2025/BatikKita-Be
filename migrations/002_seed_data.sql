-- Insert sample users
INSERT INTO users (username, email, password_hash, first_name, last_name, phone, role) VALUES
('admin', 'admin@batikkita.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewITD7zdVPeD5QPi', 'Admin', 'Batik Kita', '+6281234567890', 'admin'),
('johndoe', 'john@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewITD7zdVPeD5QPi', 'John', 'Doe', '+6281234567891', 'customer'),
('janedoe', 'jane@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewITD7zdVPeD5QPi', 'Jane', 'Doe', '+6281234567892', 'customer');

-- Insert sample addresses
INSERT INTO user_addresses (user_id, street, city, state, postal_code, is_default) VALUES
(2, 'Jl. Sudirman No. 123', 'Jakarta', 'DKI Jakarta', '10220', true),
(3, 'Jl. Malioboro No. 456', 'Yogyakarta', 'DIY', '55271', true);

-- Insert sample products
INSERT INTO products (name, description, short_description, price, discount_price, sku, stock_quantity, category, brand, weight) VALUES
('Batik Parang Klasik', 'Batik tradisional dengan motif parang yang elegan dan timeless. Dibuat dengan teknik tulis tangan oleh pengrajin berpengalaman.', 'Batik tulis motif parang klasik berkualitas premium', 450000, 350000, 'BTK-PAR-001', 25, 'Batik Tulis', 'Batik Kita', 0.3),
('Batik Mega Mendung', 'Batik khas Cirebon dengan motif mega mendung yang indah. Menggunakan pewarna alami dan kain katun berkualitas tinggi.', 'Batik mega mendung Cirebon asli', 380000, 320000, 'BTK-MM-001', 30, 'Batik Cap', 'Batik Kita', 0.25),
('Batik Kawung Modern', 'Interpretasi modern dari motif kawung klasik dengan sentuhan kontemporer. Cocok untuk acara formal maupun casual.', 'Batik kawung dengan desain modern', 420000, NULL, 'BTK-KW-001', 20, 'Batik Kombinasi', 'Batik Kita', 0.28),
('Batik Lereng Elegant', 'Motif lereng dengan gradasi warna yang memukau. Sempurna untuk tampilan elegan di berbagai kesempatan.', 'Batik lereng dengan gradasi warna indah', 390000, 340000, 'BTK-LR-001', 15, 'Batik Tulis', 'Batik Kita', 0.26),
('Batik Sidomukti Premium', 'Batik dengan motif sidomukti yang membawa makna kemakmuran. Dibuat dengan detail yang sangat teliti.', 'Batik sidomukti premium berkualitas tinggi', 520000, 480000, 'BTK-SM-001', 10, 'Batik Tulis', 'Batik Kita', 0.32),
('Batik Truntum Wedding', 'Batik khusus untuk acara pernikahan dengan motif truntum yang penuh makna. Kualitas super premium.', 'Batik truntum untuk acara pernikahan', 650000, NULL, 'BTK-TR-001', 8, 'Batik Tulis', 'Batik Kita', 0.35),
('Batik Pekalongan Coastal', 'Batik khas Pekalongan dengan nuansa pantai yang segar. Motif yang terinspirasi dari kehidupan pesisir.', 'Batik Pekalongan dengan tema coastal', 360000, 310000, 'BTK-PK-001', 35, 'Batik Cap', 'Batik Kita', 0.24),
('Batik Jogja Sultan', 'Batik klasik Yogyakarta dengan motif yang mencerminkan keanggunan keraton. Warna coklat sogan yang khas.', 'Batik Jogja klasik motif keraton', 480000, 430000, 'BTK-JS-001', 18, 'Batik Tulis', 'Batik Kita', 0.29),
('Batik Modern Geometric', 'Desain batik kontemporer dengan pola geometris yang unik. Cocok untuk generasi muda yang stylish.', 'Batik modern dengan pola geometris', 320000, 280000, 'BTK-MG-001', 40, 'Batik Printing', 'Batik Kita', 0.22),
('Batik Lasem Tiga Negeri', 'Batik Lasem dengan pengaruh tiga budaya yang harmonis. Perpaduan warna merah, putih, dan biru yang menawan.', 'Batik Lasem tiga negeri authentic', 550000, 500000, 'BTK-LS-001', 12, 'Batik Tulis', 'Batik Kita', 0.33);

-- Insert product images
INSERT INTO product_images (product_id, image_url, alt_text, is_primary, sort_order) VALUES
(1, '/images/batik-1.jpg', 'Batik Parang Klasik', true, 1),
(1, '/images/batik-2.jpg', 'Detail motif Parang', false, 2),
(2, '/images/batik-3.jpg', 'Batik Mega Mendung', true, 1),
(2, '/images/batik-4.jpg', 'Detail Mega Mendung', false, 2),
(3, '/images/batik-5.jpg', 'Batik Kawung Modern', true, 1),
(3, '/images/batik-6.jpg', 'Detail Kawung', false, 2),
(4, '/images/batik-7.jpg', 'Batik Lereng Elegant', true, 1),
(4, '/images/batik-8.jpg', 'Detail Lereng', false, 2),
(5, '/images/batik-9.jpg', 'Batik Sidomukti Premium', true, 1),
(5, '/images/batik-10.jpg', 'Detail Sidomukti', false, 2),
(6, '/images/batik-11.jpg', 'Batik Truntum Wedding', true, 1),
(6, '/images/batik-12.jpg', 'Detail Truntum', false, 2),
(7, '/images/batik-1.jpg', 'Batik Pekalongan Coastal', true, 1),
(8, '/images/batik-2.jpg', 'Batik Jogja Sultan', true, 1),
(9, '/images/batik-3.jpg', 'Batik Modern Geometric', true, 1),
(10, '/images/batik-4.jpg', 'Batik Lasem Tiga Negeri', true, 1);

-- Insert product features
INSERT INTO product_features (product_id, feature_name, feature_value) VALUES
-- Batik Parang Klasik
(1, 'Teknik', 'Tulis Tangan'),
(1, 'Bahan', 'Katun Prima'),
(1, 'Ukuran', '2.5m x 1.1m'),
(1, 'Pewarna', 'Alami'),
(1, 'Asal Daerah', 'Solo'),
-- Batik Mega Mendung
(2, 'Teknik', 'Cap'),
(2, 'Bahan', 'Katun Primisima'),
(2, 'Ukuran', '2.4m x 1.1m'),
(2, 'Pewarna', 'Alami'),
(2, 'Asal Daerah', 'Cirebon'),
-- Batik Kawung Modern
(3, 'Teknik', 'Kombinasi Tulis-Cap'),
(3, 'Bahan', 'Katun Prima'),
(3, 'Ukuran', '2.5m x 1.1m'),
(3, 'Pewarna', 'Sintetis'),
(3, 'Asal Daerah', 'Yogyakarta'),
-- Batik Lereng Elegant
(4, 'Teknik', 'Tulis Tangan'),
(4, 'Bahan', 'Katun Primisima'),
(4, 'Ukuran', '2.4m x 1.1m'),
(4, 'Pewarna', 'Alami'),
(4, 'Asal Daerah', 'Solo'),
-- Batik Sidomukti Premium
(5, 'Teknik', 'Tulis Halus'),
(5, 'Bahan', 'Katun Sutra'),
(5, 'Ukuran', '2.5m x 1.2m'),
(5, 'Pewarna', 'Alami Premium'),
(5, 'Asal Daerah', 'Solo');

-- Insert sample reviews
INSERT INTO product_reviews (product_id, user_id, rating, title, comment, is_verified_purchase) VALUES
(1, 2, 5, 'Kualitas Luar Biasa', 'Batik parang ini benar-benar berkualitas tinggi. Motifnya halus dan warna sangat tahan lama. Sangat puas dengan pembelian ini!', true),
(1, 3, 4, 'Bagus tapi agak mahal', 'Kualitas memang bagus, tapi harganya lumayan mahal. Overall tetap worth it untuk koleksi batik.', true),
(2, 2, 5, 'Mega Mendung Terbaik', 'Ini batik mega mendung terbaik yang pernah saya beli. Motifnya authentic dan pewarnaan sangat rapi.', true),
(3, 3, 4, 'Modern dan Stylish', 'Desain kawung modern ini sangat cocok untuk anak muda. Bisa dipake ke acara formal maupun casual.', false),
(4, 2, 5, 'Gradasi Warna Indah', 'Lereng dengan gradasi warna yang sangat indah. Kualitas jahitan juga rapi. Recommended!', true);

-- Insert sample cart items
INSERT INTO cart (user_id, product_id, quantity) VALUES
(2, 1, 1),
(2, 3, 2),
(3, 2, 1),
(3, 4, 1);

-- Insert sample orders
INSERT INTO orders (user_id, order_number, status, total_amount, shipping_cost, tax_amount, payment_method, payment_status, shipping_address_id, billing_address_id) VALUES
(2, 'ORD-2024-0001', 'delivered', 680000, 25000, 68000, 'transfer_bank', 'paid', 1, 1),
(3, 'ORD-2024-0002', 'processing', 320000, 20000, 32000, 'e_wallet', 'paid', 2, 2);

-- Insert order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price, total_price) VALUES
(1, 1, 1, 350000, 350000),
(1, 3, 1, 420000, 420000),
(2, 2, 1, 320000, 320000);

-- Insert sample favorites
INSERT INTO favorites (user_id, product_id) VALUES
(2, 1),
(2, 2),
(2, 5),
(3, 1),
(3, 3),
(3, 4),
(3, 6);

-- Update product statistics
UPDATE products SET 
    is_featured = true 
WHERE id IN (1, 2, 5, 6, 10);
