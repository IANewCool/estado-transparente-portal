# Cómo nos financiamos

> Texto base para la página pública del portal

---

## Versión Completa (para página dedicada)

### Independencia total

Estado Transparente es un portal **100% independiente**. No recibimos dinero del gobierno, partidos políticos ni entidades que aparecen en nuestros datos.

¿Por qué? Porque no podemos fiscalizar a quien nos paga.

---

### Nuestras fuentes de ingreso

#### Donaciones ciudadanas
Personas como tú que creen en la transparencia. Sin montos mínimos, sin obligación, sin contraprestación editorial.

#### Suscripciones Pro
Organizaciones (medios, ONGs, universidades) que necesitan acceso avanzado: más consultas API, exportaciones masivas, alertas automáticas. **Pagan por servicios, no por datos.**

#### Licenciamiento del motor
Instituciones que quieren replicar este sistema en otros contextos. Vendemos el cómo, no el qué.

#### Educación
Talleres y cursos sobre datos públicos y transparencia.

---

### Lo que NUNCA aceptamos

| Rechazamos | Razón |
|------------|-------|
| Dinero del Estado | Conflicto de interés |
| Publicidad política | Sesgo editorial |
| Patrocinios con condiciones | Compromete independencia |
| Pago por posicionamiento | Corrompe neutralidad |
| Donaciones de entidades fiscalizadas | Conflicto directo |

---

### El dinero NO toca los datos

Nuestra arquitectura separa físicamente el financiamiento del motor de datos:

```
MOTOR DE DATOS          SERVICIOS
(intocable)             (monetizable)
     │                       │
     │    ══ FIREWALL ══     │
     │                       │
  facts                  rate limits
  evidencia              exports
  cálculos               alertas
                         UI premium
```

Un donante de $10 millones ve **exactamente los mismos datos** que alguien que nunca ha pagado un peso.

---

### Transparencia financiera

Publicamos mensualmente:
- Ingresos totales por categoría
- Gastos detallados
- Ofertas rechazadas (sin nombres, con razón)

