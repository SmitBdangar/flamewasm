const express = require('express');
const mongoose = require('mongoose');
const cors = require('cors');
require('dotenv').config();

const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// MongoDB Connection
const MONGODB_URI = process.env.MONGODB_URI || 'mongodb://localhost:27017/visualization_dashboard';

mongoose.connect(MONGODB_URI, {
  useNewUrlParser: true,
  useUnifiedTopology: true,
})
.then(() => console.log('MongoDB Connected'))
.catch(err => console.error('MongoDB connection error:', err));

// Data Model
const dataSchema = new mongoose.Schema({
  intensity: Number,
  likelihood: Number,
  relevance: Number,
  year: Number,
  country: String,
  topics: String,
  region: String,
  city: String,
  sector: String,
  pest: String,
  source: String,
  swot: String,
  title: String,
  insight: String
}, { collection: 'dashboard_data' });

const DataModel = mongoose.model('Data', dataSchema);

// Helper function to build match filter for aggregation
function buildMatchFilter(query) {
  const matchFilter = {};
  
  if (query.endYear) {
    matchFilter.year = { $lte: parseInt(query.endYear) };
  }
  if (query.topics) {
    matchFilter.topics = { $regex: query.topics, $options: 'i' };
  }
  if (query.sector) {
    matchFilter.sector = { $regex: query.sector, $options: 'i' };
  }
  if (query.region) {
    matchFilter.region = { $regex: query.region, $options: 'i' };
  }
  if (query.pest) {
    matchFilter.pest = { $regex: query.pest, $options: 'i' };
  }
  if (query.source) {
    matchFilter.source = { $regex: query.source, $options: 'i' };
  }
  if (query.swot) {
    matchFilter.swot = { $regex: query.swot, $options: 'i' };
  }
  if (query.country) {
    matchFilter.country = { $regex: query.country, $options: 'i' };
  }
  if (query.city) {
    matchFilter.city = { $regex: query.city, $options: 'i' };
  }
  
  return matchFilter;
}

// API Routes

// Get all data with filters
app.get('/api/data', async (req, res) => {
  try {
    const {
      endYear,
      topics,
      sector,
      region,
      pest,
      source,
      swot,
      country,
      city,
      limit,
      skip
    } = req.query;

    // Build filter object
    const filter = {};
    
    if (endYear) {
      filter.year = { $lte: parseInt(endYear) };
    }
    if (topics) {
      filter.topics = { $regex: topics, $options: 'i' };
    }
    if (sector) {
      filter.sector = { $regex: sector, $options: 'i' };
    }
    if (region) {
      filter.region = { $regex: region, $options: 'i' };
    }
    if (pest) {
      filter.pest = { $regex: pest, $options: 'i' };
    }
    if (source) {
      filter.source = { $regex: source, $options: 'i' };
    }
    if (swot) {
      filter.swot = { $regex: swot, $options: 'i' };
    }
    if (country) {
      filter.country = { $regex: country, $options: 'i' };
    }
    if (city) {
      filter.city = { $regex: city, $options: 'i' };
    }

    const query = DataModel.find(filter);
    
    if (limit) {
      query.limit(parseInt(limit));
    }
    if (skip) {
      query.skip(parseInt(skip));
    }

    const data = await query.exec();
    const total = await DataModel.countDocuments(filter);

    res.json({
      success: true,
      data,
      total,
      count: data.length
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching data',
      error: error.message
    });
  }
});

// Get aggregated statistics
app.get('/api/stats', async (req, res) => {
  try {
    const stats = await DataModel.aggregate([
      {
        $group: {
          _id: null,
          avgIntensity: { $avg: '$intensity' },
          avgLikelihood: { $avg: '$likelihood' },
          avgRelevance: { $avg: '$relevance' },
          minYear: { $min: '$year' },
          maxYear: { $max: '$year' },
          totalRecords: { $sum: 1 }
        }
      }
    ]);

    // Get unique values for filters
    const uniqueTopics = await DataModel.distinct('topics');
    const uniqueSectors = await DataModel.distinct('sector');
    const uniqueRegions = await DataModel.distinct('region');
    const uniquePests = await DataModel.distinct('pest');
    const uniqueSources = await DataModel.distinct('source');
    const uniqueSwots = await DataModel.distinct('swot');
    const uniqueCountries = await DataModel.distinct('country');
    const uniqueCities = await DataModel.distinct('city');

    res.json({
      success: true,
      stats: stats[0] || {},
      filters: {
        topics: uniqueTopics,
        sectors: uniqueSectors,
        regions: uniqueRegions,
        pests: uniquePests,
        sources: uniqueSources,
        swots: uniqueSwots,
        countries: uniqueCountries,
        cities: uniqueCities
      }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching statistics',
      error: error.message
    });
  }
});

