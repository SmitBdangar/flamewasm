@echo off
echo ====================================
echo Data Visualization Dashboard Setup
echo ====================================
echo.

echo Step 1: Installing backend dependencies...
cd backend
call npm install
if errorlevel 1 (
    echo Backend installation failed!
    pause
    exit /b 1
)
echo Backend dependencies installed successfully!
echo.

echo Step 2: Installing frontend dependencies...
cd ..\frontend
call npm install
if errorlevel 1 (
    echo Frontend installation failed!
    pause
    exit /b 1
)
echo Frontend dependencies installed successfully!
echo.

echo Step 3: Setting up environment file...
cd ..\backend
if not exist .env (
    echo PORT=5000 > .env
    echo MONGODB_URI=mongodb://localhost:27017/visualization_dashboard >> .env
    echo NODE_ENV=development >> .env
    echo Created .env file. Please update MONGODB_URI if needed.
) else (
    echo .env file already exists.
)
echo.

echo ====================================
echo Setup Complete!
echo ====================================
echo.
echo Next steps:
echo 1. Make sure MongoDB is running
echo 2. Run: cd backend ^&^& npm run seed
echo 3. Start backend: cd backend ^&^& npm start
echo 4. Start frontend: cd frontend ^&^& npm start
echo.
pause

