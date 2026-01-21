# Independencia Algorítmica — Estado Transparente

> **Versión:** 1.0
> **Fecha:** 2026-01-21
> **Estado:** Política vigente

---

## Declaración de Independencia

```
El motor de Estado Transparente es:

  DETERMINISTA  →  Misma entrada = misma salida, siempre
  AUDITABLE     →  Cualquier persona puede verificar
  REPRODUCIBLE  →  Los cálculos se pueden replicar
  NEUTRAL       →  No favorece ni perjudica a nadie
  INDEPENDIENTE →  No responde a intereses externos

Esta independencia se garantiza por DISEÑO TÉCNICO,
no por promesas ni buenas intenciones.
```

---

## 1. Principios Técnicos

### 1.1 Determinismo

**Definición:** Dado el mismo artifact (archivo fuente) y la misma versión del parser, el resultado es idéntico.

```
artifact_v1.csv + parser_v2.3.1 = facts_hash_abc123
artifact_v1.csv + parser_v2.3.1 = facts_hash_abc123  ← siempre igual
```

**Implementación:**
- Sin timestamps en cálculos (solo en metadata)
- Sin random/UUID en lógica de parsing
- Sin dependencias externas variables
- Versionado semántico estricto del parser

**Verificación:**
```bash
# Cualquiera puede verificar
cargo run --bin parser -- --artifact-id X --dry-run
# Debe producir el mismo hash de salida
```

### 1.2 Auditabilidad

**Definición:** Todo dato mostrado tiene una cadena de evidencia verificable.

```
┌─────────────────────────────────────────────────────────────┐
│                    CADENA DE EVIDENCIA                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  DATO MOSTRADO                                              │
│       ↓                                                     │
│  fact_id: uuid-123                                          │
│       ↓                                                     │
│  provenance: artifact_id=uuid-456, location="csv:line=42"   │
│       ↓                                                     │
│  artifact: url="https://...", hash="sha256:abc...",         │
│            captured_at="2026-01-15T10:30:00Z"               │
│       ↓                                                     │
│  RAW FILE: descargable, verificable                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Garantías:**
- Cada `fact` tiene `provenance` obligatorio
- Cada `artifact` tiene hash SHA-256
- Los archivos raw son descargables
- La ubicación exacta del dato es rastreable

### 1.3 Reproducibilidad

**Definición:** Cualquier persona puede replicar todo el proceso.

**Componentes públicos:**
| Componente | Disponibilidad |
|------------|----------------|
| Código fuente | GitHub (MIT) |
| Esquema de BD | `shared/sql/` |
| Schemas JSON | `shared/schema/` |
| Docker Compose | `infra/` |
| Documentación | `docs/` |

**Proceso de verificación:**
```bash
# 1. Clonar repositorio
git clone https://github.com/estado-transparente/portal

# 2. Levantar infraestructura
cd infra && docker compose up -d

# 3. Descargar artifact específico
curl -O "https://raw-store.estadotransparente.cl/artifact_uuid"

# 4. Verificar hash
sha256sum artifact_uuid  # debe coincidir con BD

# 5. Ejecutar parser
cargo run --bin parser -- --artifact-id uuid --verify

# 6. Comparar facts generados con los publicados
```

---

## 2. Separación Financiamiento ↔ Motor

### 2.1 Arquitectura de Aislamiento

```
┌────────────────────────────────────────────────────────────────┐
│                         SISTEMA                                │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              MOTOR (zona protegida)                      │  │
│  │                                                          │  │
│  │   ┌─────────┐    ┌────────┐    ┌───────┐    ┌─────────┐ │  │
│  │   │collector│ → │ parser │ → │ facts │ → │provenance│ │  │
│  │   └─────────┘    └────────┘    └───────┘    └─────────┘ │  │
│  │                                                          │  │
│  │   SIN ACCESO A:                                          │  │
│  │   • Información de financiamiento                        │  │
│  │   • Identidad de donantes                                │  │
│  │   • Planes de suscripción                                │  │
│  │   • Flags comerciales                                    │  │
│  │                                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ▲                                    │
│                           │ FIREWALL LÓGICO                    │
│                           │ (sin comunicación)                 │
│                           ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              SERVICIOS (zona monetizable)                │  │
│  │                                                          │  │
│  │   ┌────────┐    ┌─────────┐    ┌──────────┐             │  │
│  │   │  auth  │    │ billing │    │ features │             │  │
│  │   └────────┘    └─────────┘    └──────────┘             │  │
│  │                                                          │  │
│  │   PUEDE ACCEDER A:                                       │  │
│  │   • Rate limits                                          │  │
│  │   • Features UI                                          │  │
│  │   • Exports                                              │  │
│  │                                                          │  │
│  │   NO PUEDE MODIFICAR:                                    │  │
│  │   • Datos                                                │  │
│  │   • Orden de resultados                                  │  │
│  │   • Evidencia                                            │  │
│  │                                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### 2.2 Reglas de Código

