# Data Visualization Dashboard

A comprehensive data visualization dashboard built with MERN stack (MongoDB, Express, React, Node.js) that provides interactive charts and filters for analyzing data.

## Features

- **Interactive Visualizations**: Multiple chart types including Bar, Line, Pie, and Doughnut charts
- **Comprehensive Filters**: Filter data by End Year, Topics, Sector, Region, PEST, Source, SWOT, Country, and City
- **Real-time Updates**: Charts update automatically when filters are applied
- **Statistics Cards**: Display key metrics including average Intensity, Likelihood, and Relevance
- **Responsive Design**: Modern UI built with Bootstrap and React

## Visualizations

The dashboard includes the following visualizations:

1. **Intensity Distribution** - Bar chart showing intensity ranges
2. **Likelihood Distribution** - Doughnut chart showing likelihood levels
3. **Relevance Distribution** - Line chart showing relevance trends
4. **Trends by Year** - Multi-bar chart showing Intensity, Likelihood, and Relevance over years
5. **Top Countries** - Horizontal bar chart showing record count by country
6. **Distribution by Topics** - Pie chart showing topic distribution
7. **Analysis by Region** - Bar chart comparing regions
8. **Top Cities** - Horizontal bar chart showing record count by city

## Prerequisites

- Node.js (v14 or higher)
- MongoDB (local installation or MongoDB Atlas account)
- npm or yarn

## Installation

### 1. Clone or navigate to the project directory

```bash
cd Shasati
```

### 2. Install Backend Dependencies

```bash
cd backend
npm install
```

### 3. Install Frontend Dependencies

```bash
cd ../frontend
npm install
```

### 4. Configure MongoDB

Create a `.env` file in the `backend` directory:

```bash
cd ../backend
cp .env.example .env
```

Edit `.env` and set your MongoDB connection string:

```
PORT=5000
MONGODB_URI=mongodb://localhost:27017/visualization_dashboard
NODE_ENV=development
```

For MongoDB Atlas, use:
```
MONGODB_URI=mongodb+srv://username:password@cluster.mongodb.net/visualization_dashboard
```

### 5. Start MongoDB

Make sure MongoDB is running on your system. If using local MongoDB:

```bash
# Windows
net start MongoDB

# macOS/Linux
sudo systemctl start mongod
# or
mongod
```

### 6. Seed the Database

Import the JSON data into MongoDB:

```bash
cd backend
npm run seed
```

This will create the database and import data from `jsondata.json`.

## Running the Application

### Start the Backend Server

```bash
cd backend
npm start
# or for development with auto-reload
npm run dev
```

The backend API will run on `http://localhost:5000`

### Start the Frontend Application

Open a new terminal:

```bash
cd frontend
npm start
```

The frontend will run on `http://localhost:3000` and automatically open in your browser.

## API Endpoints

### Get All Data (with filters)
```
GET /api/data?endYear=2020&topics=oil&sector=Energy&region=Asia&...
```

### Get Statistics
```
GET /api/stats
```

### Get Data Grouped by Year
```
GET /api/data/by-year
```

### Get Data Grouped by Country
```
GET /api/data/by-country
```

### Get Data Grouped by Topics
```
GET /api/data/by-topics
```

### Get Data Grouped by Region
```
GET /api/data/by-region
```

### Get Data Grouped by City
```
GET /api/data/by-city
```

## Project Structure

```
Shasati/
├── backend/
│   ├── scripts/
│   │   └── seedDatabase.js
│   ├── .env.example
│   ├── package.json
│   └── server.js
├── frontend/
│   ├── public/
│   │   └── index.html
│   ├── src/
│   │   ├── components/
│   │   │   ├── charts/
│   │   │   │   ├── CityChart.js
│   │   │   │   ├── CountryChart.js
│   │   │   │   ├── IntensityChart.js
│   │   │   │   ├── LikelihoodChart.js
│   │   │   │   ├── RelevanceChart.js
│   │   │   │   ├── RegionChart.js
│   │   │   │   ├── TopicsChart.js
│   │   │   │   └── YearChart.js
│   │   │   ├── Dashboard.js
│   │   │   ├── Filters.js
│   │   │   └── StatsCards.js
│   │   ├── App.js
│   │   ├── App.css
│   │   ├── index.js
│   │   └── index.css
│   └── package.json
├── jsondata.json
└── README.md
```

## Technologies Used

### Backend
- **Node.js** - Runtime environment
- **Express** - Web framework
- **MongoDB** - Database
- **Mongoose** - MongoDB object modeling

### Frontend
- **React** - UI library
- **Chart.js** - Charting library
- **React-Chartjs-2** - React wrapper for Chart.js
- **Bootstrap** - CSS framework
- **React-Bootstrap** - Bootstrap components for React
- **Axios** - HTTP client

## Customization

### Adding More Data

1. Update `jsondata.json` with your data
2. Run the seed script again: `npm run seed` (from backend directory)

### Adding New Charts

1. Create a new chart component in `frontend/src/components/charts/`
2. Import and add it to `Dashboard.js`
3. Create corresponding API endpoint in `backend/server.js` if needed

### Modifying Filters

Edit `frontend/src/components/Filters.js` to add or modify filter options.

## Troubleshooting

### MongoDB Connection Issues

- Ensure MongoDB is running
- Check the connection string in `.env`
- Verify network connectivity for MongoDB Atlas

### Port Already in Use

- Change the port in `backend/.env` or `frontend/package.json`
- Update the API URL in frontend if backend port changes

### CORS Errors

- Ensure backend CORS is configured (already included)
- Check that API URL matches in frontend

## License

This project is created for educational purposes.

