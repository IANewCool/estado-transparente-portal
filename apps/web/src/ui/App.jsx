import React, { useState, useEffect } from 'react'

const API = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8080'

// ============================================================================
// Utility Functions
// ============================================================================

function formatCLP(num) {
  if (num === null || num === undefined) return '-'
  return new Intl.NumberFormat('es-CL', {
    style: 'currency',
    currency: 'CLP',
    maximumFractionDigits: 0
  }).format(num)
}

function formatPct(num, showSign = true) {
  if (num === null || num === undefined) return '-'
  const sign = showSign && num >= 0 ? '+' : ''
  return `${sign}${num.toFixed(1)}%`
}

function formatBillones(num) {
  if (num === null || num === undefined) return '-'
  const billones = num / 1_000_000_000_000
  return `$${billones.toFixed(2)} billones`
}

// ============================================================================
// Styles
// ============================================================================

const styles = {
  container: {
    fontFamily: 'system-ui, -apple-system, sans-serif',
    padding: '16px',
    maxWidth: '1200px',
    margin: '0 auto',
    backgroundColor: '#f8fafc',
    minHeight: '100vh',
  },
  header: {
    marginBottom: '24px',
    textAlign: 'center',
  },
  title: {
    margin: 0,
    fontSize: '28px',
    fontWeight: 700,
    color: '#0f172a',
  },
  subtitle: {
    margin: '8px 0 0',
    color: '#64748b',
    fontSize: '14px',
  },
  card: {
    backgroundColor: 'white',
    padding: '20px',
    borderRadius: '12px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
    marginBottom: '16px',
  },
  summaryGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '16px',
    marginBottom: '24px',
  },
  summaryCard: {
    backgroundColor: 'white',
    padding: '20px',
    borderRadius: '12px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
    textAlign: 'center',
  },
  summaryValue: {
    fontSize: '24px',
    fontWeight: 700,
    color: '#0f172a',
    margin: '8px 0',
  },
  summaryLabel: {
    fontSize: '13px',
    color: '#64748b',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  yearSelector: {
    display: 'flex',
    justifyContent: 'center',
    gap: '8px',
    marginBottom: '24px',
    flexWrap: 'wrap',
  },
  yearButton: (active) => ({
    padding: '8px 16px',
    border: 'none',
    borderRadius: '20px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
    backgroundColor: active ? '#3b82f6' : '#e2e8f0',
    color: active ? 'white' : '#475569',
    transition: 'all 0.2s',
  }),
  barContainer: {
    marginBottom: '12px',
  },
  barHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '6px',
    fontSize: '14px',
  },
  barName: {
    fontWeight: 500,
    color: '#1e293b',
    whiteSpace: 'nowrap',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    maxWidth: '60%',
  },
  barValue: {
    color: '#64748b',
    fontSize: '13px',
  },
  barTrack: {
    height: '24px',
    backgroundColor: '#e2e8f0',
    borderRadius: '4px',
    overflow: 'hidden',
  },
  barFill: (pct, color) => ({
    height: '100%',
    width: `${Math.min(pct, 100)}%`,
    backgroundColor: color || '#3b82f6',
    borderRadius: '4px',
    transition: 'width 0.5s ease-out',
    display: 'flex',
    alignItems: 'center',
    paddingLeft: pct > 10 ? '8px' : '0',
  }),
  barPct: {
    fontSize: '12px',
    fontWeight: 600,
    color: 'white',
  },
  tabs: {
    display: 'flex',
    gap: '4px',
    marginBottom: '20px',
    borderBottom: '1px solid #e2e8f0',
    paddingBottom: '4px',
  },
  tab: (active) => ({
    padding: '10px 20px',
    border: 'none',
    backgroundColor: 'transparent',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
    color: active ? '#3b82f6' : '#64748b',
    borderBottom: active ? '2px solid #3b82f6' : '2px solid transparent',
    marginBottom: '-5px',
  }),
  select: {
    padding: '10px 12px',
    borderRadius: '8px',
    border: '1px solid #e2e8f0',
    fontSize: '14px',
    backgroundColor: 'white',
    minWidth: '180px',
  },
  input: {
    padding: '10px 12px',
    borderRadius: '8px',
    border: '1px solid #e2e8f0',
    fontSize: '14px',
    width: '100%',
    boxSizing: 'border-box',
  },
  button: (primary = true, disabled = false) => ({
    padding: '12px 24px',
    border: 'none',
    borderRadius: '8px',
    fontSize: '14px',
    fontWeight: 600,
    cursor: disabled ? 'not-allowed' : 'pointer',
    backgroundColor: disabled ? '#cbd5e1' : primary ? '#3b82f6' : '#e2e8f0',
    color: primary ? 'white' : '#475569',
  }),
  table: {
    width: '100%',
    borderCollapse: 'collapse',
    fontSize: '14px',
  },
  th: {
    textAlign: 'left',
    padding: '12px 8px',
    fontWeight: 600,
    borderBottom: '2px solid #e2e8f0',
    color: '#475569',
  },
  thRight: {
    textAlign: 'right',
    padding: '12px 8px',
    fontWeight: 600,
    borderBottom: '2px solid #e2e8f0',
    color: '#475569',
  },
  td: {
    padding: '12px 8px',
    borderBottom: '1px solid #e2e8f0',
  },
  tdRight: {
    textAlign: 'right',
    padding: '12px 8px',
    borderBottom: '1px solid #e2e8f0',
  },
  positive: { color: '#16a34a' },
  negative: { color: '#dc2626' },
  loading: {
    textAlign: 'center',
    padding: '40px',
    color: '#64748b',
  },
  error: {
    color: '#dc2626',
    padding: '12px',
    backgroundColor: '#fef2f2',
    borderRadius: '8px',
    marginTop: '12px',
  },
  footer: {
    marginTop: '40px',
    paddingTop: '20px',
    borderTop: '1px solid #e2e8f0',
    fontSize: '13px',
    color: '#64748b',
    textAlign: 'center',
  },
  link: {
    color: '#3b82f6',
    textDecoration: 'none',
  },
  modal: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0,0,0,0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 100,
    padding: '16px',
  },
  modalContent: {
    backgroundColor: 'white',
    padding: '24px',
    borderRadius: '12px',
    maxWidth: '600px',
    width: '100%',
    maxHeight: '80vh',
    overflowY: 'auto',
  },
}

