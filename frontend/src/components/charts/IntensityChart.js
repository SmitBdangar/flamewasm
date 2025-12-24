import React, { useState, useEffect } from 'react';
import { Bar } from 'react-chartjs-2';
import { Chart as ChartJS, CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend } from 'chart.js';
import Card from 'react-bootstrap/Card';
import axios from 'axios';

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend);

const IntensityChart = ({ apiUrl, filters }) => {
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
        
        const intensityRanges = {
          '0-2': 0,
          '3-4': 0,
          '5-6': 0,
          '7-8': 0,
          '9-10': 0
        };

        data.forEach(item => {
          const intensity = item.intensity || 0;
          if (intensity <= 2) intensityRanges['0-2']++;
          else if (intensity <= 4) intensityRanges['3-4']++;
          else if (intensity <= 6) intensityRanges['5-6']++;
          else if (intensity <= 8) intensityRanges['7-8']++;
          else intensityRanges['9-10']++;
        });

        setChartData({
          labels: Object.keys(intensityRanges),
          datasets: [{
            label: 'Intensity Distribution',
            data: Object.values(intensityRanges),
            backgroundColor: 'rgba(102, 126, 234, 0.6)',
            borderColor: 'rgba(102, 126, 234, 1)',
            borderWidth: 1
          }]
        });
      }
    } catch (error) {
      console.error('Error fetching intensity data:', error);
    }
  };

  if (!chartData) return <Card className="chart-container"><Card.Body>Loading...</Card.Body></Card>;

  return (
    <Card className="chart-container">
      <Card.Body>
        <h5 className="chart-title">Intensity Distribution</h5>
        <Bar
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

export default IntensityChart;

