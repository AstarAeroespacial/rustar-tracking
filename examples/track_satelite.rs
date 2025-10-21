use chrono::{Duration, Utc};
use predict_rs::{observer::predict_observe_orbit, orbit::predict_orbit, predict::PredictObserver};
use sgp4::{Constants, Elements};
use std::fs;
use tracking::{doppler_downlink, frequencies, tle_loader};

fn main() {
    println!("=== VALIDACIÓN DOPPLER - SATÉLITE ===\n");

    println!("[1] Descargando TLE...");

    let tle_data = match tle_loader::obtener_tle_por_nombre("AO-91") {
        Ok(data) => {
            println!("✓ {}", data.name);

            // Guardar TLE
            let tle_content = format!("{}\n{}\n{}\n", data.name, data.line1, data.line2);
            fs::create_dir_all("validacion_doppler/satelites").ok();
            fs::write("validacion_doppler/satelites/satelite_tle.txt", tle_content).ok();

            data
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            return;
        }
    };

    let elements = Elements::from_tle(
        Some(tle_data.name.clone()),
        tle_data.line1.as_bytes(),
        tle_data.line2.as_bytes(),
    )
    .expect("Error al parsear TLE");

    let constants = Constants::from_elements(&elements).expect("Error al crear constantes");

    // Configurar observador
    let observer = PredictObserver {
        name: "Buenos Aires".to_string(),
        latitude: (-34.6037_f64).to_radians(),
        longitude: (-58.3816_f64).to_radians(),
        altitude: 25.0,
        min_elevation: 0.0,
    };

    println!("✓ Observador: Buenos Aires");

    // Obtener frecuencia
    println!("\n[2] Obteniendo frecuencia...");
    let freq_hz =
        frequencies::obtener_frecuencia_por_nombre("AO-91").expect("Frecuencia no encontrada");
    println!("✓ {:.3} MHz", freq_hz / 1_000_000.0);

    // Generar datos
    println!("\n[3] Generando datos...");

    let start_time = Utc::now();
    let freq_mhz = freq_hz / 1_000_000.0;
    let mut csv_data = format!(
        "timestamp,range_m,range_rate_m_s,doppler_{:.2}MHz_Hz,elevation_deg\n",
        freq_mhz
    );
    let duracion_minutos = 180;
    let intervalo_segundos = 60;
    let total_puntos = (duracion_minutos * 60) / intervalo_segundos;

    for i in 0..total_puntos {
        let t = start_time + Duration::seconds((i * intervalo_segundos) as i64);

        // Calcular observación usando predict-rs
        if let Ok(sat_orbit) = predict_orbit(&elements, &constants, t.timestamp() as f64) {
            let observation = predict_observe_orbit(&observer, &sat_orbit);

            let rango_m = observation.range * 1000.0; // km to m
            let range_rate_m_s = observation.range_rate * 1000.0; // km/s to m/s
            let freq_rx = doppler_downlink(freq_hz, range_rate_m_s);
            let doppler_hz = freq_rx - freq_hz;

            csv_data.push_str(&format!(
                "{},{:.6},{:.6},{:.6}\n",
                t, rango_m, range_rate_m_s, doppler_hz
            ));
        }
    }

    // Guardar archivo
    fs::write("validacion_doppler/satelites/doppler_output.csv", csv_data)
        .expect("Error al escribir CSV");

    println!("\n✓ CSV: validacion_doppler/satelites/doppler_output.csv");
}
