use chrono::Utc;
use predict_rs::predict::PredictObserver;
use sgp4::{Constants, Elements};
use std::fs;
use tracking::tle_loader;
use tracking::validaciones::generar_comparacion;

fn main() {
    println!("=== VALIDACIÓN DOPPLER ISS ===\n");

    println!("[1] Descargando TLE...");

    let tle_data = match tle_loader::obtener_tle_por_nombre("ISS") {
        Ok(data) => {
            println!("✓ {}", data.name);

            let tle_content = format!("{}\n{}\n{}\n", data.name, data.line1, data.line2);
            fs::create_dir_all("validacion_doppler/iss").ok();
            fs::write("validacion_doppler/iss/iss_tle.txt", tle_content).ok();

            data
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            return;
        }
    };

    let elements = match Elements::from_tle(
        Some(tle_data.name.clone()),
        tle_data.line1.as_bytes(),
        tle_data.line2.as_bytes(),
    ) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("✗ Error: {:?}", e);
            return;
        }
    };

    let constants = match Constants::from_elements(&elements) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ Error: {:?}", e);
            return;
        }
    };

    // Observador
    let observer = PredictObserver {
        latitude: -34.6037_f64.to_radians(),
        longitude: -58.3816_f64.to_radians(),
        altitude: 25.0,
        name: "Buenos Aires".to_string(),
        min_elevation: 0.0,
    };

    println!("✓ Observador: Buenos Aires");

    // Generar datos
    println!("\n[2] Generando datos...");

    let inicio = Utc::now();
    match generar_comparacion(&observer, &elements, &constants, inicio, 90) {
        Ok(_) => {
            println!("\n✓ CSV: validacion_doppler/iss/doppler_output.csv");
            println!("\nComparar con: python3 src/validaciones/validar_iss.py");
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
        }
    }
}
