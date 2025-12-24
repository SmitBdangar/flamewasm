import React, { useState, useEffect } from 'react';
import { Line } from 'react-chartjs-2';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend } from 'chart.js';
import Card from 'react-bootstrap/Card';
import axios from 'axios';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend);

const RelevanceChart = ({ apiUrl, filters }) => {
  const [chartData, setChartData] = useState(null);

  useEffect(() => {
    fetchData();
  }, [filters]);

  const fetchData = async () => {
    try {
      const params = new URLSearchParams();
      Object.keys(filters).forEach(key => {
        if (filters[key]) {
          params.append(key, filters[key]);
        }
      });

      const response = await axios.get(`${apiUrl}/data?${params.toString()}`);
      if (response.data.success) {
        const data = response.data.data;
        
        const relevanceCounts = {};
        data.forEach(item => {
          const relevance = item.relevance || 0;
          relevanceCounts[relevance] = (relevanceCounts[relevance] || 0) + 1;
        });

        const sortedKeys = Object.keys(relevanceCounts).sort((a, b) => a - b);

        setChartData({
          labels: sortedKeys.map(k => `Level ${k}`),
          datasets: [{
            label: 'Relevance Count',
            data: sortedKeys.map(k => relevanceCounts[k]),
            borderColor: 'rgba(75, 192, 192, 1)',
            backgroundColor: 'rgba(75, 192, 192, 0.2)',
            tension: 0.4,
            fill: true
          }]
        });
      }
    } catch (error) {
      console.error('Error fetching relevance data:', error);
    }
  };

  if (!chartData) return <Card className="chart-container"><Card.Body>Loading...</Card.Body></Card>;

  return (
    <Card className="chart-container">
      <Card.Body>
        <h5 className="chart-title">Relevance Distribution</h5>
        <Line
          data={chartData}
          options={{
            responsive: true,
            maintainAspectRatio: true,
            plugins: {
              legend: {
                display: false
              }
            },
            scales: {
              y: {
                beginAtZero: true
              }
            }
          }}
        />
      </Card.Body>
    </Card>
  );
};

export default RelevanceChart;

