/**
 * React компонент для интеграции с Payment Analytics API
 * Требуется: npm install recharts
 */

import React, { useState } from 'react';
import { BarChart, Bar, LineChart, Line, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface QueryRequest {
  question: string;
  include_analysis?: boolean;
  use_cache?: boolean;
}

interface Insight {
  title: string;
  description: string;
  significance: 'High' | 'Medium' | 'Low';
}

interface AnalysisResult {
  headline: string;
  insights: Insight[];
  explanation: string;
  suggested_questions: string[];
  chart_type?: 'Bar' | 'Line' | 'Pie' | 'Table' | 'Trend';
  data: any[];
}

interface QueryResponse {
  question: string;
  sql: string;
  data: any[];
  execution_time_ms: number;
  row_count: number;
  analysis?: AnalysisResult;
  cached: boolean;
}

const API_URL = 'http://localhost:3000/api/query';

async function queryDatabase(request: QueryRequest): Promise<QueryResponse> {
  const response = await fetch(API_URL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });
  
  if (!response.ok) {
    throw new Error(`Query failed: ${response.statusText}`);
  }
  
  return response.json();
}

export function PaymentAnalyticsQuery() {
  const [question, setQuestion] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<QueryResponse | null>(null);
  const [includeAnalysis, setIncludeAnalysis] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    
    try {
      const response = await queryDatabase({
        question,
        include_analysis: includeAnalysis,
        use_cache: true,
      });
      setResult(response);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const renderChart = () => {
    if (!result?.analysis?.chart_type || !result.data.length) return null;

    const chartType = result.analysis.chart_type;
    const data = result.data;

    switch (chartType) {
      case 'Bar':
        return (
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={data}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey={Object.keys(data[0] || {})[0]} />
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey={Object.keys(data[0] || {})[1] || 'value'} fill="#8884d8" />
            </BarChart>
          </ResponsiveContainer>
        );
      
      case 'Line':
        return (
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey={Object.keys(data[0] || {})[0]} />
              <YAxis />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey={Object.keys(data[0] || {})[1] || 'value'} stroke="#8884d8" />
            </LineChart>
          </ResponsiveContainer>
        );
      
      case 'Pie':
        const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884d8'];
        return (
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={data}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name}: ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey={Object.keys(data[0] || {})[1] || 'value'}
              >
                {data.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        );
      
      default:
        return null;
    }
  };

  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '20px' }}>
      <h1>📊 Payment Analytics Query</h1>
      
      <form onSubmit={handleSubmit} style={{ marginBottom: '20px' }}>
        <div style={{ display: 'flex', gap: '10px', marginBottom: '10px' }}>
          <input
            type="text"
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            placeholder="Задайте вопрос о данных..."
            style={{ flex: 1, padding: '10px', fontSize: '16px' }}
            disabled={loading}
          />
          <button
            type="submit"
            disabled={loading || !question.trim()}
            style={{ padding: '10px 20px', fontSize: '16px' }}
          >
            {loading ? 'Загрузка...' : 'Отправить'}
          </button>
        </div>
        
        <label style={{ display: 'flex', alignItems: 'center', gap: '5px' }}>
          <input
            type="checkbox"
            checked={includeAnalysis}
            onChange={(e) => setIncludeAnalysis(e.target.checked)}
            disabled={loading}
          />
          Включить LLM-анализ
        </label>
      </form>

      {error && (
        <div style={{ padding: '10px', background: '#fee', color: '#c00', borderRadius: '5px', marginBottom: '20px' }}>
          ❌ Ошибка: {error}
        </div>
      )}

      {result && (
        <div>
          {/* Анализ */}
          {result.analysis && (
            <div style={{ marginBottom: '30px', padding: '20px', background: '#f5f5f5', borderRadius: '5px' }}>
              <h2>{result.analysis.headline}</h2>
              
              {/* Инсайты */}
              <div style={{ marginTop: '20px' }}>
                {result.analysis.insights.map((insight, i) => (
                  <div
                    key={i}
                    style={{
                      marginBottom: '15px',
                      padding: '15px',
                      background: 'white',
                      borderRadius: '5px',
                      borderLeft: `4px solid ${
                        insight.significance === 'High' ? '#f00' :
                        insight.significance === 'Medium' ? '#fa0' : '#0a0'
                      }`
                    }}
                  >
                    <h3 style={{ margin: '0 0 5px 0' }}>{insight.title}</h3>
                    <p style={{ margin: 0, color: '#666' }}>{insight.description}</p>
                  </div>
                ))}
              </div>
              
              {/* Объяснение */}
              <p style={{ marginTop: '20px', lineHeight: '1.6' }}>
                {result.analysis.explanation}
              </p>
              
              {/* Диаграмма */}
              {result.analysis.chart_type && result.data.length > 0 && (
                <div style={{ marginTop: '30px' }}>
                  <h3>Диаграмма ({result.analysis.chart_type})</h3>
                  {renderChart()}
                </div>
              )}
            </div>
          )}
          
          {/* Таблица данных */}
          {result.data.length > 0 && (
            <div style={{ marginTop: '30px' }}>
              <h3>Данные ({result.row_count} строк)</h3>
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', background: 'white' }}>
                  <thead>
                    <tr style={{ background: '#f0f0f0' }}>
                      {Object.keys(result.data[0]).map(key => (
                        <th key={key} style={{ padding: '10px', textAlign: 'left', border: '1px solid #ddd' }}>
                          {key}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {result.data.map((row, i) => (
                      <tr key={i}>
                        {Object.values(row).map((value, j) => (
                          <td key={j} style={{ padding: '10px', border: '1px solid #ddd' }}>
                            {String(value)}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
          
          {/* Предложенные вопросы */}
          {result.analysis?.suggested_questions && result.analysis.suggested_questions.length > 0 && (
            <div style={{ marginTop: '30px', padding: '20px', background: '#e3f2fd', borderRadius: '5px' }}>
              <h3>❓ Следующие вопросы:</h3>
              <ul style={{ listStyle: 'none', padding: 0 }}>
                {result.analysis.suggested_questions.map((q, i) => (
                  <li key={i} style={{ marginBottom: '10px' }}>
                    <button
                      onClick={() => {
                        setQuestion(q);
                        handleSubmit({ preventDefault: () => {} } as React.FormEvent);
                      }}
                      style={{
                        padding: '8px 15px',
                        background: '#2196F3',
                        color: 'white',
                        border: 'none',
                        borderRadius: '5px',
                        cursor: 'pointer'
                      }}
                    >
                      {q}
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}
          
          {/* Метаинформация */}
          <div style={{ marginTop: '20px', padding: '10px', background: '#f9f9f9', borderRadius: '5px', fontSize: '14px', color: '#666' }}>
            ⚡ Время выполнения: {result.execution_time_ms}ms
            {result.cached && ' | 💾 Результат из кэша'}
            <br />
            📝 SQL: <code style={{ background: '#eee', padding: '2px 5px', borderRadius: '3px' }}>{result.sql}</code>
          </div>
        </div>
      )}
    </div>
  );
}

