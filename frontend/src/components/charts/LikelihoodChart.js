import React, { useState, useEffect } from 'react';
import { Doughnut } from 'react-chartjs-2';
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js';
import Card from 'react-bootstrap/Card';
import axios from 'axios';

ChartJS.register(ArcElement, Tooltip, Legend);

const LikelihoodChart = ({ apiUrl, filters }) => {
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
        
        const likelihoodCounts = {};
        data.forEach(item => {
          const likelihood = item.likelihood || 0;
          likelihoodCounts[likelihood] = (likelihoodCounts[likelihood] || 0) + 1;
        });

        setChartData({
          labels: Object.keys(likelihoodCounts).sort((a, b) => a - b).map(k => `Level ${k}`),
          datasets: [{
            label: 'Likelihood',
            data: Object.keys(likelihoodCounts).sort((a, b) => a - b).map(k => likelihoodCounts[k]),
            backgroundColor: [
              'rgba(255, 99, 132, 0.6)',
              'rgba(54, 162, 235, 0.6)',
              'rgba(255, 206, 86, 0.6)',
              'rgba(75, 192, 192, 0.6)',
              'rgba(153, 102, 255, 0.6)',
              'rgba(255, 159, 64, 0.6)'
            ],
            borderColor: [
              'rgba(255, 99, 132, 1)',
              'rgba(54, 162, 235, 1)',
              'rgba(255, 206, 86, 1)',
              'rgba(75, 192, 192, 1)',
              'rgba(153, 102, 255, 1)',
              'rgba(255, 159, 64, 1)'
            ],
            borderWidth: 1
          }]
        });
      }
    } catch (error) {
      console.error('Error fetching likelihood data:', error);
    }
  };

  if (!chartData) return <Card className="chart-container"><Card.Body>Loading...</Card.Body></Card>;

  return (
    <Card className="chart-container">
      <Card.Body>
        <h5 className="chart-title">Likelihood Distribution</h5>
        <Doughnut
          data={chartData}
          options={{
            responsive: true,
            maintainAspectRatio: true,
            plugins: {
              legend: {
                position: 'bottom'
              }
            }
          }}
        />
      </Card.Body>
    </Card>
  );
};

export default LikelihoodChart;

