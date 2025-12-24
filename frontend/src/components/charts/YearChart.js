import React, { useState, useEffect } from 'react';
import { Bar } from 'react-chartjs-2';
import { Chart as ChartJS, CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend } from 'chart.js';
import Card from 'react-bootstrap/Card';
import axios from 'axios';

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend);

const YearChart = ({ apiUrl, filters }) => {
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

      const response = await axios.get(`${apiUrl}/data/by-year?${params.toString()}`);
      if (response.data.success) {
        const data = response.data.data;
        
        setChartData({
          labels: data.map(item => item._id),
          datasets: [
            {
              label: 'Avg Intensity',
              data: data.map(item => item.avgIntensity?.toFixed(2) || 0),
              backgroundColor: 'rgba(102, 126, 234, 0.6)',
              borderColor: 'rgba(102, 126, 234, 1)',
              borderWidth: 1
            },
            {
              label: 'Avg Likelihood',
              data: data.map(item => item.avgLikelihood?.toFixed(2) || 0),
              backgroundColor: 'rgba(255, 99, 132, 0.6)',
              borderColor: 'rgba(255, 99, 132, 1)',
              borderWidth: 1
            },
            {
              label: 'Avg Relevance',
              data: data.map(item => item.avgRelevance?.toFixed(2) || 0),
              backgroundColor: 'rgba(75, 192, 192, 0.6)',
              borderColor: 'rgba(75, 192, 192, 1)',
              borderWidth: 1
            }
          ]
        });
      }
    } catch (error) {
      console.error('Error fetching year data:', error);
    }
  };

  if (!chartData) return <Card className="chart-container"><Card.Body>Loading...</Card.Body></Card>;

  return (
    <Card className="chart-container">
      <Card.Body>
        <h5 className="chart-title">Trends by Year</h5>
        <Bar
          data={chartData}
          options={{
            responsive: true,
            maintainAspectRatio: true,
            plugins: {
              legend: {
                position: 'top'
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

export default YearChart;

