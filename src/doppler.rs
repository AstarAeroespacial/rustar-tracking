use chrono::{DateTime, Duration as ChronoDuration, Utc};
use predict_rs::{orbit::predict_orbit, predict::PredictObserver};
use sgp4::{Constants, Elements};

fn geodetic_to_ecef(lat_rad: f64, lon_rad: f64, alt_m: f64) -> [f64; 3] {
    let a = 6378137.0;
    let e2 = 6.69437999014e-3;

    let sin_lat = lat_rad.sin();
    let cos_lat = lat_rad.cos();
    let cos_lon = lon_rad.cos();
    let sin_lon = lon_rad.sin();

    let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();

    let x = (n + alt_m) * cos_lat * cos_lon;
    let y = (n + alt_m) * cos_lat * sin_lon;
    let z = ((1.0 - e2) * n + alt_m) * sin_lat;

    [x, y, z]
}

fn calcular_rango(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    when: DateTime<Utc>,
) -> Option<f64> {
    let sat_orbit = predict_orbit(elements, constants, when.timestamp() as f64).ok()?;
    let sat_pos = sat_orbit.position; // [x, y, z] en km

    let obs_pos = geodetic_to_ecef(
        observer.latitude,  // en radianes
        observer.longitude, // en radianes
        observer.altitude,  // en metros
    );

    let obs_pos_km = [
        obs_pos[0] / 1000.0,
        obs_pos[1] / 1000.0,
        obs_pos[2] / 1000.0,
    ];

    let dx = sat_pos.0 - obs_pos_km[0];
    let dy = sat_pos.1 - obs_pos_km[1];
    let dz = sat_pos.2 - obs_pos_km[2];
    Some((dx * dx + dy * dy + dz * dz).sqrt() * 1000.0) // metros
}

pub fn calcular_doppler(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    freq_tx: f64,
    when: DateTime<Utc>,
    dt_secs: i64,
) -> Option<f64> {
    let rango1 = calcular_rango(observer, elements, constants, when)?;
    let rango2 = calcular_rango(
        observer,
        elements,
        constants,
        when + ChronoDuration::seconds(dt_secs),
    )?;
    let rate = (rango2 - rango1) / (dt_secs as f64);

    let c = 299_792_458.0;
    Some(-freq_tx * (rate / c))
}
