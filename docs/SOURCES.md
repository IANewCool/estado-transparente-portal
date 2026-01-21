# Fuentes (catálogo)

Este archivo define fuentes de datos públicos y sus reglas de ingesta/parsing.

## Fuentes Implementadas

### 1. Demo: Presupuesto por Ministerio

| Campo | Valor |
|-------|-------|
| **source_id** | `demo-presupuesto` |
| **Tipo** | CSV |
| **URL** | Local: `data/demo_presupuesto.csv` |
| **Frecuencia** | Manual (demo) |
| **Parser** | `csv_parser_v1` |
| **Métrica** | `monto` → Monto asignado |

**Columnas:**
- `entidad`: Nombre del organismo
- `categoria`: Personal, Operaciones, Inversión
- `anio`: Año del presupuesto
- `monto`: Monto en CLP

**Comando de ingesta:**
```bash
# Desde la raíz del proyecto
cargo run --bin collector -- \
  --source-id demo-presupuesto \
  --url "file://$(pwd)/data/demo_presupuesto.csv"
```

---

## Fuentes Planificadas

### 2. DIPRES - Presupuesto de la Nación

| Campo | Valor |
|-------|-------|
| **source_id** | `dipres-presupuesto` |
| **Tipo** | PDF / Excel |
| **URL** | https://www.dipres.gob.cl/... |
| **Frecuencia** | Anual |
| **Parser** | `dipres_parser_v1` (pendiente) |

### 3. ChileCompra - Órdenes de Compra

| Campo | Valor |
|-------|-------|
| **source_id** | `chilecompra-ordenes` |
| **Tipo** | CSV / API |
| **URL** | https://www.mercadopublico.cl/... |
| **Frecuencia** | Diaria |
| **Parser** | `chilecompra_parser_v1` (pendiente) |

### 4. Contraloría - Remuneraciones

| Campo | Valor |
|-------|-------|
| **source_id** | `contraloria-remuneraciones` |
| **Tipo** | CSV |
| **URL** | https://www.contraloria.cl/... |
| **Frecuencia** | Mensual |
| **Parser** | `contraloria_parser_v1` (pendiente) |

---

## Proceso de Adición de Fuentes

1. **Identificar fuente pública**
   - Verificar que los datos son públicos
   - Documentar URL y formato
   - Verificar frecuencia de actualización

2. **Crear parser determinista**
   - Implementar en `services/parser/src/parsers/`
   - Asegurar que mismo input = mismo output
   - Agregar tests

3. **Documentar aquí**
   - source_id único
   - Todas las columnas/campos
   - Comando de ingesta

4. **Probar pipeline completo**
   ```bash
   # Collector
   cargo run --bin collector -- --source-id X --url "..."

   # Parser
   cargo run --bin parser -- --artifact-id <UUID>

   # Verificar en API
   curl http://localhost:8080/facts
   ```

---

## Notas Legales

- Solo se ingestan datos de **fuentes públicas oficiales**
- Se respeta el `robots.txt` de cada sitio
- Rate limit mínimo de 1 segundo entre requests
- Los artifacts se almacenan con hash verificable
- La URL original siempre se preserva para auditoría

---

## Contacto para Nuevas Fuentes

Si conoces una fuente de datos públicos que debería incluirse:
1. Abre un issue en GitHub
2. Incluye: URL, formato, frecuencia de actualización
3. Verificaremos que sea datos públicos
