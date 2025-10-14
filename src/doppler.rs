use chrono::{DateTime, Utc};
use predict_rs::predict::PredictObserver;
use sgp4::{Constants, Elements, MinutesSinceEpoch};
use std::f64::consts::PI;

// ====== Constantes físicas y astronómicas ========

/// Velocidad de la luz en metros por segundo
const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Semi-eje mayor de la Tierra según WGS84 (km)
const WGS84_A: f64 = 6378.137;

/// Factor de aplanamiento de la Tierra según WGS84
const WGS84_F: f64 = 1.0 / 298.257223563;

/// Fecha Juliana de la época J2000.0
const JD_J2000: f64 = 2451545.0;

/// Fecha Juliana de la época Unix (1 enero 1970)
const JD_UNIX_EPOCH: f64 = 2440587.5;

/// Segundos en un día
const SECONDS_PER_DAY: f64 = 86400.0;

/// Conversión de radianes a grados por segundo para GMST
const GMST_RAD_PER_SEC: f64 = PI / 43200.0;

/// Convierte coordenadas geodésicas (lat, lon, alt) a ECEF (Earth-Centered Earth-Fixed)
fn geodetic_to_ecef(lat_deg: f64, lon_deg: f64, alt_m: f64) -> [f64; 3] {
    let lat_rad = lat_deg * PI / 180.0;
    let lon_rad = lon_deg * PI / 180.0;

    // Primera excentricidad al cuadrado
    let e2 = 2.0 * WGS84_F - WGS84_F * WGS84_F;

    // Radio de curvatura
    let n = WGS84_A / (1.0 - e2 * lat_rad.sin().powi(2)).sqrt();

    // Posición en ECEF (km)
    let alt_km = alt_m / 1000.0;
    let x = (n + alt_km) * lat_rad.cos() * lon_rad.cos();
    let y = (n + alt_km) * lat_rad.cos() * lon_rad.sin();
    let z = (n * (1.0 - e2) + alt_km) * lat_rad.sin();

    [x, y, z]
}

/// Calcula GMST (Greenwich Mean Sidereal Time) en radianes
fn calculate_gmst(unix_timestamp: f64) -> f64 {
    let jd = JD_UNIX_EPOCH + unix_timestamp / SECONDS_PER_DAY;
    let tu = (jd - JD_J2000) / 36525.0;

    let gmst_sec = 67310.54841 + (876600.0 * 3600.0 + 8640184.812866) * tu + 0.093104 * tu * tu
        - 6.2e-6 * tu * tu * tu;

    (gmst_sec % SECONDS_PER_DAY) * GMST_RAD_PER_SEC
}

/// Convierte posición TEME a ECEF
fn teme_to_ecef(teme_pos: [f64; 3], gmst_rad: f64) -> [f64; 3] {
    let cos_gmst = gmst_rad.cos();
    let sin_gmst = gmst_rad.sin();

    [
        teme_pos[0] * cos_gmst + teme_pos[1] * sin_gmst,
        -teme_pos[0] * sin_gmst + teme_pos[1] * cos_gmst,
        teme_pos[2],
    ]
}

pub fn calcular_rango(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    when: DateTime<Utc>,
) -> Option<f64> {
    // Calcular minutos desde epoch
    let epoch_timestamp = elements.datetime.and_utc().timestamp() as f64;
    let when_timestamp = when.timestamp() as f64;
    let minutes_since_epoch = (when_timestamp - epoch_timestamp) / 60.0;

    // Propagar satélite con SGP4
    let prediction = constants
        .propagate(MinutesSinceEpoch(minutes_since_epoch))
        .ok()?;

    // Calcular GMST para convertir TEME a ECEF
    let gmst = calculate_gmst(when_timestamp);

    // Convertir posición del satélite de TEME a ECEF
    let sat_ecef = teme_to_ecef(prediction.position, gmst);

    // Convertir observador a ECEF
    let lat_deg = observer.latitude * 180.0 / PI;
    let lon_deg = observer.longitude * 180.0 / PI;
    let obs_ecef = geodetic_to_ecef(lat_deg, lon_deg, observer.altitude);

    // Vector desde observador a satélite
    let dx = sat_ecef[0] - obs_ecef[0];
    let dy = sat_ecef[1] - obs_ecef[1];
    let dz = sat_ecef[2] - obs_ecef[2];

    // Distancia (slant range) en metros
    let range_km = (dx * dx + dy * dy + dz * dz).sqrt();
    Some(range_km * 1000.0)
}

pub fn calcular_doppler(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    freq_tx: f64,
    when: DateTime<Utc>,
    dt_secs: i64,
) -> Option<f64> {
    // Calcular rango en dos momentos para obtener range_rate por diferencias finitas
    let rango1 = calcular_rango(observer, elements, constants, when)?;

    let when2 = when + chrono::Duration::seconds(dt_secs);
    let rango2 = calcular_rango(observer, elements, constants, when2)?;

    let range_rate = (rango2 - rango1) / (dt_secs as f64); // m/s

    // Calcular Doppler shift
    Some(-freq_tx * (range_rate / SPEED_OF_LIGHT))
}
