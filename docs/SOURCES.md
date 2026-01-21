# Fuentes de Datos — Estado Transparente

Este archivo define las fuentes de datos oficiales y sus reglas de ingesta/parsing.

---

## Fuente MVP: Ley de Presupuestos 2026 (DIPRES)

### Identificación

| Campo | Valor |
|-------|-------|
| **source_id** | `dipres-ley-presupuestos-2026` |
| **Tipo** | CSV |
| **URL** | https://www.dipres.gob.cl/597/articles-397499_doc_csv.csv |
| **Frecuencia** | Anual (publicación de Ley de Presupuestos) |
| **Parser** | `dipres_ley_csv_v1` |

### Descripción

**Resumen Presupuesto de Partida [Pesos]** — Ley de Presupuestos del Sector Público, Año Fiscal 2026.

Archivo CSV oficial publicado por la Dirección de Presupuestos (DIPRES) del Ministerio de Hacienda de Chile.

### Estructura del CSV

| Columna | Tipo | Descripción |
|---------|------|-------------|
| Partida | TEXT | Código de partida presupuestaria (ministerio/servicio) |
| Capitulo | TEXT | Código de capítulo |
| Programa | TEXT | Código de programa |
| Subtitulo | TEXT | Clasificación económica del gasto/ingreso |
| Ítem | TEXT | Detalle del ítem presupuestario |
| Asignacion | TEXT | Código de asignación específica |
| Denominacion | TEXT | Nombre descriptivo del concepto |
| Monto Pesos | NUMBER | Monto en pesos chilenos (CLP), miles de pesos |
| Monto Dolar | NUMBER | Monto en dólares (USD) |

**Delimitador:** `;` (punto y coma)
**Encoding:** UTF-8 con BOM
**Tamaño:** ~792 KB (~4,500 líneas)

### Justificación de Estabilidad

1. **URL permanente**: El patrón `articles-XXXXXX_doc_csv.csv` es usado consistentemente por DIPRES desde 2009 para todos los documentos de Ley de Presupuestos.

2. **Publicación oficial**: Forma parte de la [Ley de Presupuestos](https://www.dipres.gob.cl/597/w3-multipropertyvalues-15145-37782.html), documento legal de publicación obligatoria por el Estado de Chile.

3. **Sin autenticación**: Acceso público directo, sin login, cookies ni API keys.

4. **Estructura estable**: El formato de 9 columnas se mantiene consistente entre años fiscales (verificado 2020-2026).

5. **Metadatos verificables**:
   ```
   HTTP/1.1 200 OK
   Content-Type: text/csv
   Last-Modified: Tue, 16 Dec 2025 15:35:10 GMT
   Content-Length: 792627
   ```

### Mapeo a Modelo de Datos

| Campo CSV | Campo Fact | Transformación |
|-----------|------------|----------------|
| Partida | entity_key | Código numérico (ej: "01") |
| Denominacion (primera ocurrencia por Partida) | entity_name | Texto descriptivo |
| "2026" | period_start | 2026-01-01 |
| "2026" | period_end | 2026-12-31 |
| SUM(Monto Pesos) por Partida | value_num | Agregación |
| Subtitulo | dims.subtitulo | Clasificación económica |
| — | metric_key | `presupuesto_ley` |
| — | unit | `CLP` |

### Verificación Manual

```bash
# 1. Verificar disponibilidad
curl -sI "https://www.dipres.gob.cl/597/articles-397499_doc_csv.csv" | grep -E "HTTP|Content-Type|Last-Modified"

# 2. Descargar y contar líneas
curl -s "https://www.dipres.gob.cl/597/articles-397499_doc_csv.csv" | wc -l

# 3. Ver estructura (primeras 5 líneas)
curl -s "https://www.dipres.gob.cl/597/articles-397499_doc_csv.csv" | head -5

# 4. Verificar hash
curl -s "https://www.dipres.gob.cl/597/articles-397499_doc_csv.csv" | sha256sum
```

### Referencias Oficiales

- [Portal de Datos Abiertos DIPRES](https://www.dipres.gob.cl/598/w3-propertyvalue-24024.html)
- [Ley de Presupuestos 2026](https://www.dipres.gob.cl/597/w3-multipropertyvalues-15145-37782.html)
- [Estadísticas DIPRES](https://www.dipres.gob.cl/598/w3-propertyname-706.html)

---

## Fuentes Descartadas para MVP

Las siguientes fuentes se evaluaron pero **no cumplen los criterios** de estabilidad para el MVP:

| Fuente | Razón de exclusión |
|--------|-------------------|
| ChileCompra | Requiere API con autenticación |
| Contraloría | Solo PDF, sin CSV público estable |
| DIPRES Ejecución Mensual | Formato XLS con headers no estándar (títulos en lugar de columnas) |
| datos.gob.cl | API REST, no descarga directa de CSV |
| DIPRES artículos XLS genéricos | Estructura variable entre archivos |

---

## Criterios de Inclusión de Fuentes

Para agregar una nueva fuente, debe cumplir **todos** estos criterios:

1. **Formato**: CSV o formato tabular con estructura predecible
2. **Acceso**: URL pública directa, sin autenticación
3. **Estabilidad**: URL verificada funcionando por al menos 1 año
4. **Estructura**: Columnas documentadas y consistentes
5. **Mapeo**: Correspondencia clara con el modelo de datos (entity, metric, period, value)
6. **Determinismo**: Mismo archivo = mismo output de parser

Si existe ambigüedad en cualquiera de estos criterios, la fuente **no se implementa** hasta resolverla.

---

## Notas Legales

- Solo se ingestan datos de **fuentes públicas oficiales del Estado de Chile**
- Se respeta el `robots.txt` de cada sitio
- Los artifacts se almacenan con hash SHA-256 verificable
- La URL original siempre se preserva para auditoría y trazabilidad
- Este proyecto no tiene afiliación oficial con DIPRES ni el Gobierno de Chile

---

*Última actualización: 2026-01-21*