// Color palette for bars
const BAR_COLORS = [
  '#3b82f6', '#8b5cf6', '#ec4899', '#f97316', '#eab308',
  '#22c55e', '#14b8a6', '#06b6d4', '#6366f1', '#a855f7',
]

// ============================================================================
// Components
// ============================================================================

function Dashboard({ onCompareClick }) {
  const [data, setData] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [selectedYear, setSelectedYear] = useState(null)

  useEffect(() => {
    fetchDashboard()
  }, [selectedYear])

  async function fetchDashboard() {
    setLoading(true)
    setError('')
    try {
      const url = selectedYear
        ? `${API}/dashboard?year=${selectedYear}`
        : `${API}/dashboard`
      const res = await fetch(url)
      if (!res.ok) throw new Error('Error al cargar datos')
      const json = await res.json()
      setData(json)
      if (!selectedYear) setSelectedYear(json.year)
    } catch (e) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  if (loading && !data) {
    return <div style={styles.loading}>Cargando datos...</div>
  }

  if (error) {
    return <div style={styles.error}>{error}</div>
  }

  if (!data) return null

  // Get top 10 entities for the chart
  const topEntities = data.entities.slice(0, 10)
  const maxPct = topEntities[0]?.percentage || 100

  return (
    <div>
      {/* Year Selector */}
      <div style={styles.yearSelector}>
        {data.available_years.map(year => (
          <button
            key={year}
            onClick={() => setSelectedYear(year)}
            style={styles.yearButton(year === selectedYear)}
          >
            {year}
          </button>
        ))}
      </div>

      {/* Summary Cards */}
      <div style={styles.summaryGrid}>
        <div style={styles.summaryCard}>
          <div style={styles.summaryLabel}>Presupuesto Total {data.year}</div>
          <div style={styles.summaryValue}>{data.total_formatted}</div>
          {data.yoy_change_pct !== null && (
            <div style={data.yoy_change_pct >= 0 ? styles.positive : styles.negative}>
              {formatPct(data.yoy_change_pct)} vs {data.previous_year}
            </div>
          )}
        </div>
        <div style={styles.summaryCard}>
          <div style={styles.summaryLabel}>Ministerios/Servicios</div>
          <div style={styles.summaryValue}>{data.entities.length}</div>
          <div style={{ color: '#64748b', fontSize: '13px' }}>partidas presupuestarias</div>
        </div>
        <div style={styles.summaryCard}>
          <div style={styles.summaryLabel}>Mayor Presupuesto</div>
          <div style={{ ...styles.summaryValue, fontSize: '18px' }}>
            {topEntities[0]?.display_name || '-'}
          </div>
          <div style={{ color: '#64748b', fontSize: '13px' }}>
            {formatPct(topEntities[0]?.percentage, false)} del total
          </div>
        </div>
      </div>

      {/* Bar Chart */}
      <div style={styles.card}>
        <h3 style={{ margin: '0 0 20px', fontSize: '16px', fontWeight: 600 }}>
          Top 10 Presupuestos por Partida - {data.year}
        </h3>
        {topEntities.map((entity, idx) => (
          <div key={entity.entity_id} style={styles.barContainer}>
            <div style={styles.barHeader}>
              <span style={styles.barName}>{entity.display_name}</span>
              <span style={styles.barValue}>{entity.budget_formatted}</span>
            </div>
            <div style={styles.barTrack}>
              <div style={styles.barFill(
                (entity.percentage / maxPct) * 100,
                BAR_COLORS[idx % BAR_COLORS.length]
              )}>
                {entity.percentage > 5 && (
                  <span style={styles.barPct}>{formatPct(entity.percentage, false)}</span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Full Table */}
      <div style={styles.card}>
        <h3 style={{ margin: '0 0 16px', fontSize: '16px', fontWeight: 600 }}>
          Todas las Partidas - {data.year}
        </h3>
        <div style={{ overflowX: 'auto' }}>
          <table style={styles.table}>
            <thead>
              <tr>
                <th style={styles.th}>Partida</th>
                <th style={styles.thRight}>Presupuesto</th>
                <th style={styles.thRight}>% del Total</th>
              </tr>
            </thead>
            <tbody>
              {data.entities.map((entity, idx) => (
                <tr key={entity.entity_id} style={{ backgroundColor: idx % 2 ? '#f8fafc' : 'white' }}>
                  <td style={styles.td}>{entity.display_name}</td>
                  <td style={styles.tdRight}>{entity.budget_formatted}</td>
                  <td style={styles.tdRight}>{formatPct(entity.percentage, false)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}

function Comparador() {
  const [metrics, setMetrics] = useState([])
  const [entities, setEntities] = useState([])
  const [selectedMetric, setSelectedMetric] = useState('')
  const [selectedEntity, setSelectedEntity] = useState('')
  const [entitySearch, setEntitySearch] = useState('')
  const [yearA, setYearA] = useState('2025')
  const [yearB, setYearB] = useState('2026')
  const [result, setResult] = useState(null)
  const [evidence, setEvidence] = useState(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  useEffect(() => {
    fetch(`${API}/metrics`)
      .then(r => r.json())
      .then(data => {
        setMetrics(data.metrics || [])
        // Auto-select first metric
        if (data.metrics?.length > 0) {
          setSelectedMetric(data.metrics[0].metric_id)
        }
      })
      .catch(console.error)
  }, [])

  useEffect(() => {
    if (entitySearch.length < 2) {
      setEntities([])
      return
    }
    const timeout = setTimeout(() => {
      fetch(`${API}/entities?query=${encodeURIComponent(entitySearch)}&limit=10`)
        .then(r => r.json())
        .then(data => setEntities(data.entities || []))
        .catch(console.error)
    }, 300)
    return () => clearTimeout(timeout)
  }, [entitySearch])

  async function runCompare() {
    if (!selectedMetric) {
      setError('Selecciona una métrica')
      return
    }
    setError('')
    setLoading(true)
    setResult(null)

    try {
      let url = `${API}/compare?metric_id=${selectedMetric}&year_a=${yearA}&year_b=${yearB}`
      if (selectedEntity) url += `&entity_id=${selectedEntity}`

      const res = await fetch(url)
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || 'Error en la comparación')
      }
      setResult(await res.json())
    } catch (e) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  async function openEvidence(factId) {
    if (!factId) return
    try {
      const res = await fetch(`${API}/evidence?fact_id=${factId}`)
      if (!res.ok) throw new Error('Evidencia no encontrada')
      setEvidence(await res.json())
    } catch (e) {
      alert('Error: ' + e.message)
    }
  }

  return (
    <div>
      <div style={styles.card}>
        <h3 style={{ margin: '0 0 16px', fontSize: '16px', fontWeight: 600 }}>
          Comparar Presupuestos por Año
        </h3>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '16px' }}>
          <div>
            <label style={{ display: 'block', marginBottom: '6px', fontSize: '13px', fontWeight: 500, color: '#475569' }}>
              Métrica
            </label>
            <select
              value={selectedMetric}
              onChange={e => setSelectedMetric(e.target.value)}
              style={styles.select}
            >
              <option value="">Seleccionar...</option>
              {metrics.map(m => (
                <option key={m.metric_id} value={m.metric_id}>
                  {m.display_name}
                </option>
              ))}
            </select>
          </div>

          <div style={{ position: 'relative' }}>
            <label style={{ display: 'block', marginBottom: '6px', fontSize: '13px', fontWeight: 500, color: '#475569' }}>
              Entidad (opcional)
            </label>
            <input
              type="text"
              value={entitySearch}
              onChange={e => { setEntitySearch(e.target.value); setSelectedEntity('') }}
              placeholder="Buscar ministerio..."
              style={styles.input}
            />
            {entities.length > 0 && !selectedEntity && (
              <div style={{
                position: 'absolute',
                top: '100%',
                left: 0,
                right: 0,
                backgroundColor: 'white',
                border: '1px solid #e2e8f0',
                borderRadius: '8px',
                maxHeight: '200px',
                overflowY: 'auto',
                zIndex: 10,
                boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
              }}>
                {entities.map(e => (
                  <div
                    key={e.entity_id}
                    onClick={() => {
                      setSelectedEntity(e.entity_id)
                      setEntitySearch(e.display_name)
                      setEntities([])
                    }}
                    style={{ padding: '10px 12px', cursor: 'pointer', borderBottom: '1px solid #f1f5f9' }}
                    onMouseOver={ev => ev.target.style.backgroundColor = '#f8fafc'}
                    onMouseOut={ev => ev.target.style.backgroundColor = 'white'}
                  >
                    {e.display_name}
                  </div>
                ))}
              </div>
            )}
          </div>

          <div>
            <label style={{ display: 'block', marginBottom: '6px', fontSize: '13px', fontWeight: 500, color: '#475569' }}>
              Año A
            </label>
            <select value={yearA} onChange={e => setYearA(e.target.value)} style={styles.select}>
              {[2020, 2021, 2022, 2023, 2024, 2025, 2026].map(y => (
                <option key={y} value={y}>{y}</option>
              ))}
            </select>
          </div>

          <div>
            <label style={{ display: 'block', marginBottom: '6px', fontSize: '13px', fontWeight: 500, color: '#475569' }}>
              Año B
            </label>
            <select value={yearB} onChange={e => setYearB(e.target.value)} style={styles.select}>
              {[2020, 2021, 2022, 2023, 2024, 2025, 2026].map(y => (
                <option key={y} value={y}>{y}</option>
              ))}
            </select>
          </div>
        </div>

        <div style={{ marginTop: '20px' }}>
          <button onClick={runCompare} disabled={loading || !selectedMetric} style={styles.button(true, loading || !selectedMetric)}>
            {loading ? 'Comparando...' : 'Comparar'}
          </button>
        </div>

        {error && <div style={styles.error}>{error}</div>}
      </div>

      {result && (
        <div style={styles.card}>
          <h3 style={{ margin: '0 0 16px', fontSize: '16px', fontWeight: 600 }}>
            Resultados: {result.year_a} vs {result.year_b}
          </h3>

          {result.rows.length === 0 ? (
            <p style={{ color: '#64748b' }}>No hay datos para comparar.</p>
          ) : (
            <div style={{ overflowX: 'auto' }}>
              <table style={styles.table}>
                <thead>
                  <tr>
                    <th style={styles.th}>Entidad</th>
                    <th style={styles.thRight}>{result.year_a}</th>
                    <th style={styles.thRight}>{result.year_b}</th>
                    <th style={styles.thRight}>Cambio</th>
                    <th style={styles.thRight}>%</th>
                    <th style={{ ...styles.th, textAlign: 'center' }}>Evidencia</th>
                  </tr>
                </thead>
                <tbody>
                  {result.rows.map((row, idx) => (
                    <tr key={idx} style={{ backgroundColor: idx % 2 ? '#f8fafc' : 'white' }}>
                      <td style={styles.td}>{row.entity_name}</td>
                      <td style={styles.tdRight}>{formatCLP(row.value_a)}</td>
                      <td style={styles.tdRight}>{formatCLP(row.value_b)}</td>
                      <td style={{ ...styles.tdRight, ...(row.delta > 0 ? styles.positive : row.delta < 0 ? styles.negative : {}) }}>
                        {row.delta !== null ? formatCLP(row.delta) : '-'}
                      </td>
                      <td style={{ ...styles.tdRight, ...(row.pct_change > 0 ? styles.positive : row.pct_change < 0 ? styles.negative : {}) }}>
                        {formatPct(row.pct_change)}
                      </td>
                      <td style={{ ...styles.td, textAlign: 'center' }}>
                        {row.fact_id_a && (
                          <button onClick={() => openEvidence(row.fact_id_a)} style={{ ...styles.button(false), padding: '4px 8px', fontSize: '12px', marginRight: '4px' }}>
                            A
                          </button>
                        )}
                        {row.fact_id_b && (
                          <button onClick={() => openEvidence(row.fact_id_b)} style={{ ...styles.button(false), padding: '4px 8px', fontSize: '12px' }}>
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
        <div style={styles.modal} onClick={() => setEvidence(null)}>
          <div style={styles.modalContent} onClick={e => e.stopPropagation()}>
            <h3 style={{ margin: '0 0 16px', fontSize: '18px' }}>Evidencia Verificable</h3>

            <div style={{ fontSize: '14px' }}>
              <div style={{ marginBottom: '12px' }}>
                <strong>Fuente original:</strong><br />
                <a href={evidence.artifact?.url} target="_blank" rel="noopener noreferrer" style={{ ...styles.link, wordBreak: 'break-all' }}>
                  {evidence.artifact?.url}
                </a>
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Fecha de captura:</strong><br />
                {new Date(evidence.artifact?.captured_at).toLocaleString('es-CL')}
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Hash SHA-256:</strong><br />
                <code style={{ fontSize: '11px', backgroundColor: '#f1f5f9', padding: '4px 8px', borderRadius: '4px', wordBreak: 'break-all', display: 'inline-block' }}>
                  {evidence.artifact?.content_hash}
                </code>
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Ubicación:</strong> {evidence.location || 'CSV completo'}
              </div>

              <div style={{ marginBottom: '12px' }}>
                <strong>Método:</strong> {evidence.method}
              </div>
            </div>

            <div style={{ marginTop: '20px', padding: '12px', backgroundColor: '#fef3c7', borderRadius: '8px', fontSize: '13px' }}>
              <strong>Cómo verificar:</strong><br />
              1. Descarga el archivo desde la URL<br />
              2. Calcula el hash SHA-256<br />
              3. Compara con el hash mostrado
            </div>

            <button onClick={() => setEvidence(null)} style={{ ...styles.button(false), marginTop: '16px' }}>
              Cerrar
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

// ============================================================================
// Main App
// ============================================================================

export default function App() {
  const [activeTab, setActiveTab] = useState('dashboard')

  return (
    <div style={styles.container}>
      {/* Header */}
      <header style={styles.header}>
        <h1 style={styles.title}>Estado Transparente</h1>
        <p style={styles.subtitle}>
          Portal ciudadano de datos fiscales verificables - Chile 2020-2026
        </p>
      </header>

      {/* Tabs */}
      <div style={styles.tabs}>
        <button style={styles.tab(activeTab === 'dashboard')} onClick={() => setActiveTab('dashboard')}>
          Dashboard
        </button>
        <button style={styles.tab(activeTab === 'comparar')} onClick={() => setActiveTab('comparar')}>
          Comparar Años
        </button>
      </div>

      {/* Content */}
      {activeTab === 'dashboard' && <Dashboard />}
      {activeTab === 'comparar' && <Comparador />}

      {/* Footer */}
      <footer style={styles.footer}>
        <p>
          <strong>Estado Transparente</strong> — Datos públicos, verificables, sin interpretación editorial.
        </p>
        <p>
          Cada dato tiene cadena de evidencia: URL original, fecha de captura, hash SHA-256.
        </p>
        <p style={{ marginTop: '12px' }}>
          <a href="https://github.com/IANewCool/estado-transparente-portal" target="_blank" rel="noopener noreferrer" style={styles.link}>
            Código fuente en GitHub
          </a>
        </p>
      </footer>
    </div>
  )
}