**El motor NO puede importar:**
```rust
// ❌ PROHIBIDO en services/collector, services/parser
use crate::billing::*;
use crate::subscriptions::*;
use crate::user_tier::*;
use crate::payments::*;
```

**El motor NO tiene tablas de:**
```sql
-- ❌ NO EXISTEN en el schema del motor
CREATE TABLE donations ...
CREATE TABLE subscriptions ...
CREATE TABLE user_payments ...
CREATE TABLE billing ...
```

**Validación CI/CD:**
```yaml
# .github/workflows/independence.yml
- name: Check motor isolation
  run: |
    # Verificar que el motor no importa módulos de billing
    ! grep -r "billing\|subscription\|payment\|donation" services/collector/
    ! grep -r "billing\|subscription\|payment\|donation" services/parser/
```

### 2.3 Qué SÍ puede hacer el financiamiento

| Acción | Permitido | Ejemplo |
|--------|-----------|---------|
| Limitar requests/hora | ✅ | Free: 100/h, Pro: 10.000/h |
| Habilitar exports | ✅ | Free: CSV, Pro: Excel+JSON |
| Agregar alertas | ✅ | Solo Pro tiene notificaciones |
| Mejorar UX | ✅ | Dashboards solo para Pro |
| Dar soporte | ✅ | Pro tiene respuesta en 24h |

### 2.4 Qué NO puede hacer el financiamiento

| Acción | Permitido | Razón |
|--------|-----------|-------|
| Ocultar datos | ❌ | Viola transparencia |
| Cambiar orden de resultados | ❌ | Viola neutralidad |
| Priorizar entidades | ❌ | Viola igualdad |
| Modificar cálculos | ❌ | Viola determinismo |
| Acceso anticipado a datos | ❌ | Viola igualdad |
| Eliminar evidencia | ❌ | Viola auditabilidad |

---

## 3. Garantías para el Ciudadano

### 3.1 Compromiso de Neutralidad

```
Estado Transparente garantiza que:

1. Los mismos datos se muestran a todos los usuarios
2. El orden de resultados no depende de quién pregunta
3. Ningún pago puede alterar un dato
4. Ninguna entidad puede "desaparecer" de los resultados
5. La evidencia está disponible para todos, siempre
```

### 3.2 Cómo Verificar

**Cualquier ciudadano puede:**

| Verificación | Cómo hacerlo |
|--------------|--------------|
| Auditar el código | `git clone` + revisar |
| Verificar un dato | Click "Ver evidencia" → descargar raw |
| Replicar un cálculo | Ejecutar parser localmente |
| Comparar resultados | Comparar API pública vs local |
| Denunciar anomalías | Abrir issue público en GitHub |

### 3.3 Protocolo de Reporte

Si un ciudadano detecta inconsistencias:

1. **Reporte público**: Issue en GitHub con evidencia
2. **Investigación**: Equipo revisa en 72 horas
3. **Respuesta pública**: Explicación o corrección
4. **Post-mortem**: Si hubo error, se documenta cómo se corrigió

---

## 4. Garantías para Periodistas

### 4.1 Acceso Igualitario

```
Todo periodista, de cualquier medio, tiene:

• Mismo acceso a datos que cualquier ciudadano
• Misma API (dentro de rate limits)
• Misma evidencia verificable
• Mismo código fuente para auditar
```

### 4.2 Cómo Citar

```
Formato sugerido:

"Según datos de Estado Transparente, que consolida información
de [FUENTE ORIGINAL], verificable en [URL_EVIDENCIA]..."

Importante: Citar la fuente original, no solo el portal.
```

### 4.3 Verificación Periodística

| Pregunta | Respuesta disponible |
|----------|---------------------|
| ¿De dónde viene este dato? | Endpoint `/evidence` |
| ¿Cuándo se capturó? | Campo `captured_at` |
| ¿Puedo ver el original? | Link de descarga raw |
| ¿El cálculo es correcto? | Código fuente del parser |
| ¿Quién financia esto? | Página pública de finanzas |

---

## 5. Garantías para Financiadores

### 5.1 Qué Obtiene un Financiador

