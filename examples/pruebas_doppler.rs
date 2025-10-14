use chrono::{Duration, Utc};
use predict_rs::predict::PredictObserver;
use sgp4::{Constants, Elements};
use tracking::doppler::calcular_doppler;
use tracking::{frequencies, tle_loader};

fn main() {
    println!("=== VALIDACIÓN DOPPLER ===\n");

    println!("Obteniendo TLE de ISS...");
    let tle_data = match tle_loader::obtener_tle_por_nombre("ISS") {
        Ok(data) => {
            println!("✓ {}", data.name);
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
    .unwrap();
    let constants = Constants::from_elements(&elements).unwrap();

    let freq_tx = 437_500_000.0; // 437.5 MHz
    let when = Utc::now(); // Usar tiempo actual

    // Observador en Buenos Aires
    let observer = PredictObserver {
        latitude: -34.6037_f64.to_radians(),
        longitude: -58.3816_f64.to_radians(),
        altitude: 25.0,
        name: "Buenos Aires".to_string(),
        min_elevation: 0.0,
    };

    println!("\n[1] Evolución en 10 minutos");
    println!("{}", "-".repeat(50));

    let mut current_time = when;
    let mut shifts = Vec::new();

    for i in 0..10 {
        if let Some(shift) =
            calcular_doppler(&observer, &elements, &constants, freq_tx, current_time, 1)
        {
            shifts.push(shift);
            println!(
                "T+{:2} min | {:19} | Doppler: {:+8.1} Hz",
                i,
                current_time.format("%Y-%m-%d %H:%M:%S"),
                shift
            );
        }
        current_time += Duration::minutes(1);
    }

    println!("\n[2] Diferentes frecuencias");
    println!("{}", "-".repeat(50));

    // Obtener frecuencias reales de satélites
    let iss_freq = frequencies::obtener_frecuencia_por_nombre("ISS").unwrap_or(145_800_000.0);
    let ao91_freq = frequencies::obtener_frecuencia_por_nombre("AO-91").unwrap_or(145_960_000.0);

    let freq_bands = vec![
        ("ISS VHF", iss_freq),
        ("AO-91 VHF", ao91_freq),
        ("UHF", 437_500_000.0),
        ("S-Band", 2_200_000_000.0),
        ("X-Band", 8_400_000_000.0),
    ];

    for (band, freq) in freq_bands {
        if let Some(shift) = calcular_doppler(&observer, &elements, &constants, freq, when, 1) {
            println!(
                "{:8} ({:9.1} MHz) | Doppler: {:+10.1} Hz | Ratio: {:.1}%",
                band,
                freq / 1_000_000.0,
                shift,
                (shift / freq) * 100.0
            );
        }
    }

    println!("\n[3] Sensibilidad al intervalo (dt)");
    println!("{}", "-".repeat(50));

    let freq_test = 437_500_000.0;
    for dt in [1, 5, 10, 30, 60] {
        if let Some(shift) = calcular_doppler(&observer, &elements, &constants, freq_test, when, dt)
        {
            println!("dt = {:3} seg | Doppler: {:+8.1} Hz", dt, shift);
        }
    }

    // Análisis estadístico
    println!("\n[4] Análisis estadístico");
    println!("{}", "-".repeat(50));
    if !shifts.is_empty() {
        let max_shift = shifts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_shift = shifts.iter().cloned().fold(f64::INFINITY, f64::min);
        let avg_shift: f64 = shifts.iter().sum::<f64>() / shifts.len() as f64;

        println!("Doppler máximo:  {:+.1} Hz", max_shift);
        println!("Doppler mínimo:  {:+.1} Hz", min_shift);
        println!("Doppler promedio: {:+.1} Hz", avg_shift);
        println!("Rango total:      {:.1} Hz", max_shift - min_shift);

        // Validación de sanidad (para LEO @ UHF ~437 MHz)
        // Doppler máximo teórico: ~±11 kHz
        println!("\n✓ Validaciones:");
        let doppler_limite = 12000.0; // 12 kHz límite para LEO

        if max_shift.abs() < doppler_limite {
            println!(
                "  ✓ Doppler máximo OK ({:.1} kHz < {:.1} kHz)",
                max_shift / 1000.0,
                doppler_limite / 1000.0
            );
        } else {
            println!(
                "  ⚠ Doppler máximo fuera de rango ({:.1} kHz > {:.1} kHz)",
                max_shift / 1000.0,
                doppler_limite / 1000.0
            );
        }

        if min_shift.abs() < doppler_limite {
            println!(
                "  ✓ Doppler mínimo OK ({:.1} kHz < {:.1} kHz)",
                min_shift / 1000.0,
                doppler_limite / 1000.0
            );
        } else {
            println!(
                "  ⚠ Doppler mínimo fuera de rango ({:.1} kHz > {:.1} kHz)",
                min_shift / 1000.0,
                doppler_limite / 1000.0
            );
        }
    }
}