// Get data grouped by year
app.get('/api/data/by-year', async (req, res) => {
  try {
    const matchFilter = buildMatchFilter(req.query);
    const pipeline = [];
    
    if (Object.keys(matchFilter).length > 0) {
      pipeline.push({ $match: matchFilter });
    }
    
    pipeline.push(
      {
        $group: {
          _id: '$year',
          avgIntensity: { $avg: '$intensity' },
          avgLikelihood: { $avg: '$likelihood' },
          avgRelevance: { $avg: '$relevance' },
          count: { $sum: 1 }
        }
      },
      { $sort: { _id: 1 } }
    );
    
    const data = await DataModel.aggregate(pipeline);

    res.json({
      success: true,
      data
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching year data',
      error: error.message
    });
  }
});

// Get data grouped by country
app.get('/api/data/by-country', async (req, res) => {
  try {
    const matchFilter = buildMatchFilter(req.query);
    const pipeline = [];
    
    if (Object.keys(matchFilter).length > 0) {
      pipeline.push({ $match: matchFilter });
    }
    
    pipeline.push(
      {
        $group: {
          _id: '$country',
          avgIntensity: { $avg: '$intensity' },
          avgLikelihood: { $avg: '$likelihood' },
          avgRelevance: { $avg: '$relevance' },
          count: { $sum: 1 }
        }
      },
      { $sort: { count: -1 } }
    );
    
    const data = await DataModel.aggregate(pipeline);

    res.json({
      success: true,
      data
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching country data',
      error: error.message
    });
  }
});

// Get data grouped by topics
app.get('/api/data/by-topics', async (req, res) => {
  try {
    const matchFilter = buildMatchFilter(req.query);
    const pipeline = [];
    
    if (Object.keys(matchFilter).length > 0) {
      pipeline.push({ $match: matchFilter });
    }
    
    pipeline.push(
      {
        $group: {
          _id: '$topics',
          avgIntensity: { $avg: '$intensity' },
          avgLikelihood: { $avg: '$likelihood' },
          avgRelevance: { $avg: '$relevance' },
          count: { $sum: 1 }
        }
      },
      { $sort: { count: -1 } }
    );
    
    const data = await DataModel.aggregate(pipeline);

    res.json({
      success: true,
      data
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching topics data',
      error: error.message
    });
  }
});

// Get data grouped by region
app.get('/api/data/by-region', async (req, res) => {
  try {
    const matchFilter = buildMatchFilter(req.query);
    const pipeline = [];
    
    if (Object.keys(matchFilter).length > 0) {
      pipeline.push({ $match: matchFilter });
    }
    
    pipeline.push(
      {
        $group: {
          _id: '$region',
          avgIntensity: { $avg: '$intensity' },
          avgLikelihood: { $avg: '$likelihood' },
          avgRelevance: { $avg: '$relevance' },
          count: { $sum: 1 }
        }
      },
      { $sort: { count: -1 } }
    );
    
    const data = await DataModel.aggregate(pipeline);

    res.json({
      success: true,
      data
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching region data',
      error: error.message
    });
  }
});

// Get data grouped by city
app.get('/api/data/by-city', async (req, res) => {
  try {
    const matchFilter = buildMatchFilter(req.query);
    const pipeline = [];
    
    if (Object.keys(matchFilter).length > 0) {
      pipeline.push({ $match: matchFilter });
    }
    
    pipeline.push(
      {
        $group: {
          _id: '$city',
          avgIntensity: { $avg: '$intensity' },
          avgLikelihood: { $avg: '$likelihood' },
          avgRelevance: { $avg: '$relevance' },
          count: { $sum: 1 },
          country: { $first: '$country' }
        }
      },
      { $sort: { count: -1 } }
    );
    
    const data = await DataModel.aggregate(pipeline);

    res.json({
      success: true,
      data
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: 'Error fetching city data',
      error: error.message
    });
  }
});

const PORT = process.env.PORT || 5000;

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

