# Modelo de Financiamiento — Estado Transparente

> **Versión:** 1.0
> **Fecha:** 2026-01-21
> **Estado:** Diseño aprobado

---

## Principio Rector (NO negociable)

```
El financiamiento NUNCA puede influir en los datos, el pipeline ni los resultados.
Esto se cumple por DISEÑO, no por buena fe.
```

---

## 1. Arquitectura de Separación

### Motor Determinista (zona protegida)

```
┌─────────────────────────────────────────────────────────┐
│                   ZONA PROTEGIDA                        │
│                                                         │
│   collector → parser → facts → provenance → evidence    │
│                                                         │
│   ❌ NO sabe quién financia                             │
│   ❌ NO recibe flags comerciales                        │
│   ❌ NO cambia prioridades por dinero                   │
│   ✔️  Funciona igual con $0 o con $1.000.000            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Capas Externas (zona monetizable)

```
┌─────────────────────────────────────────────────────────┐
│                  ZONA MONETIZABLE                       │
│                                                         │
│   • UI/UX premium                                       │
│   • Infraestructura (rate limits, storage)              │
│   • Servicios adicionales (alertas, exports)            │
│   • Acceso API (volumen, no contenido)                  │
│                                                         │
│   ✔️  Puede recibir dinero                              │
│   ✔️  Puede diferenciar usuarios                        │
│   ❌ NO puede alterar datos                             │
│   ❌ NO puede ocultar información pública               │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Modelo Open Core Cívico

### 🟢 Capa Pública (gratuita, siempre)

| Característica | Incluido | Límite |
|----------------|----------|--------|
| Acceso a datos consolidados | ✅ | Sin límite |
| Comparaciones por año | ✅ | Básicas |
| Evidencia verificable | ✅ | Completa |
| Descarga de datos | ✅ | CSV básico |
| API pública | ✅ | 100 req/hora |
| Código fuente | ✅ | MIT License |
| Auditoría del pipeline | ✅ | Completa |

**Esta capa NO se monetiza. Jamás.**

### 🔵 Capa Pro (servicios, no datos)

| Servicio | Descripción | Monetizable |
|----------|-------------|-------------|
| Comparaciones avanzadas | Multi-año, multi-entidad, tendencias | ✅ |
| Exportaciones masivas | JSON, Excel, formatos personalizados | ✅ |
| API con rate limit alto | 10.000+ req/hora | ✅ |
| Alertas automáticas | Notificaciones de cambios | ✅ |
| Time Machine extendido | Snapshots históricos completos | ✅ |
| Acceso offline | PWA con cache avanzado | ✅ |
| Soporte prioritario | Respuesta en 24h | ✅ |
| Dashboards personalizados | Visualizaciones custom | ✅ |

**Lo que NUNCA se cobra:**

| Prohibido | Razón |
|-----------|-------|
| Ocultar datos | Viola transparencia |
| Priorizar entidades | Viola neutralidad |
| Cambiar resultados | Viola determinismo |
| Acceso anticipado a datos | Viola igualdad |
| "Informes editoriales" | Viola independencia |

---

## 3. Fuentes de Financiamiento PERMITIDAS

### A) Donaciones Ciudadanas

```yaml
tipo: voluntario
contraprestación_editorial: ninguna
contraprestación_técnica:
  - badge de donante (opcional, anónimo disponible)
  - acceso anticipado a features de UX
  - nombre en página de agradecimientos (si acepta)
monto_sugerido:
  - único: $1.000 - $50.000 CLP
  - mensual: $2.000 - $10.000 CLP
plataformas:
  - flow.cl (Chile)
  - mercadopago (Latam)
  - stripe (internacional)
transparencia: monto total publicado mensualmente (sin nombres)
```

### B) Suscripciones Pro (B2B)

**Clientes objetivo:**

| Segmento | Caso de uso |
|----------|-------------|
| Medios de comunicación | Investigación periodística |
| ONGs | Monitoreo de políticas públicas |
| Universidades | Investigación académica |
| Fundaciones | Análisis de transparencia |
| Centros de estudio | Think tanks, policy research |
| Consultoras | Due diligence, compliance |

**Planes propuestos:**

| Plan | Precio CLP/mes | Precio USD/mes | Incluye |
|------|----------------|----------------|---------|
| **Investigador** | $29.990 | $29 | API 5.000 req/h, exports ilimitados |
| **Organización** | $99.990 | $99 | API 20.000 req/h, alertas, dashboards |
| **Enterprise** | $299.990 | $299 | API ilimitada, soporte dedicado, SLA |

**Descuentos:**
- Anual: 20% descuento
- ONGs acreditadas: 50% descuento
- Universidades públicas: 70% descuento
- Medios independientes: caso a caso

### C) Licenciamiento del Motor

**Producto:** El stack técnico (pipeline + evidencia + comparador)

**NO incluye:** Datos de Chile (esos son públicos, no vendibles)

| Licencia | Precio USD | Incluye |
|----------|------------|---------|
| **Comunidad** | $0 | Código fuente MIT, sin soporte |
| **Institucional** | $5.000/año | Soporte, actualizaciones, capacitación |
| **Gobierno** | $15.000/año | Implementación asistida, SLA, personalización |

**Clientes potenciales:**
- Municipalidades chilenas
- Universidades (cursos de data pública)
- Organismos internacionales (BID, CEPAL, etc.)
- Otros países (replicar el modelo)

### D) Educación y Formación

| Producto | Precio CLP | Descripción |
|----------|------------|-------------|
| Taller "Datos Públicos" | $50.000/persona | 4 horas, presencial/online |
| Curso "Auditoría Ciudadana" | $150.000 | 12 horas, certificado |
| Bootcamp "Transparencia Tech" | $500.000 | 40 horas, proyecto real |
| Licencia académica | Gratuito | Uso en cursos universitarios |

