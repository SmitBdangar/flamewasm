const mongoose = require('mongoose');
const fs = require('fs');
const path = require('path');
require('dotenv').config();

const MONGODB_URI = process.env.MONGODB_URI || 'mongodb://localhost:27017/visualization_dashboard';

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

async function seedDatabase() {
  try {
    // Connect to MongoDB
    await mongoose.connect(MONGODB_URI, {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    });
    console.log('Connected to MongoDB');

    // Read JSON file
    const jsonPath = path.join(__dirname, '../../jsondata.json');
    const jsonData = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));

    // Clear existing data
    await DataModel.deleteMany({});
    console.log('Cleared existing data');

    // Insert new data
    await DataModel.insertMany(jsonData);
    console.log(`Inserted ${jsonData.length} records`);

    // Verify insertion
    const count = await DataModel.countDocuments();
    console.log(`Total records in database: ${count}`);

    process.exit(0);
  } catch (error) {
    console.error('Error seeding database:', error);
    process.exit(1);
  }
}

seedDatabase();

