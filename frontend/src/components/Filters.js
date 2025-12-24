import React from 'react';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Form from 'react-bootstrap/Form';
import Button from 'react-bootstrap/Button';
import Card from 'react-bootstrap/Card';

const Filters = ({ filters, filterOptions, stats, onFilterChange, onClearFilters }) => {
  const maxYear = stats?.maxYear || new Date().getFullYear();
  const minYear = stats?.minYear || 2016;

  return (
    <Card className="filter-section">
      <Card.Header>
        <h4 className="mb-0">Filters</h4>
      </Card.Header>
      <Card.Body>
        <Row>
          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>End Year</Form.Label>
              <Form.Select
                value={filters.endYear}
                onChange={(e) => onFilterChange('endYear', e.target.value)}
              >
                <option value="">All Years</option>
                {Array.from({ length: maxYear - minYear + 1 }, (_, i) => minYear + i)
                  .reverse()
                  .map(year => (
                    <option key={year} value={year}>{year}</option>
                  ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>Topics</Form.Label>
              <Form.Select
                value={filters.topics}
                onChange={(e) => onFilterChange('topics', e.target.value)}
              >
                <option value="">All Topics</option>
                {filterOptions.topics.map(topic => (
                  <option key={topic} value={topic}>{topic}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>Sector</Form.Label>
              <Form.Select
                value={filters.sector}
                onChange={(e) => onFilterChange('sector', e.target.value)}
              >
                <option value="">All Sectors</option>
                {filterOptions.sectors.map(sector => (
                  <option key={sector} value={sector}>{sector}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>Region</Form.Label>
              <Form.Select
                value={filters.region}
                onChange={(e) => onFilterChange('region', e.target.value)}
              >
                <option value="">All Regions</option>
                {filterOptions.regions.map(region => (
                  <option key={region} value={region}>{region}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>PEST</Form.Label>
              <Form.Select
                value={filters.pest}
                onChange={(e) => onFilterChange('pest', e.target.value)}
              >
                <option value="">All PEST</option>
                {filterOptions.pests.map(pest => (
                  <option key={pest} value={pest}>{pest}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>Source</Form.Label>
              <Form.Select
                value={filters.source}
                onChange={(e) => onFilterChange('source', e.target.value)}
              >
                <option value="">All Sources</option>
                {filterOptions.sources.map(source => (
                  <option key={source} value={source}>{source}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>SWOT</Form.Label>
              <Form.Select
                value={filters.swot}
                onChange={(e) => onFilterChange('swot', e.target.value)}
              >
                <option value="">All SWOT</option>
                {filterOptions.swots.map(swot => (
                  <option key={swot} value={swot}>{swot}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>Country</Form.Label>
              <Form.Select
                value={filters.country}
                onChange={(e) => onFilterChange('country', e.target.value)}
              >
                <option value="">All Countries</option>
                {filterOptions.countries.map(country => (
                  <option key={country} value={country}>{country}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>

          <Col md={6} lg={3}>
            <Form.Group className="mb-3">
              <Form.Label>City</Form.Label>
              <Form.Select
                value={filters.city}
                onChange={(e) => onFilterChange('city', e.target.value)}
              >
                <option value="">All Cities</option>
                {filterOptions.cities.map(city => (
                  <option key={city} value={city}>{city}</option>
                ))}
              </Form.Select>
            </Form.Group>
          </Col>
        </Row>

        <Row>
          <Col>
            <Button variant="outline-danger" onClick={onClearFilters}>
              Clear All Filters
            </Button>
          </Col>
        </Row>
      </Card.Body>
    </Card>
  );
};

export default Filters;

