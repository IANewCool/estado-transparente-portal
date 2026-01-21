import React, { useState, useEffect } from 'react'

const API = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8080'

// Format number as Chilean pesos
function formatCLP(num) {
  if (num === null || num === undefined) return '-'
  return new Intl.NumberFormat('es-CL', {
    style: 'currency',
    currency: 'CLP',
    maximumFractionDigits: 0
  }).format(num)
}

// Format percentage
function formatPct(num) {
  if (num === null || num === undefined) return '-'
  const sign = num >= 0 ? '+' : ''
  return `${sign}${num.toFixed(1)}%`
}

export default function App() {
  // Data state
  const [metrics, setMetrics] = useState([])
  const [entities, setEntities] = useState([])
  const [selectedMetric, setSelectedMetric] = useState('')
  const [selectedEntity, setSelectedEntity] = useState('')
  const [entitySearch, setEntitySearch] = useState('')
  const [yearA, setYearA] = useState('2024')
  const [yearB, setYearB] = useState('2025')

  // Results state
  const [compareResult, setCompareResult] = useState(null)
  const [evidence, setEvidence] = useState(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  // Load metrics on mount
  useEffect(() => {
    fetch(`${API}/metrics`)
      .then(r => r.json())
      .then(data => setMetrics(data.metrics || []))
      .catch(e => console.error('Failed to load metrics:', e))
  }, [])

  // Search entities when typing
  useEffect(() => {
    if (entitySearch.length < 2) {
      setEntities([])
      return
    }
    const timeout = setTimeout(() => {
      fetch(`${API}/entities?query=${encodeURIComponent(entitySearch)}&limit=10`)
        .then(r => r.json())
        .then(data => setEntities(data.entities || []))
        .catch(e => console.error('Failed to search entities:', e))
    }, 300)
    return () => clearTimeout(timeout)
  }, [entitySearch])

  // Run comparison
  async function runCompare() {
    if (!selectedMetric) {
      setError('Selecciona una métrica')
      return
    }

    setError('')
    setLoading(true)
    setCompareResult(null)

    try {
      let url = `${API}/compare?metric_id=${selectedMetric}&year_a=${yearA}&year_b=${yearB}`
      if (selectedEntity) {
        url += `&entity_id=${selectedEntity}`
      }

      const res = await fetch(url)
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || 'Error en la comparación')
      }

      const data = await res.json()
      setCompareResult(data)
    } catch (e) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  // Open evidence modal
  async function openEvidence(factId) {
    if (!factId) return

    try {
      const res = await fetch(`${API}/evidence?fact_id=${factId}`)
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || 'Evidencia no encontrada')
      }
      const data = await res.json()
      setEvidence(data)
    } catch (e) {
      alert('Error: ' + e.message)
    }
  }

  return (
    <div style={{
      fontFamily: 'system-ui, -apple-system, sans-serif',
      padding: '20px',
      maxWidth: '1000px',
      margin: '0 auto',
      backgroundColor: '#fafafa',
      minHeight: '100vh'
    }}>
      {/* Header */}
      <header style={{ marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '28px', color: '#1a1a1a' }}>
          Estado Transparente
        </h1>
        <p style={{ margin: '8px 0 0', color: '#666', fontSize: '14px' }}>
          Portal ciudadano independiente. Cada dato tiene evidencia verificable.
        </p>
      </header>

      {/* Filters */}
      <div style={{
        backgroundColor: 'white',
        padding: '20px',
        borderRadius: '8px',
        boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
        marginBottom: '20px'
      }}>
        <h2 style={{ margin: '0 0 16px', fontSize: '18px' }}>Comparar por años</h2>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px' }}>
          {/* Metric selector */}
          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: 500 }}>
              Métrica *
            </label>
            <select
              value={selectedMetric}
              onChange={e => setSelectedMetric(e.target.value)}
              style={{
                width: '100%',
                padding: '10px',
                borderRadius: '4px',
                border: '1px solid #ddd',
                fontSize: '14px'
              }}
            >
              <option value="">Seleccionar métrica...</option>
              {metrics.map(m => (
                <option key={m.metric_id} value={m.metric_id}>
                  {m.display_name} ({m.unit})
                </option>
              ))}
            </select>
          </div>

          {/* Entity search */}
          <div style={{ position: 'relative' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: 500 }}>
              Entidad (opcional)
            </label>
            <input
              type="text"
              value={entitySearch}
              onChange={e => {
                setEntitySearch(e.target.value)
                setSelectedEntity('')
              }}
              placeholder="Buscar entidad..."
              style={{
                width: '100%',
                padding: '10px',
                borderRadius: '4px',
                border: '1px solid #ddd',
                fontSize: '14px',
                boxSizing: 'border-box'
              }}
            />
            {entities.length > 0 && !selectedEntity && (
              <div style={{
                position: 'absolute',
                top: '100%',
                left: 0,
                right: 0,
                backgroundColor: 'white',
                border: '1px solid #ddd',
                borderRadius: '4px',
                maxHeight: '200px',
                overflowY: 'auto',
                zIndex: 10,
                boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
              }}>
                {entities.map(e => (
                  <div
                    key={e.entity_id}
                    onClick={() => {
                      setSelectedEntity(e.entity_id)
                      setEntitySearch(e.display_name)
                      setEntities([])
                    }}
                    style={{
                      padding: '10px',
                      cursor: 'pointer',
                      borderBottom: '1px solid #eee'
                    }}
                    onMouseOver={ev => ev.target.style.backgroundColor = '#f5f5f5'}
                    onMouseOut={ev => ev.target.style.backgroundColor = 'white'}
                  >
                    {e.display_name}
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Year A */}
          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: 500 }}>
              Año A
            </label>
            <input
              type="number"
              value={yearA}
              onChange={e => setYearA(e.target.value)}
              min="2000"
              max="2030"
              style={{
                width: '100%',
                padding: '10px',
                borderRadius: '4px',
                border: '1px solid #ddd',
                fontSize: '14px',
                boxSizing: 'border-box'
              }}
            />
          </div>

          {/* Year B */}
          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: 500 }}>
              Año B
            </label>
            <input
              type="number"
              value={yearB}
              onChange={e => setYearB(e.target.value)}
              min="2000"
              max="2030"
              style={{
                width: '100%',
                padding: '10px',
                borderRadius: '4px',
                border: '1px solid #ddd',
                fontSize: '14px',
                boxSizing: 'border-box'
              }}
            />
          </div>
        </div>

        <button
          onClick={runCompare}
          disabled={loading || !selectedMetric}
          style={{
            marginTop: '16px',
            padding: '12px 24px',
            backgroundColor: loading ? '#ccc' : '#2563eb',
            color: 'white',
            border: 'none',
            borderRadius: '6px',
            fontSize: '14px',
            fontWeight: 500,
            cursor: loading ? 'not-allowed' : 'pointer'
          }}
        >
          {loading ? 'Comparando...' : 'Comparar'}
        </button>

        {error && (
          <p style={{ color: '#dc2626', marginTop: '12px', fontSize: '14px' }}>
            {error}
          </p>
        )}
      </div>

      {/* Results */}
      {compareResult && (
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 1px 3px rgba(0,0,0,0.1)'
        }}>
          <h2 style={{ margin: '0 0 16px', fontSize: '18px' }}>
            Resultados: {compareResult.year_a} vs {compareResult.year_b}
          </h2>

          {compareResult.rows.length === 0 ? (
            <p style={{ color: '#666' }}>No hay datos para comparar en estos años.</p>
          ) : (
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
                <thead>
                  <tr style={{ borderBottom: '2px solid #e5e7eb' }}>
                    <th style={{ textAlign: 'left', padding: '12px 8px', fontWeight: 600 }}>Entidad</th>
                    <th style={{ textAlign: 'right', padding: '12px 8px', fontWeight: 600 }}>{compareResult.year_a}</th>
                    <th style={{ textAlign: 'right', padding: '12px 8px', fontWeight: 600 }}>{compareResult.year_b}</th>
                    <th style={{ textAlign: 'right', padding: '12px 8px', fontWeight: 600 }}>Δ</th>
                    <th style={{ textAlign: 'right', padding: '12px 8px', fontWeight: 600 }}>%</th>
                    <th style={{ textAlign: 'center', padding: '12px 8px', fontWeight: 600 }}>Evidencia</th>
                  </tr>
                </thead>
                <tbody>
                  {compareResult.rows.map((row, idx) => (
                    <tr
                      key={idx}
                      style={{
                        borderBottom: '1px solid #e5e7eb',
                        backgroundColor: idx % 2 === 0 ? 'white' : '#f9fafb'
                      }}
                    >
                      <td style={{ padding: '12px 8px' }}>{row.entity_name}</td>
                      <td style={{ textAlign: 'right', padding: '12px 8px' }}>
                        {formatCLP(row.value_a)}
                      </td>
                      <td style={{ textAlign: 'right', padding: '12px 8px' }}>
                        {formatCLP(row.value_b)}
                      </td>
                      <td style={{
                        textAlign: 'right',
                        padding: '12px 8px',
                        color: row.delta > 0 ? '#059669' : row.delta < 0 ? '#dc2626' : '#666'
                      }}>
                        {row.delta !== null ? formatCLP(row.delta) : '-'}
                      </td>
                      <td style={{
                        textAlign: 'right',
                        padding: '12px 8px',
                        color: row.pct_change > 0 ? '#059669' : row.pct_change < 0 ? '#dc2626' : '#666'
                      }}>
                        {formatPct(row.pct_change)}
                      </td>
                      <td style={{ textAlign: 'center', padding: '12px 8px' }}>
                        {row.fact_id_a && (
                          <button
                            onClick={() => openEvidence(row.fact_id_a)}
                            style={{
                              padding: '4px 8px',
                              fontSize: '12px',
                              backgroundColor: '#f3f4f6',
                              border: '1px solid #d1d5db',
                              borderRadius: '4px',
                              cursor: 'pointer',
                              marginRight: '4px'
                            }}
                            title={`Ver evidencia año ${compareResult.year_a}`}
                          >
                            A
                          </button>
                        )}
                        {row.fact_id_b && (
                          <button
                            onClick={() => openEvidence(row.fact_id_b)}
                            style={{
                              padding: '4px 8px',
                              fontSize: '12px',
                              backgroundColor: '#f3f4f6',
                              border: '1px solid #d1d5db',
                              borderRadius: '4px',
                              cursor: 'pointer'
                            }}
                            title={`Ver evidencia año ${compareResult.year_b}`}
                          >
                            B
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Evidence Modal */}
      {evidence && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0,0,0,0.5)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 100
        }} onClick={() => setEvidence(null)}>
          <div
            style={{
              backgroundColor: 'white',
              padding: '24px',
              borderRadius: '8px',
              maxWidth: '600px',
              width: '90%',
              maxHeight: '80vh',
              overflowY: 'auto'
            }}
            onClick={e => e.stopPropagation()}
          >
            <h3 style={{ margin: '0 0 16px', fontSize: '18px' }}>
              Evidencia Verificable
            </h3>

            <div style={{ fontSize: '14px' }}>
              <div style={{ marginBottom: '12px' }}>
                <strong>Fuente original:</strong><br />
                <a
                  href={evidence.artifact?.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{ color: '#2563eb', wordBreak: 'break-all' }}
                >
                  {evidence.artifact?.url}
                </a>
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Fecha de captura:</strong><br />
                {new Date(evidence.artifact?.captured_at).toLocaleString('es-CL')}
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Hash SHA-256:</strong><br />
                <code style={{
                  fontSize: '12px',
                  backgroundColor: '#f3f4f6',
                  padding: '4px 8px',
                  borderRadius: '4px',
                  wordBreak: 'break-all'
                }}>
                  {evidence.artifact?.content_hash}
                </code>
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Ubicación en archivo:</strong><br />
                <code style={{
                  fontSize: '12px',
                  backgroundColor: '#f3f4f6',
                  padding: '4px 8px',
                  borderRadius: '4px'
                }}>
                  {evidence.location || 'No especificada'}
                </code>
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Método de extracción:</strong><br />
                {evidence.method}
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Tamaño:</strong><br />
                {(evidence.artifact?.size_bytes / 1024).toFixed(1)} KB
              </div>
            </div>

            <div style={{
              marginTop: '20px',
              padding: '12px',
              backgroundColor: '#fef3c7',
              borderRadius: '6px',
              fontSize: '13px'
            }}>
              <strong>Cómo verificar:</strong><br />
              1. Descarga el archivo original desde la URL<br />
              2. Calcula el hash SHA-256 del archivo<br />
              3. Compara con el hash mostrado arriba<br />
              4. Busca el dato en la ubicación indicada
            </div>

            <button
              onClick={() => setEvidence(null)}
              style={{
                marginTop: '16px',
                padding: '10px 20px',
                backgroundColor: '#374151',
                color: 'white',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontSize: '14px'
              }}
            >
              Cerrar
            </button>
          </div>
        </div>
      )}

      {/* Footer */}
      <footer style={{
        marginTop: '40px',
        paddingTop: '20px',
        borderTop: '1px solid #e5e7eb',
        fontSize: '13px',
        color: '#666'
      }}>
        <p>
          <strong>Estado Transparente</strong> es un portal ciudadano independiente.
          No recibimos financiamiento del Estado ni de entidades fiscalizadas.
        </p>
        <p>
          Cada dato mostrado tiene una cadena de evidencia verificable: URL original,
          fecha de captura, hash SHA-256, y ubicación exacta en el archivo fuente.
        </p>
        <p style={{ marginTop: '12px' }}>
          <a href="/financiamiento" style={{ color: '#2563eb', marginRight: '16px' }}>
            Cómo nos financiamos
          </a>
          <a href="https://github.com/estado-transparente" target="_blank" rel="noopener noreferrer" style={{ color: '#2563eb' }}>
            Código fuente (GitHub)
          </a>
        </p>
      </footer>
    </div>
  )
}
