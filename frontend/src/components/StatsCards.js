import React from 'react';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Card from 'react-bootstrap/Card';

const StatsCards = ({ stats, data }) => {
  if (!stats) return null;

  const currentStats = {
    totalRecords: data.length,
    avgIntensity: data.length > 0 
      ? (data.reduce((sum, item) => sum + (item.intensity || 0), 0) / data.length).toFixed(2)
      : stats.avgIntensity?.toFixed(2) || '0',
    avgLikelihood: data.length > 0
      ? (data.reduce((sum, item) => sum + (item.likelihood || 0), 0) / data.length).toFixed(2)
      : stats.avgLikelihood?.toFixed(2) || '0',
    avgRelevance: data.length > 0
      ? (data.reduce((sum, item) => sum + (item.relevance || 0), 0) / data.length).toFixed(2)
      : stats.avgRelevance?.toFixed(2) || '0'
  };

  return (
    <Row className="mb-4">
      <Col md={3}>
        <Card className="stats-card text-white" style={{ background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)' }}>
          <Card.Body>
            <h5>Total Records</h5>
            <h2>{currentStats.totalRecords}</h2>
          </Card.Body>
        </Card>
      </Col>
      <Col md={3}>
        <Card className="stats-card text-white" style={{ background: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)' }}>
          <Card.Body>
            <h5>Avg Intensity</h5>
            <h2>{currentStats.avgIntensity}</h2>
          </Card.Body>
        </Card>
      </Col>
      <Col md={3}>
        <Card className="stats-card text-white" style={{ background: 'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)' }}>
          <Card.Body>
            <h5>Avg Likelihood</h5>
            <h2>{currentStats.avgLikelihood}</h2>
          </Card.Body>
        </Card>
      </Col>
      <Col md={3}>
        <Card className="stats-card text-white" style={{ background: 'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)' }}>
          <Card.Body>
            <h5>Avg Relevance</h5>
            <h2>{currentStats.avgRelevance}</h2>
          </Card.Body>
        </Card>
      </Col>
    </Row>
  );
};

export default StatsCards;

