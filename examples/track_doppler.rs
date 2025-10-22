/// Ejemplo completo de tracking satelital con corrección de Doppler
///
/// Este programa muestra cómo hacer el tracking correcto de un satélite,
/// incluyendo:
/// - Predicción de pases (AOS/LOS - Acquisition of Signal / Loss of Signal)
/// - Verificación de elevación mínima
/// - Cálculo de Doppler en tiempo real durante el pase
/// - Corrección de frecuencia del receptor
///
/// Flujo de tracking:
/// 1. Esperar a que el satélite aparezca sobre el horizonte (AOS)
/// 2. Durante el pase, calcular continuamente el Doppler shift
/// 3. Ajustar la frecuencia del receptor: freq_rx = freq_tx + doppler_shift
/// 4. Finalizar cuando el satélite desaparezca bajo el horizonte (LOS)
use chrono::{DateTime, Duration, Utc};
use predict_rs::{observer::predict_observe_orbit, orbit::predict_orbit, predict::PredictObserver};
use sgp4::{Constants, Elements};
use tracking::doppler_downlink;
use tracking::tle_loader;

/// Encuentra el próximo pase del satélite con elevación > elevación mínima
fn encontrar_proximo_pase(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    start_time: DateTime<Utc>,
    max_hours: i64,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let mut current_time = start_time;
    let end_search = start_time + Duration::hours(max_hours);
    let mut in_pass = false;
    let mut aos_time = None;

    // Buscar en intervalos de 1 minuto
    while current_time < end_search {
        let sat_orbit = predict_orbit(elements, constants, current_time.timestamp() as f64).ok()?;
        let observation = predict_observe_orbit(observer, &sat_orbit);

        let elevation_deg = observation.elevation.to_degrees();
        let is_visible = elevation_deg > observer.min_elevation.to_degrees();

        if is_visible && !in_pass {
            // AOS - Acquisition of Signal
            aos_time = Some(current_time);
            in_pass = true;
        } else if !is_visible && in_pass {
            // LOS - Loss of Signal
            return Some((aos_time.unwrap(), current_time));
        }

        current_time += Duration::minutes(1);
    }

    None
}

/// Trackea un pase completo del satélite con corrección de Doppler
#[derive(Debug)]
#[allow(dead_code)]
struct Observacion {
    tiempo: DateTime<Utc>,
    elevacion: f64,
    azimut: f64,
    doppler_hz: f64,
    range_rate: f64,
}

fn trackear_pase(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    freq_tx: f64,
    aos: DateTime<Utc>,
    los: DateTime<Utc>,
) {
    println!("\n=== TRACKING ===");
    println!("AOS: {} UTC", aos.format("%H:%M:%S"));
    println!("LOS: {} UTC", los.format("%H:%M:%S"));
    println!(
        "Duración: {:.1} min",
        (los - aos).num_seconds() as f64 / 60.0
    );

    println!("\n{:<10} |      ANTENA     |         RECEPTOR", "Tiempo");
    println!(
        "{:<10} | {:>7} {:>7} | {:>11} {:>12}",
        "", "Elev°", "Az°", "Doppler(Hz)", "RX(MHz)"
    );
    println!("{}", "-".repeat(60));

    let mut current_time = aos;
    let update_interval = 5; // actualizar cada 5 segundos

    let mut observaciones: Vec<Observacion> = Vec::new();

    while current_time <= los {
        // Obtener posición del satélite
        let sat_orbit = predict_orbit(elements, constants, current_time.timestamp() as f64)
            .expect("Error al predecir órbita");
        let observation = predict_observe_orbit(observer, &sat_orbit);

        let elevation_deg = observation.elevation.to_degrees();
        let azimuth_deg = observation.azimuth.to_degrees();
        let range_rate = observation.range_rate * 1000.0; // Convertir a m/s

        // Verificar que seguimos visible
        if elevation_deg < observer.min_elevation.to_degrees() {
            println!(
                "{:<10} | {:>7.2} {:>7.2} | [Bajo horizonte]",
                current_time.format("%H:%M:%S"),
                elevation_deg,
                azimuth_deg
            );
            current_time += Duration::seconds(update_interval);
            continue;
        }

        // Calcular Doppler usando la nueva función
        let freq_rx = doppler_downlink(freq_tx, range_rate);
        let doppler_hz = freq_rx - freq_tx;

        println!(
            "{:<10} | {:>7.2} {:>7.2} | {:>11.2} {:>12.6}",
            current_time.format("%H:%M:%S"),
            elevation_deg,
            azimuth_deg,
            doppler_hz,
            freq_rx / 1_000_000.0
        );

        // Crear y almacenar la observación
        let observacion = Observacion {
            tiempo: current_time,
            elevacion: elevation_deg,
            azimut: azimuth_deg,
            doppler_hz,
            range_rate,
        };
        observaciones.push(observacion);

        current_time += Duration::seconds(update_interval);
    }

    println!("\n✓ Pase completado\n");

    // Mostrar todas las observaciones al final
    println!("Observaciones completas: {:?}", observaciones);
}

