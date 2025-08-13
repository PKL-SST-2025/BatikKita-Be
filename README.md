# Batik Kita - Backend API

Backend API untuk aplikasi e-commerce Batik Kita yang dibangun dengan Rust menggunakan framework Actix-web.

## Features

- **Authentication & Authorization**: JWT-based authentication dengan role-based access control
- **Product Management**: CRUD operations untuk produk dengan kategori, gambar, dan fitur
- **User Management**: Registrasi, login, profil pengguna, dan manajemen alamat
- **Shopping Cart**: Add, update, delete items dalam keranjang belanja
- **Order Management**: Proses checkout, manajemen pesanan, dan tracking status
- **Favorites**: Sistem wishlist untuk produk favorit
- **Reviews**: Rating dan review produk dari pengguna
- **Admin Panel**: Dashboard admin untuk manajemen produk dan pesanan

## Tech Stack

- **Framework**: Actix-web
- **Database**: PostgreSQL dengan SQLx
- **Authentication**: JWT (JSON Web Tokens)
- **Password Hashing**: bcrypt
- **Serialization**: Serde
- **CORS**: Actix-cors
- **Environment**: dotenv

## Installation

1. Install Rust dan Cargo
2. Install PostgreSQL
3. Clone repository dan navigate ke direktori backend:
```bash
cd be
```

4. Install dependencies:
```bash
cargo build
```

5. Setup database dan environment variables:
```bash
cp .env.example .env
```

6. Edit file `.env` dengan konfigurasi database Anda:
```env
DATABASE_URL=postgresql://username:password@localhost/batik_kita
JWT_SECRET=your-secret-key-here
RUST_LOG=info
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

7. Run database migrations:
```bash
# Buat database terlebih dahulu
createdb batik_kita

# Run migrations
psql -d batik_kita -f migrations/001_initial.sql
psql -d batik_kita -f migrations/002_seed_data.sql
```

8. Run the server:
```bash
cargo run
```

Server akan berjalan di `http://localhost:8080`

## API Endpoints

### Authentication
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout
- `POST /api/auth/refresh` - Refresh JWT token

### Products
- `GET /api/products` - Get all products (with pagination and filters)
- `GET /api/products/{id}` - Get product by ID
- `POST /api/products` - Create new product (Admin only)
- `PUT /api/products/{id}` - Update product (Admin only)
- `DELETE /api/products/{id}` - Delete product (Admin only)
- `GET /api/products/{id}/reviews` - Get product reviews

### Cart
- `GET /api/cart` - Get user's cart
- `POST /api/cart` - Add item to cart
- `PUT /api/cart/{id}` - Update cart item quantity
- `DELETE /api/cart/{id}` - Remove item from cart
- `DELETE /api/cart` - Clear entire cart

### Orders
- `POST /api/checkout` - Create new order
- `GET /api/orders` - Get user's orders
- `GET /api/orders/{id}` - Get order details
- `PUT /api/orders/{id}/status` - Update order status (Admin only)

### User Management
- `GET /api/user/profile` - Get user profile
- `PUT /api/user/profile` - Update user profile
- `GET /api/user/addresses` - Get user addresses
- `POST /api/user/addresses` - Add new address
- `PUT /api/user/addresses/{id}` - Update address
- `DELETE /api/user/addresses/{id}` - Delete address

### Favorites
- `GET /api/favorites` - Get user's favorite products
- `POST /api/favorites` - Add product to favorites
- `DELETE /api/favorites/{product_id}` - Remove from favorites

### Admin
- `GET /api/admin/dashboard` - Admin dashboard statistics
- `GET /api/admin/orders` - Get all orders
- `GET /api/admin/users` - Get all users
- `PUT /api/admin/users/{id}/status` - Update user status

## Database Schema

Database terdiri dari beberapa tabel utama:

- **users**: Data pengguna dan authentication
- **user_addresses**: Alamat pengiriman pengguna
- **products**: Data produk batik
- **product_images**: Gambar produk
- **product_features**: Fitur dan spesifikasi produk
- **product_reviews**: Review dan rating produk
- **cart**: Keranjang belanja
- **orders**: Data pesanan
- **order_items**: Detail item dalam pesanan
- **favorites**: Produk favorit pengguna

## Authentication

API menggunakan JWT untuk authentication. Setelah login, client akan menerima access token yang harus disertakan dalam header:

```
Authorization: Bearer <token>
```

## Error Handling

API mengembalikan response dalam format JSON yang konsisten:

```json
{
  "success": boolean,
  "data": object|null,
  "message": string,
  "error_code": string|null
}
```

## Development

Untuk development, gunakan:

```bash
cargo watch -x run
```

Ini akan otomatis restart server ketika ada perubahan code.

## Testing

Run tests dengan:

```bash
cargo test
```

## Production Build

Untuk production build:

```bash
cargo build --release
```

## Deployment

### Option 1: Railway (Recommended)

1. Push repository ke GitHub
2. Connect ke Railway.app
3. Deploy secara otomatis

Environment Variables untuk Railway:
- `DATABASE_URL=postgresql://username:password@host:port/database`
- `JWT_SECRET=your-secret-key`
- `RUST_LOG=info`

### Option 2: Render

1. Push repository ke GitHub
2. Connect ke Render.com
3. Set build command: `cargo build --release`
4. Set start command: `./target/release/be`

### Option 3: Heroku

1. Create Procfile
2. Use Rust buildpack
3. Configure environment variables

## Contributing

1. Fork repository
2. Create feature branch
3. Commit changes
4. Push to branch
5. Create Pull Request

## License

MIT License
