import React, { useState, useEffect } from 'react';
import { Bar } from 'react-chartjs-2';
import { Chart as ChartJS, CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend } from 'chart.js';
import Card from 'react-bootstrap/Card';
import axios from 'axios';

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend);

const CountryChart = ({ apiUrl, filters }) => {
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

      const response = await axios.get(`${apiUrl}/data/by-country?${params.toString()}`);
      if (response.data.success) {
        const data = response.data.data.slice(0, 10); // Top 10 countries
        
        setChartData({
          labels: data.map(item => item._id),
          datasets: [{
            label: 'Record Count',
            data: data.map(item => item.count),
            backgroundColor: 'rgba(54, 162, 235, 0.6)',
            borderColor: 'rgba(54, 162, 235, 1)',
            borderWidth: 1
          }]
        });
      }
    } catch (error) {
      console.error('Error fetching country data:', error);
    }
  };

  if (!chartData) return <Card className="chart-container"><Card.Body>Loading...</Card.Body></Card>;

  return (
    <Card className="chart-container">
      <Card.Body>
        <h5 className="chart-title">Top Countries by Record Count</h5>
        <Bar
          data={chartData}
          options={{
            responsive: true,
            maintainAspectRatio: true,
            indexAxis: 'y',
            plugins: {
              legend: {
                display: false
              }
            },
            scales: {
              x: {
                beginAtZero: true
              }
            }
          }}
        />
      </Card.Body>
    </Card>
  );
};

export default CountryChart;

