//! Frecuencias de downlink de satélites LEO
//!
//! Este módulo descarga frecuencias actualizadas desde SatNOGS DB API
//! (https://db.satnogs.org/api/transmitters/)

use std::io;
use std::process::Command;

/// Estructura con información completa de frecuencias de un satélite
#[derive(Debug, Clone)]
pub struct SatelliteFrequencies {
    pub name: String,
    pub norad_id: u32,
    /// Frecuencia de downlink en Hz
    pub downlink_hz: f64,
    /// Frecuencia de uplink en Hz (si aplica)
    pub uplink_hz: Option<f64>,
    /// Modo de operación (FM, SSB, CW, etc.)
    pub mode: String,
}

/// Descarga frecuencias desde SatNOGS DB API
///
/// # Argumentos
/// * `norad_id` - El ID NORAD del satélite
pub fn descargar_frecuencias_satnogs(norad_id: u32) -> io::Result<SatelliteFrequencies> {
    let url = format!(
        "https://db.satnogs.org/api/transmitters/?satellite__norad_cat_id={}&format=json&status=active",
        norad_id
    );

    let output = Command::new("curl").args(["-s", &url]).output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error al descargar frecuencias desde SatNOGS",
        ));
    }

    let content = String::from_utf8_lossy(&output.stdout);

    parse_satnogs_json(&content, norad_id)
}

/// Parser simple de JSON de SatNOGS
fn parse_satnogs_json(json: &str, norad_id: u32) -> io::Result<SatelliteFrequencies> {
    // Buscar "description", "downlink_low", "uplink_low", "mode"
    let mut description = String::new();
    let mut downlink_hz = None;
    let mut uplink_hz = None;
    let mut mode = String::from("Unknown");

    // Parser simple línea por línea
    for line in json.lines() {
        let line = line.trim();

        if line.contains("\"description\":") {
            if let Some(value) = extract_json_string(line) {
                description = value;
            }
        } else if line.contains("\"downlink_low\":") {
            if let Some(value) = extract_json_number(line) {
                downlink_hz = Some(value);
            }
        } else if line.contains("\"uplink_low\":") {
            if let Some(value) = extract_json_number(line) {
                uplink_hz = Some(value);
            }
        } else if line.contains("\"mode\":") {
            if let Some(value) = extract_json_string(line) {
                mode = value;
            }
        }
    }

    let downlink = downlink_hz.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "No se encontró frecuencia de downlink en SatNOGS",
        )
    })?;

    Ok(SatelliteFrequencies {
        name: description,
        norad_id,
        downlink_hz: downlink,
        uplink_hz,
        mode,
    })
}

fn extract_json_string(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

fn extract_json_number(line: &str) -> Option<f64> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 2 {
        let num_str = parts[1].trim().trim_end_matches(',');
        if num_str == "null" {
            None
        } else {
            num_str.parse::<f64>().ok()
        }
    } else {
        None
    }
}

/// Base de datos local de satélites con sus frecuencias (fallback)
fn get_satellite_info_local(satellite_name: &str) -> Option<SatelliteFrequencies> {
    let satellites = vec![
        SatelliteFrequencies {
            name: "ISS".to_string(),
            norad_id: 25544,
            downlink_hz: 145_800_000.0,
            uplink_hz: Some(145_200_000.0),
            mode: "FM".to_string(),
        },
        SatelliteFrequencies {
            name: "AO-91".to_string(),
            norad_id: 43017,
            downlink_hz: 145_960_000.0,
            uplink_hz: Some(435_250_000.0),
            mode: "FM".to_string(),
        },
        SatelliteFrequencies {
            name: "FO-29".to_string(),
            norad_id: 24278,
            downlink_hz: 435_850_000.0,
            uplink_hz: Some(145_900_000.0),
            mode: "SSB/CW".to_string(),
        },
        SatelliteFrequencies {
            name: "FUNCUBE-1".to_string(),
            norad_id: 39444,
            downlink_hz: 145_935_000.0,
            uplink_hz: None,
            mode: "BPSK".to_string(),
        },
        SatelliteFrequencies {
            name: "LILACSAT-2".to_string(),
            norad_id: 40069,
            downlink_hz: 437_200_000.0,
            uplink_hz: None,
            mode: "GMSK".to_string(),
        },
    ];

    satellites.into_iter().find(|s| {
        s.name.to_uppercase() == satellite_name.to_uppercase()
            || s.norad_id.to_string() == satellite_name
    })
}

/// Obtiene información de frecuencias de un satélite
///
/// Primero intenta descargar desde SatNOGS API.
/// Si falla, usa la base de datos local como fallback.
///
/// # Argumentos
/// * `satellite_name` - Nombre del satélite o NORAD ID como string
pub fn get_satellite_info(satellite_name: &str) -> Option<SatelliteFrequencies> {
    // Primero intentar obtener NORAD ID
    let norad_id = if let Ok(id) = satellite_name.parse::<u32>() {
        // Ya es un NORAD ID
        id
    } else {
        // Buscar en base de datos local para obtener NORAD ID
        get_satellite_info_local(satellite_name)?.norad_id
    };

    // Intentar descargar desde SatNOGS
    match descargar_frecuencias_satnogs(norad_id) {
        Ok(freq) => {
            println!("✓ Frecuencias desde SatNOGS");
            Some(freq)
        }
        Err(e) => {
            eprintln!("⚠ Error SatNOGS: {}", e);
            println!("⚠ Usando base de datos local");
            get_satellite_info_local(satellite_name)
        }
    }
}

/// Obtiene solo la frecuencia de downlink de un satélite
///
/// # Argumentos
/// * `satellite_name` - Nombre del satélite o NORAD ID como string
pub fn obtener_frecuencia_por_nombre(satellite_name: &str) -> Option<f64> {
    get_satellite_info(satellite_name).map(|info| info.downlink_hz)
}
