use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct TleData {
    pub name: String,
    pub line1: String,
    pub line2: String,
}

pub fn cargar_tle_desde_archivo<P: AsRef<Path>>(path: P) -> io::Result<TleData> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Limpiar líneas vacías
    lines.retain(|line| !line.trim().is_empty());

    if lines.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "TLE inválido: se esperaban 3 líneas, se encontraron {}",
                lines.len()
            ),
        ));
    }

    Ok(TleData {
        name: lines[0].trim().to_string(),
        line1: lines[1].trim().to_string(),
        line2: lines[2].trim().to_string(),
    })
}

/// Descarga un TLE actualizado desde CelesTrak para un satélite específico
///
/// # Argumentos
/// * `norad_id` - El ID NORAD del satélite (ej: 25544 para ISS)
pub fn descargar_tle_celestrak(norad_id: u32) -> io::Result<TleData> {
    let url = format!(
        "https://celestrak.org/NORAD/elements/gp.php?CATNR={}&FORMAT=TLE",
        norad_id
    );

    let output = Command::new("curl").args(["-s", &url]).output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error al descargar TLE desde CelesTrak",
        ));
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Respuesta de CelesTrak no contiene un TLE válido",
        ));
    }

    Ok(TleData {
        name: lines[0].trim().to_string(),
        line1: lines[1].trim().to_string(),
        line2: lines[2].trim().to_string(),
    })
}

/// Obtiene el TLE de un satélite por su nombre
///
/// Soporta los siguientes satélites:
/// - ISS (NORAD 25544)
/// - AO-91 / FOX-1B / RADFXSAT (NORAD 43017)
/// - FO-29 / JAS-2 (NORAD 24278)
/// - FUNCUBE-1 / AO-73 (NORAD 39444)
/// - LILACSAT-2 / CAS-3H (NORAD 40069)
pub fn obtener_tle_por_nombre(satellite_name: &str) -> io::Result<TleData> {
    let norad_id = match satellite_name.to_uppercase().as_str() {
        "ISS" => 25544,
        "AO-91" | "FOX-1B" | "RADFXSAT" => 43017, // AO-91 = FOX-1B = RADFXSAT
        "FO-29" | "JAS-2" => 24278,               // FO-29 = JAS-2 (satélite diferente)
        "FUNCUBE-1" | "AO-73" => 39444,
        "LILACSAT-2" | "CAS-3H" => 40069,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Satélite desconocido: {}", satellite_name),
            ));
        }
    };

    println!("Descargando TLE de {}...", satellite_name);

    match descargar_tle_celestrak(norad_id) {
        Ok(tle) => {
            println!("✓ TLE descargado");
            Ok(tle)
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            Err(e)
        }
    }
}
