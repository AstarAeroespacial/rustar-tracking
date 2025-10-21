use chrono::{Duration, Utc};
use predict_rs::predict::PredictObserver;
use sgp4::{Constants, Elements};
use std::fs;
use tracking::{frequencies, tle_loader};

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

        // Calcular observación usando las funciones de doppler
        let rango_opt = tracking::doppler::calcular_rango(&observer, &elements, &constants, t);
        let doppler_opt = tracking::doppler::calcular_doppler(
            &observer, &elements, &constants, freq_hz, t,
            10, // dt de 10 segundos para calcular range_rate
        );

        if let (Some(rango_m), Some(doppler_hz)) = (rango_opt, doppler_opt) {
            // Calcular range rate usando diferencias finitas
            let dt = Duration::seconds(10);
            let t2 = t + dt;

            if let Some(rango2_m) =
                tracking::doppler::calcular_rango(&observer, &elements, &constants, t2)
            {
                let range_rate_m_s = (rango2_m - rango_m) / 10.0;

                // Calcular elevación para el CSV
                let sat_orbit =
                    predict_rs::orbit::predict_orbit(&elements, &constants, t.timestamp() as f64);

                if let Ok(orbit) = sat_orbit {
                    let observation =
                        predict_rs::observer::predict_observe_orbit(&observer, &orbit);
                    let elevation_deg = observation.elevation.to_degrees();

                    csv_data.push_str(&format!(
                        "{},{:.6},{:.6},{:.6},{:.6}\n",
                        t, rango_m, range_rate_m_s, doppler_hz, elevation_deg
                    ));
                }
            }
        }
    }

    // Guardar archivo
    fs::write("validacion_doppler/satelites/doppler_output.csv", csv_data)
        .expect("Error al escribir CSV");

    println!("\n✓ CSV: validacion_doppler/satelites/doppler_output.csv");
}
