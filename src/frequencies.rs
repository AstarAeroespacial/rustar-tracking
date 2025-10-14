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
    // Buscar el primer objeto con "downlink_low" válido
    // JSON de SatNOGS viene en una sola línea, así que buscamos patrones

    // Extraer descripción del primer transmisor
    let description = if let Some(desc_start) = json.find("\"description\":\"") {
        let desc_start = desc_start + 15; // Saltar "description":"
        if let Some(desc_end) = json[desc_start..].find('\"') {
            json[desc_start..desc_start + desc_end].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Buscar downlink_low (primer valor no nulo)
    let downlink_hz = if let Some(down_start) = json.find("\"downlink_low\":") {
        let down_start = down_start + 15; // Saltar "downlink_low":
        if let Some(comma_pos) = json[down_start..].find(',') {
            let num_str = json[down_start..down_start + comma_pos].trim();
            if num_str != "null" {
                num_str.parse::<f64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Buscar uplink_low
    let uplink_hz = if let Some(up_start) = json.find("\"uplink_low\":") {
        let up_start = up_start + 13; // Saltar "uplink_low":
        if let Some(comma_pos) = json[up_start..].find(',') {
            let num_str = json[up_start..up_start + comma_pos].trim();
            if num_str != "null" {
                num_str.parse::<f64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Buscar mode
    let mode = if let Some(mode_start) = json.find("\"mode\":\"") {
        let mode_start = mode_start + 8; // Saltar "mode":"
        if let Some(mode_end) = json[mode_start..].find('\"') {
            json[mode_start..mode_start + mode_end].to_string()
        } else {
            String::from("Unknown")
        }
    } else {
        String::from("Unknown")
    };

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