---

## 4. Fuentes de Financiamiento PROHIBIDAS

| Fuente | Razón de prohibición |
|--------|---------------------|
| Financiamiento estatal directo | Conflicto de interés con objeto de auditoría |
| Publicidad política | Sesgo editorial implícito |
| Patrocinios con derecho a veto | Compromete independencia |
| Pago por ranking/exposición | Corrupción del orden de resultados |
| Monetización ideológica | Polarización, pérdida de neutralidad |
| Venta de "informes editoriales" | Mezcla opinión con datos |
| Donaciones de entidades fiscalizadas | Conflicto de interés directo |
| Gobiernos extranjeros | Riesgo de influencia geopolítica |

**Protocolo de rechazo:**

1. Toda oferta de financiamiento se evalúa contra esta lista
2. Si viola algún principio, se rechaza por escrito
3. El rechazo se documenta internamente (sin publicar detalles sensibles)
4. Se publica estadística anual: "X ofertas rechazadas por conflicto de interés"

---

## 5. Estructura de Costos Estimada

### Costos Fijos Mensuales (MVP)

| Concepto | Costo CLP | Costo USD |
|----------|-----------|-----------|
| Servidores (API + DB) | $50.000 | $50 |
| Storage (MinIO/S3) | $20.000 | $20 |
| CDN | $10.000 | $10 |
| Dominio + SSL | $5.000 | $5 |
| **Total MVP** | **$85.000** | **$85** |

### Costos Variables

| Concepto | Costo unitario |
|----------|----------------|
| Storage por GB adicional | $100 CLP |
| Requests API sobre cuota | $0.001 CLP/req |
| Procesamiento de PDF | $10 CLP/página |

### Costos Opcionales (escala)

| Concepto | Costo mensual |
|----------|---------------|
| Desarrollador part-time | $500.000 - $1.000.000 CLP |
| Diseñador UX (freelance) | $300.000 - $500.000 CLP |
| Auditoría de seguridad anual | $2.000.000 CLP |

---

## 6. Proyección de Sostenibilidad

### Escenario Mínimo Viable

```
Ingresos necesarios: $85.000 CLP/mes ($85 USD)

Fuentes:
- 42 donantes de $2.000/mes = $84.000
- O 3 suscriptores Investigador = $89.970

Estado: Sostenible sin trabajo remunerado
```

### Escenario Crecimiento Moderado

```
Ingresos objetivo: $500.000 CLP/mes ($500 USD)

Fuentes:
- 100 donantes promedio $2.500/mes = $250.000
- 5 suscriptores Investigador = $149.950
- 1 suscriptor Organización = $99.990

Estado: Sostenible con mejoras continuas
```

### Escenario Profesionalización

```
Ingresos objetivo: $2.000.000 CLP/mes ($2.000 USD)

Fuentes:
- Donaciones ciudadanas = $400.000
- 20 suscriptores Pro = $800.000
- 2 licencias institucionales = $500.000
- Talleres/cursos = $300.000

Estado: Equipo part-time dedicado
```

---

## 7. Gobernanza del Financiamiento

### Principios

1. **Transparencia total**: Ingresos y gastos publicados mensualmente
2. **Sin dependencia única**: Ninguna fuente > 30% del total
3. **Reserva de operación**: Mantener 6 meses de costos en reserva
4. **Decisiones públicas**: Cambios en modelo se anuncian con 30 días de anticipación

### Comité de Ética (futuro)

Cuando el proyecto escale:
- 3-5 personas independientes
- Revisan conflictos de interés
- Aprueban/rechazan financiamientos dudosos
- Publican informe anual

---

## 8. Métricas de Salud Financiera

| Métrica | Objetivo | Alerta |
|---------|----------|--------|
| Diversificación | Ninguna fuente > 30% | > 40% |
| Reserva | 6 meses de operación | < 3 meses |
| Crecimiento donantes | > 5% mensual | Negativo 3 meses |
| Churn Pro | < 5% mensual | > 10% |
| Ratio gratuito/pagado | > 95% usuarios gratis | < 90% |

---

## 9. Compromiso Público

```
Estado Transparente se compromete a:

1. Nunca cobrar por acceso a datos públicos
2. Nunca alterar resultados por dinero
3. Nunca aceptar financiamiento que comprometa independencia
4. Publicar finanzas mensualmente
5. Rechazar y documentar ofertas con conflicto de interés
6. Mantener el código fuente abierto
7. Permitir auditoría ciudadana del pipeline

Este compromiso es perpetuo e irrevocable.
```

---

## 10. Preguntas Frecuentes

**¿Por qué no aceptan dinero del gobierno?**
> Porque fiscalizamos al gobierno. Aceptar su dinero crea conflicto de interés real o percibido.

**¿Pueden las empresas fiscalizadas donar?**
> No. Si una entidad aparece en nuestros datos, no puede financiarnos.

**¿Qué pasa si no alcanzan los ingresos?**
> El portal sigue funcionando. Reducimos features premium, no datos públicos.

**¿Quién decide qué financiamiento aceptar?**
> Hoy: el equipo fundador. Mañana: comité de ética independiente.

**¿Puedo auditar sus finanzas?**
> Sí. Publicamos ingresos por categoría y gastos detallados mensualmente.

---

*"La confianza no se compra, se construye con transparencia."*

---

**Documento vinculante:** [INDEPENDENCIA_ALGORITMICA.md](INDEPENDENCIA_ALGORITMICA.md)