fn main() {
    println!("=== TRACKING CON CORRECCIÓN DOPPLER ===\n");

    // Obtener TLE actualizado de la ISS
    let tle_data = match tle_loader::obtener_tle_por_nombre("ISS") {
        Ok(data) => {
            println!("✓ TLE: {}", data.name);
            data
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            eprintln!("Usando TLE de respaldo");
            tle_loader::TleData {
                name: "ISS (ZARYA)".to_string(),
                line1: "1 25544U 98067A   25286.81616349  .00012055  00000+0  21953-3 0  9996"
                    .to_string(),
                line2: "2 25544  51.6332  79.1379 0000798 266.7872  93.3025 15.49912173533566"
                    .to_string(),
            }
        }
    };

    let elements = Elements::from_tle(
        Some(tle_data.name.clone()),
        tle_data.line1.as_bytes(),
        tle_data.line2.as_bytes(),
    )
    .expect("Error al cargar TLE");

    let constants = Constants::from_elements(&elements).expect("Error al crear constantes SGP4");

    // Observador en Buenos Aires
    let observer = PredictObserver {
        latitude: -34.6037_f64.to_radians(),
        longitude: -58.3816_f64.to_radians(),
        altitude: 25.0,
        name: "Buenos Aires".to_string(),
        min_elevation: 10.0_f64.to_radians(), // Solo trackear sobre 10° de elevación
    };

    // Frecuencia de transmisión del satélite (ejemplo: banda UHF amateur)
    let freq_tx = 437_500_000.0; // 437.5 MHz

    println!("Satélite: {}", tle_data.name);
    println!(
        "Observador: {} ({:.4}°, {:.4}°, {} m)",
        observer.name,
        observer.latitude.to_degrees(),
        observer.longitude.to_degrees(),
        observer.altitude
    );
    println!(
        "Frecuencia TX del satélite: {:.3} MHz",
        freq_tx / 1_000_000.0
    );
    println!(
        "Elevación mínima: {:.1}°",
        observer.min_elevation.to_degrees()
    );

    println!("\n🔍 Buscando próximo pase...");
    let now = Utc::now();

    match encontrar_proximo_pase(&observer, &elements, &constants, now, 24) {
        Some((aos, los)) => {
            let time_until_aos = aos - now;
            println!("✓ Pase encontrado");
            println!(
                "  Comienza en: {} ({:.1} minutos desde ahora)",
                aos.format("%Y-%m-%d %H:%M:%S UTC"),
                time_until_aos.num_seconds() as f64 / 60.0
            );

            if time_until_aos.num_seconds() > 0 {
                println!("\n⏳ Esperando hasta AOS...");
                println!("   (En una aplicación real, aquí esperarías o programarías el tracking)");
                println!("   (Para este ejemplo, simularemos el tracking del pase)\n");
            }

            // Trackear el pase completo
            trackear_pase(&observer, &elements, &constants, freq_tx, aos, los);

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("IMPORTANTE - Cómo usar esta información:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("ANTENA (columna izquierda):");
            println!("  - Usa 'Elev°' y 'Az°' para apuntar la antena físicamente");
            println!("  - La antena NO necesita corrección Doppler");
            println!();
            println!("RECEPTOR (columna derecha):");
            println!(
                "  - El satélite transmite en: {:.6} MHz (frecuencia fija)",
                freq_tx / 1_000_000.0
            );
            println!("  - TÚ debes sintonizar en la frecuencia 'RX(MHz)' mostrada");
            println!("  - El Doppler es positivo cuando se acerca, negativo al alejarse");
            println!("  - Corrección máxima para LEO @ 437 MHz: ~±3 kHz");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        None => {
            println!("✗ No se encontraron pases en las próximas 24 horas");
        }
    }
}
