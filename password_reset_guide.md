# PostgreSQL Password Reset Guide

## Method 1: Using pgAdmin (if installed)
1. Open pgAdmin
2. Connect to your local server
3. Right-click on "postgres" user → Properties
4. Set a new password (e.g., "password")

## Method 2: Using Command Line
1. Open Command Prompt as Administrator
2. Navigate to PostgreSQL bin directory:
   cd "C:\Program Files\PostgreSQL\17\bin"
3. Connect to postgres:
   psql -U postgres
4. When prompted for password, try common defaults:
   - Press Enter (no password)
   - Type: postgres
   - Type: admin
   - Type: password

## Method 3: Edit pg_hba.conf (Advanced)
1. Find pg_hba.conf file in PostgreSQL data directory
2. Change authentication method to "trust" temporarily
3. Restart PostgreSQL service
4. Connect and set new password
5. Change back to "md5" authentication

## Common PostgreSQL Locations:
- Data directory: C:\Program Files\PostgreSQL\17\data\
- Config file: C:\Program Files\PostgreSQL\17\data\pg_hba.conf
- Service name: postgresql-x64-17