```
✅ Servicios técnicos premium (API, exports, alertas)
✅ Reconocimiento público (si lo desea)
✅ Satisfacción de apoyar transparencia cívica
✅ Acceso a capacitaciones
```

### 5.2 Qué NO Obtiene un Financiador

```
❌ Influencia sobre datos
❌ Modificación de resultados
❌ Priorización de entidades
❌ Ocultamiento de información
❌ Acceso anticipado a datos
❌ Veto sobre publicaciones
```

### 5.3 Contrato Implícito

```
Al financiar Estado Transparente, el financiador acepta que:

1. No tendrá influencia sobre el motor
2. No podrá solicitar cambios en datos
3. Su financiamiento será público (monto, no identidad si prefiere)
4. Puede ser rechazado si hay conflicto de interés
5. La relación puede terminar si viola estos términos
```

---

## 6. Auditoría Continua

### 6.1 Auditoría Técnica

| Aspecto | Frecuencia | Método |
|---------|------------|--------|
| Hash de artifacts | Cada ingesta | Automático |
| Determinismo del parser | Cada release | Tests CI |
| Integridad de facts | Diario | Checksum DB |
| Aislamiento motor/billing | Cada PR | Grep automatizado |

### 6.2 Auditoría Financiera

| Aspecto | Frecuencia | Publicación |
|---------|------------|-------------|
| Ingresos por categoría | Mensual | Pública |
| Gastos detallados | Mensual | Pública |
| Financiamientos rechazados | Anual | Estadística |
| Conflictos de interés | Cuando ocurran | Inmediata |

### 6.3 Auditoría Ciudadana

Cualquier persona puede:
- Solicitar información adicional
- Proponer mejoras al proceso
- Reportar anomalías
- Verificar cualquier dato

---

## 7. Escenarios y Respuestas

### Escenario 1: Gran donante pide cambio

```
Situación: Donante de $10.000.000 CLP pide "ajustar" un dato.

Respuesta:
1. Se rechaza la solicitud por escrito
2. Se devuelve la donación si insiste
3. Se documenta internamente
4. Se reporta en estadística anual
```

### Escenario 2: Entidad fiscalizada ofrece financiar

```
Situación: Ministerio X ofrece $50.000.000 CLP anuales.

Respuesta:
1. Se rechaza automáticamente (regla: no financiamiento estatal)
2. Se agradece pero se explica el conflicto de interés
3. Se sugiere apoyar el código abierto de otra forma
```

### Escenario 3: Error en datos detectado

```
Situación: Ciudadano reporta que un monto está mal.

Respuesta:
1. Se verifica contra el artifact original
2. Si es error del parser: se corrige y se publica post-mortem
3. Si es error de la fuente: se documenta y se notifica
4. Se agradece públicamente al ciudadano
```

### Escenario 4: Presión política

```
Situación: Partido político exige "bajar" información.

Respuesta:
1. Se rechaza categóricamente
2. Se documenta el intento de presión
3. Se considera publicar el intento (según gravedad)
4. Se refuerza la independencia en comunicación pública
```

---

## 8. Compromiso Perpetuo

```
┌────────────────────────────────────────────────────────────────┐
│                                                                │
│   Estado Transparente se compromete a mantener la              │
│   independencia algorítmica de forma PERPETUA e IRREVOCABLE.   │
│                                                                │
│   Este compromiso aplica a:                                    │
│   • Todos los que trabajan en el proyecto                      │
│   • Todos los que lo financian                                 │
│   • Todos los que lo heredan                                   │
│                                                                │
│   Si el proyecto cambia de manos, este documento               │
│   sigue siendo vinculante.                                     │
│                                                                │
│   Si alguna vez se viola este compromiso:                      │
│   • El código sigue siendo MIT (cualquiera puede forkearlo)    │
│   • La comunidad puede continuar el proyecto                   │
│   • El nombre "Estado Transparente" debe abandonarse           │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

---

## 9. Firmas Técnicas

Este documento se verifica con:

```
Documento: INDEPENDENCIA_ALGORITMICA.md
Versión: 1.0
SHA-256: [se calcula al publicar]
Fecha: 2026-01-21
Ubicación: docs/INDEPENDENCIA_ALGORITMICA.md
```

Cada versión futura debe:
1. Incrementar versión
2. Documentar cambios
3. Mantener historial en git
4. No debilitar garantías (solo fortalecer)

---

*"La independencia no se declara, se demuestra."*

---

**Documentos relacionados:**
- [FINANCIAMIENTO.md](FINANCIAMIENTO.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [DATA_MODEL.md](DATA_MODEL.md)
