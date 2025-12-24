#!/bin/bash

echo "===================================="
echo "Data Visualization Dashboard Setup"
echo "===================================="
echo ""

echo "Step 1: Installing backend dependencies..."
cd backend
npm install
if [ $? -ne 0 ]; then
    echo "Backend installation failed!"
    exit 1
fi
echo "Backend dependencies installed successfully!"
echo ""

echo "Step 2: Installing frontend dependencies..."
cd ../frontend
npm install
if [ $? -ne 0 ]; then
    echo "Frontend installation failed!"
    exit 1
fi
echo "Frontend dependencies installed successfully!"
echo ""

echo "Step 3: Setting up environment file..."
cd ../backend
if [ ! -f .env ]; then
    cat > .env << EOF
PORT=5000
MONGODB_URI=mongodb://localhost:27017/visualization_dashboard
NODE_ENV=development
EOF
    echo "Created .env file. Please update MONGODB_URI if needed."
else
    echo ".env file already exists."
fi
echo ""

echo "===================================="
echo "Setup Complete!"
echo "===================================="
echo ""
echo "Next steps:"
echo "1. Make sure MongoDB is running"
echo "2. Run: cd backend && npm run seed"
echo "3. Start backend: cd backend && npm start"
echo "4. Start frontend: cd frontend && npm start"
echo ""

