import React, { useState, useEffect } from 'react';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Form from 'react-bootstrap/Form';
import Button from 'react-bootstrap/Button';
import Card from 'react-bootstrap/Card';
import axios from 'axios';
import Filters from './Filters';
import StatsCards from './StatsCards';
import IntensityChart from './charts/IntensityChart';
import LikelihoodChart from './charts/LikelihoodChart';
import RelevanceChart from './charts/RelevanceChart';
import YearChart from './charts/YearChart';
import CountryChart from './charts/CountryChart';
import TopicsChart from './charts/TopicsChart';
import RegionChart from './charts/RegionChart';
import CityChart from './charts/CityChart';

const Dashboard = ({ apiUrl }) => {
  const [filters, setFilters] = useState({
    endYear: '',
    topics: '',
    sector: '',
    region: '',
    pest: '',
    source: '',
    swot: '',
    country: '',
    city: ''
  });

  const [filterOptions, setFilterOptions] = useState({
    topics: [],
    sectors: [],
    regions: [],
    pests: [],
    sources: [],
    swots: [],
    countries: [],
    cities: []
  });

  const [stats, setStats] = useState(null);
  const [data, setData] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchStats();
    fetchData();
  }, []);

  useEffect(() => {
    fetchData();
  }, [filters]);

  const fetchStats = async () => {
    try {
      const response = await axios.get(`${apiUrl}/stats`);
      if (response.data.success) {
        setStats(response.data.stats);
        setFilterOptions(response.data.filters);
      }
    } catch (error) {
      console.error('Error fetching stats:', error);
    }
  };

  const fetchData = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      Object.keys(filters).forEach(key => {
        if (filters[key]) {
          params.append(key, filters[key]);
        }
      });

      const response = await axios.get(`${apiUrl}/data?${params.toString()}`);
      if (response.data.success) {
        setData(response.data.data);
      }
    } catch (error) {
      console.error('Error fetching data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleFilterChange = (name, value) => {
    setFilters(prev => ({
      ...prev,
      [name]: value
    }));
  };

  const clearFilters = () => {
    setFilters({
      endYear: '',
      topics: '',
      sector: '',
      region: '',
      pest: '',
      source: '',
      swot: '',
      country: '',
      city: ''
    });
  };

  return (
    <div>
      <Filters
        filters={filters}
        filterOptions={filterOptions}
        stats={stats}
        onFilterChange={handleFilterChange}
        onClearFilters={clearFilters}
      />

      <StatsCards stats={stats} data={data} />

      <Row>
        <Col md={6} lg={4}>
          <IntensityChart apiUrl={apiUrl} filters={filters} />
        </Col>
        <Col md={6} lg={4}>
          <LikelihoodChart apiUrl={apiUrl} filters={filters} />
        </Col>
        <Col md={6} lg={4}>
          <RelevanceChart apiUrl={apiUrl} filters={filters} />
        </Col>
      </Row>

      <Row>
        <Col md={12} lg={6}>
          <YearChart apiUrl={apiUrl} filters={filters} />
        </Col>
        <Col md={12} lg={6}>
          <CountryChart apiUrl={apiUrl} filters={filters} />
        </Col>
      </Row>

      <Row>
        <Col md={12} lg={6}>
          <TopicsChart apiUrl={apiUrl} filters={filters} />
        </Col>
        <Col md={12} lg={6}>
          <RegionChart apiUrl={apiUrl} filters={filters} />
        </Col>
      </Row>

      <Row>
        <Col md={12}>
          <CityChart apiUrl={apiUrl} filters={filters} />
        </Col>
      </Row>
    </div>
  );
};

export default Dashboard;

