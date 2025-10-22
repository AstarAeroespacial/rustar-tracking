# Tracking Satelital con Corrección de Doppler

Sistema de tracking satelital en Rust con cálculo preciso de corrección Doppler para enlaces de radio.

## 📡 API Principal

### Funciones de Doppler

```rust
use tracking::{doppler_downlink, doppler_uplink};

// Downlink: Calcular frecuencia de recepción en estación terrena
let freq_rx = doppler_downlink(freq_tx_sat, range_rate);

// Uplink: Calcular frecuencia de transmisión desde estación terrena
let freq_tx = doppler_uplink(freq_rx_sat, range_rate);
```

**Parámetros:**

-   `freq_tx_sat` / `freq_rx_sat`: Frecuencia del satélite en Hz
-   `range_rate`: Velocidad radial en m/s (de `predict-rs`)

**Nota:** El `range_rate` se obtiene automáticamente del `Observation` struct:

```rust
let observation = tracker.track(Utc::now())?;
let range_rate = observation.range_rate; // en m/s
```

## 🔬 Validación contra Skyfield

### ISS

```bash
cargo run --example comparar_con_skyfield       # Genera datos Rust
python3 src/validaciones/validar_iss.py         # Compara con Skyfield
```

**Resultado:** Diferencia promedio **13.06 Hz** (⚠️ ACEPTABLE)

### Otros Satélites

```bash
# 1. Generar datos Rust
cargo run --example track_satelite

# 2. Validar con Skyfield
python3 src/validaciones/validar_satelite.py
```

**Resultado (AO-91):** Diferencia promedio **9.42 Hz** (✅ BUENO)

## 📊 Resultados

Los resultados se guardan en:

-   `validacion_doppler/iss/` - ISS
-   `validacion_doppler/satelites/` - Otros satélites

## 🔧 Dependencias

### Rust

```toml
sgp4 = "2.3.0"           # Propagación orbital
predict-rs = "0.1.1"     # Predicción satélites
chrono = "0.4.41"        # Manejo de tiempo
```

### Python

```bash
pip install skyfield pandas matplotlib
```

### Fórmula Doppler

```
Doppler Shift (Hz) = -f₀ × (Vᵣ / c)

Donde:
  f₀ = Frecuencia transmitida (Hz)
  Vᵣ = Velocidad radial (m/s)
  c  = Velocidad de la luz (299,792,458 m/s)
```

## 🌍 Ubicación del Observador

Por defecto: **Buenos Aires, Argentina**

```rust
let observer = PredictObserver {
    latitude: -34.6037_f64.to_radians(),
    longitude: -58.3816_f64.to_radians(),
    altitude: 25.0,  // metros
    min_elevation: 0.0,
};
```

## 📝 Licencia

MIT

## 🙏 Agradecimientos

-   [sgp4](https://github.com/adjika-oss/sgp4) - Implementación SGP4 en Rust
-   [predict-rs](https://github.com/wose/predict-rs) - Librería de predicción satelital
-   [Skyfield](https://rhodesmill.org/skyfield/) - Referencia Python para validación
-   [CelesTrak](https://celestrak.org/) - TLEs actualizados
-   [SatNOGS DB](https://db.satnogs.org/) - Base de datos comunitaria de frecuencias satelitales