[Ver reporte financiero actual →](#)

---

### ¿Y si no alcanzan los ingresos?

El portal sigue funcionando. Reducimos features premium antes que acceso a datos.

**Lo último que se apaga son los datos públicos.**

---

### Apoya la transparencia

**Dona una vez**
Cualquier monto ayuda. Sin compromisos.

[Donar →](#)

**Hazte Pro**
Si eres medio, ONG o investigador, accede a herramientas avanzadas.

[Ver planes →](#)

**Audita nuestro código**
Todo es open source. Verifica que cumplimos lo que decimos.

[GitHub →](#)

---

### Preguntas frecuentes

**¿Por qué no aceptan plata del gobierno?**
Porque fiscalizamos al gobierno. Aceptar su dinero —aunque sea legal— genera conflicto de interés real o percibido.

**¿Puedo donar anónimamente?**
Sí. No publicamos nombres sin consentimiento.

**¿Qué pasa si una empresa fiscalizada quiere donar?**
Se rechaza. Si apareces en nuestros datos, no puedes financiarnos.

**¿Cómo sé que el dinero no afecta los datos?**
El código es abierto. Puedes verificar que el motor no tiene acceso a información de pagos. Además, publicamos todas nuestras finanzas.

---

*"La confianza no se compra, se construye con transparencia."*

---

## Versión Corta (para footer o sidebar)

```
FINANCIAMIENTO

Este portal es 100% independiente.
No recibimos dinero del Estado ni de entidades fiscalizadas.

Nos financiamos con:
• Donaciones ciudadanas
• Suscripciones Pro (servicios, no datos)
• Licenciamiento del motor
• Educación

El dinero no toca los datos.
Todo es auditable.

[Más información →]
```

---

## Versión Tweet/Redes (280 caracteres)

```
Estado Transparente es 100% independiente.

No recibimos plata del gobierno ni de entidades fiscalizadas.

El dinero no toca los datos. Todo es auditable.

Apoya: [link]
Verifica: [github]
```

---

## Versión para Email de Agradecimiento a Donantes

```
Asunto: Gracias por apoyar la transparencia

Hola,

Gracias por tu donación a Estado Transparente.

Tu aporte nos ayuda a mantener un portal independiente que no responde
a intereses políticos ni económicos.

Qué garantizamos:
✓ Tu donación no te da acceso especial a datos
✓ Los datos siguen siendo iguales para todos
✓ Publicamos nuestras finanzas mensualmente
✓ El código es abierto y auditable

Si en algún momento quieres verificar cómo usamos los fondos,
visita: [link a transparencia financiera]

Gracias por creer en la transparencia.

Equipo Estado Transparente
```

---

## Versión para Propuesta B2B (Suscripción Pro)

```
PROPUESTA: SUSCRIPCIÓN PRO

Para: [Organización]
Fecha: [Fecha]

---

¿QUÉ ES ESTADO TRANSPARENTE?

Portal ciudadano independiente que consolida información pública
del Estado de Chile con evidencia verificable.

---

¿QUÉ INCLUYE LA SUSCRIPCIÓN PRO?

Plan Investigador ($29.990/mes):
• API: 5.000 requests/hora (vs 100 gratuito)
• Exports ilimitados (CSV, JSON, Excel)
• Comparaciones multi-año
• Soporte por email

Plan Organización ($99.990/mes):
• API: 20.000 requests/hora
• Alertas automáticas de cambios
• Dashboards personalizados
• Soporte prioritario (24h)

Plan Enterprise ($299.990/mes):
• API ilimitada
• Soporte dedicado
• SLA 99.9%
• Integración personalizada

Descuentos:
• ONGs acreditadas: 50%
• Universidades públicas: 70%
• Pago anual: 20%

---

¿QUÉ NO INCLUYE?

❌ Acceso a datos que otros no tienen
❌ Modificación de resultados
❌ Priorización de entidades
❌ Influencia editorial

Los datos son iguales para todos.
Pagamos por servicios, no por verdad.

---

NUESTRA GARANTÍA

1. Independencia algorítmica documentada
2. Código abierto y auditable
3. Finanzas públicas mensuales
4. Sin conflictos de interés

---

SIGUIENTE PASO

Agenda una demo: [email]
O empieza directamente: [link]

---

Estado Transparente
Portal Ciudadano Independiente
```

---

## Badges para README/Sitio

```markdown
<!-- Para usar en el sitio o README -->

![Independiente](https://img.shields.io/badge/Financiamiento-Independiente-green)
![No Estatal](https://img.shields.io/badge/Estado-No%20Financia-red)
![Open Source](https://img.shields.io/badge/C%C3%B3digo-MIT%20License-blue)
![Auditable](https://img.shields.io/badge/Finanzas-P%C3%BAblicas-yellow)
```

---

## Componente React (para el sitio)

```jsx
// components/FinanciamientoCard.jsx
export function FinanciamientoCard() {
  return (
    <div className="border rounded-lg p-4 bg-slate-50">
      <h3 className="font-bold text-lg mb-2">100% Independiente</h3>
      <p className="text-sm text-slate-600 mb-3">
        No recibimos dinero del Estado ni de entidades fiscalizadas.
      </p>
      <ul className="text-sm space-y-1 mb-3">
        <li>✓ Donaciones ciudadanas</li>
        <li>✓ Suscripciones Pro (servicios, no datos)</li>
        <li>✓ Código abierto y auditable</li>
      </ul>
      <div className="flex gap-2">
        <a href="/financiamiento" className="text-blue-600 text-sm">
          Más información →
        </a>
        <a href="/donar" className="text-green-600 text-sm">
          Apoyar →
        </a>
      </div>
    </div>
  )
}
```
