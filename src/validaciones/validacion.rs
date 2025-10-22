use crate::frequencies;
use chrono::{DateTime, Duration, Utc};
use predict_rs::{observer::predict_observe_orbit, orbit::predict_orbit, predict::PredictObserver};
use sgp4::{Constants, Elements};
use std::fs::File;
use std::io::Write;

pub fn generar_comparacion(
    observer: &PredictObserver,
    elements: &Elements,
    constants: &Constants,
    inicio: DateTime<Utc>,
    duracion_mins: i64,
) -> std::io::Result<()> {
    // Obtener frecuencia de ISS
    let freq =
        frequencies::obtener_frecuencia_por_nombre("ISS").expect("Frecuencia de ISS no encontrada");
    let freq_mhz = freq / 1_000_000.0;

    let mut file = File::create("validacion_doppler/iss/doppler_output.csv")?;
    writeln!(
        file,
        "timestamp,range_m,range_rate_m_s,doppler_{:.1}MHz_Hz",
        freq_mhz
    )?;

    let mut puntos_validos = 0;
    let mut puntos_invalidos = 0;

    print!("Generando datos");

    for minuto in 0..duracion_mins {
        let cuando = inicio + Duration::minutes(minuto);

        // Obtener la órbita y observación directamente
        if let Ok(sat_orbit) = predict_orbit(elements, constants, cuando.timestamp() as f64) {
            let observation = predict_observe_orbit(observer, &sat_orbit);

            let rango = observation.range * 1000.0; // km a metros
            let range_rate = observation.range_rate * 1000.0; // km/s a m/s

            // Calcular Doppler usando range_rate
            let doppler = -freq * (range_rate / 299_792_458.0); // SPEED_OF_LIGHT

            writeln!(
                file,
                "{},{:.0},{:.2},{:.0}",
                cuando.to_rfc3339(),
                rango,
                range_rate,
                doppler
            )?;

            puntos_validos += 1;

            // Mostrar progreso cada 10 puntos
            if (minuto + 1) % 10 == 0 {
                print!(".");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
        } else {
            puntos_invalidos += 1;
        }
    }

    println!(" ✓");
    println!("{} válidos, {} inválidos", puntos_validos, puntos_invalidos);

    Ok(())
}
