# Tracking con Corrección de Doppler

Todos los ejemplos descargan automáticamente TLEs actualizados desde CelesTrak.

```bash
# Ejemplo completo con predicción de pases y corrección de Doppler
cargo run --example track_doppler
```

Este ejemplo muestra:

-   🔍 Predicción automática de pases (AOS/LOS)
-   📡 Tracking en tiempo real con elevación y azimut
-   🎯 Corrección de frecuencia del receptor
-   📊 Tabla con Doppler shift y frecuencia RX a sintonizar

## Validar ISS

```bash
cargo run --example comparar_con_skyfield       # Genera datos Rust
python3 src/validacion_doppler/validar_iss.py   # Compara con Skyfield
```

## Validar Otros Satélites

```bash
# 1. Generar datos Rust
cargo run --example track_satelite

# 2. Validar con Skyfield
python3 src/validacion_doppler/validar_satelite.py
```

## Validación de Cálculos

```bash
# Pruebas de Doppler con TLE actualizado
cargo run --example validar_doppler
```

## 📊 Resultados

Los resultados se guardan en:

-   `validacion_doppler/iss/` - ISS
-   `validacion_doppler/satelites/` - Otros satélites

## 🎯 Satélites Disponibles

| Satélite           | NORAD | Frecuencia  | Notas                      |
| ------------------ | ----- | ----------- | -------------------------- |
| **ISS**            | 25544 | 145.800 MHz | Referencia estable         |
| **AO-91** (Fox-1B) | 43017 | 145.960 MHz | Muy estable, ideal Doppler |
| **FO-29** (JAS-2)  | 24278 | 435.850 MHz | UHF, órbita excéntrica     |
| **FUNCUBE-1**      | 39444 | 145.935 MHz | BPSK 1200 bps              |
| **LILACSAT-2**     | 40069 | 437.200 MHz | UHF, ±3 kHz Doppler        |

**Nota**: Las frecuencias se descargan automáticamente desde **SatNOGS DB** (base de datos comunitaria) o se usan valores locales como fallback.

## Validaciones Realizadas

### ISS (NORAD 25544)

-   **Órbita**: ~400 km, inclinación 51.6°
-   **Resultado**: Doppler < 2 Hz, Rango ~1 km
-   **Evaluación**: ✅ EXCELENTE

### AO-91 / JAS-2 (NORAD 24278)

-   **Órbita**: ~1060 km, inclinación 98.5°, excéntrica (e=0.035)
-   **Frecuencia**: 145.960 MHz (VHF downlink)
-   **Resultado**: Doppler 1.22 Hz, Rango ~2.1 km
-   **Evaluación**: ✅ EXCELENTE

## Precisión del Sistema

### Doppler Shift

-   **Precisión**: < 10 Hz (típicamente 1-2 Hz)
-   **Evaluación**: Excelente para recepción automática de señales
-   **Método**: Diferencias finitas (10 segundos), igual que Skyfield

**Validado contra**: Skyfield (implementación de referencia Python)

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

## 📖 Cómo Funciona

1. **SGP4**: Propaga la órbita del satélite usando TLE
2. **Coordinate Transform**: Convierte ECI → Topocentric (observer frame)
3. **Range Rate**: Calcula velocidad radial por diferencias finitas
4. **Doppler**: Aplica fórmula: `shift = -freq × (range_rate / c)`

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

Modificar en los ejemplos según tu ubicación.

## 📝 Licencia

MIT

## 🙏 Agradecimientos

-   [sgp4](https://github.com/adjika-oss/sgp4) - Implementación SGP4 en Rust
-   [predict-rs](https://github.com/wose/predict-rs) - Librería de predicción satelital
-   [Skyfield](https://rhodesmill.org/skyfield/) - Referencia Python para validación
-   [CelesTrak](https://celestrak.org/) - TLEs actualizados
-   [SatNOGS DB](https://db.satnogs.org/) - Base de datos comunitaria de frecuencias satelitales
