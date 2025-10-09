use chrono::{Duration, Utc};
use predict_rs::predict::PredictObserver;
use sgp4::{Constants, Elements};
use std::thread::sleep;
use std::time::Duration as StdDuration;
use tracking::doppler::calcular_doppler;

fn main() {
    // Ejemplo de TLE (ISS)
    let tle_name = "ISS (ZARYA)";
    let tle1 = "1 25544U 98067A   21275.51041667  .00000282  00000-0  12922-4 0  9993";
    let tle2 = "2 25544  51.6442  21.3932 0003577  80.2342  41.1234 15.48815328299929";

    // Elementos orbitales y constantes
    let elements =
        Elements::from_tle(Some(tle_name.to_string()), tle1.as_bytes(), tle2.as_bytes()).unwrap();
    let constants = Constants::from_elements(&elements).unwrap();

    // Observador (ejemplo: Buenos Aires)
    let observer = PredictObserver {
        latitude: -34.6037_f64.to_radians(),
        longitude: -58.3816_f64.to_radians(),
        altitude: 25.0,
        name: todo!(),
        min_elevation: todo!(), // metros
    };

    // Frecuencia de transmisión (ejemplo: 437.5 MHz)
    let freq_tx = 437_500_000.0;

    // Tiempo inicial
    let mut now = Utc::now();
    let dt_secs = 1;

    println!("Tracking Doppler shift for ISS:");
    for _ in 0..30 {
        if let Some(shift) =
            calcular_doppler(&observer, &elements, &constants, freq_tx, now, dt_secs)
        {
            let freq_rx = freq_tx + shift;
            println!(
                "UTC: {} | Doppler: {:+.1} Hz | Freq RX: {:.1} Hz",
                now, shift, freq_rx
            );
        } else {
            println!("No se pudo calcular el Doppler");
        }
        now = now + Duration::seconds(dt_secs);
        sleep(StdDuration::from_secs(dt_secs as u64));
    }
}
