use serde::{Deserialize, Serialize};
pub use sgp4::Elements;
use std::time::Duration;
pub mod frequencies;
pub mod tle_loader;
pub mod validaciones;
use chrono::{DateTime, Utc};
use predict_rs::{
    consts::{DEG_TO_RAD, RAD_TO_DEG},
    observer::{self, get_passes},
    orbit,
    predict::{ObserverElements, Passes, PredictObserver},
};

pub type Degrees = f64;
pub type Meters = f64;

/// Velocidad de la luz en metros por segundo
const SPEED_OF_LIGHT: f64 = 299_792_458.0;

#[derive(Clone, Copy, Debug)]
pub struct Pass {
    pub start: f64,
    pub end: f64,
}

/// The observer is the location of the ground station.
#[derive(Serialize, Deserialize, Clone)]
pub struct Observer {
    /// Ground station latitude, in degrees.
    latitude: Degrees,
    /// Ground station longitude, in degrees.
    longitude: Degrees,
    /// Ground station altitude, in meters.
    altitude: Meters,
}

impl Observer {
    pub fn new(latitude: Degrees, longitude: Degrees, altitude: Meters) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }
}

/// The predicted observation.
#[derive(Debug)]
pub struct Observation {
    /// Azimuth, in degrees.
    pub azimuth: Degrees,
    /// Elevation, in degrees.
    pub elevation: Degrees,
    /// Range rate, in meters per second.
    pub range_rate: f64,
}

#[derive(Debug)]
pub enum TrackerError {
    ElementsError(sgp4::ElementsError),
    OrbitPredictionError(orbit::OrbitPredictionError),
}

/// The tracker is used to predict the position of a satellite, given its orbital elements, relative to the ground station.
pub struct Tracker {
    observer: PredictObserver,
    elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Tracker {
    /// Create a new tracker given the observer and satellite's orbital elements.
    ///
    /// # Arguments
    /// * `observer` - The ground station location
    /// * `elements` - The satellite's TLE orbital elements
    pub fn new(observer: &Observer, elements: sgp4::Elements) -> Result<Self, TrackerError> {
        let constants =
            sgp4::Constants::from_elements(&elements).map_err(TrackerError::ElementsError)?;

        let observer = PredictObserver {
            name: "".to_string(),
            latitude: observer.latitude * DEG_TO_RAD,
            longitude: observer.longitude * DEG_TO_RAD,
            altitude: observer.altitude,
            min_elevation: 0.0,
        };

        Ok(Self {
            observer,
            elements,
            constants,
        })
    }

    /// Predict the observation of the satellite at a given time.
    ///
    /// # Arguments
    /// * `at` - The time at which to predict the observation
    ///
    /// # Returns
    /// An `Observation` with azimuth, elevation, and range rate.
    pub fn track(&self, at: DateTime<Utc>) -> Result<Observation, TrackerError> {
        let orbit = orbit::predict_orbit(&self.elements, &self.constants, at.timestamp() as f64)
            .map_err(TrackerError::OrbitPredictionError)?;

        let observation = observer::predict_observe_orbit(&self.observer, &orbit);

        Ok(Observation {
            azimuth: observation.azimuth * RAD_TO_DEG,
            elevation: observation.elevation * RAD_TO_DEG,
            range_rate: observation.range_rate * 1000.0, // Convert km/s to m/s
        })
    }

    /// Predict the next pass of the satellite over the ground station, starting from a given time and within a specified time window.
    pub fn next_pass(&self, from: DateTime<Utc>, window: Duration) -> Option<Pass> {
        let oe = ObserverElements {
            observer: &self.observer,
            elements: &self.elements,
            constants: &self.constants,
        };

        let start_utc = from.timestamp() as u64;
        let stop_utc = start_utc + window.as_secs();

        let passes = get_passes(&oe, start_utc as f64, stop_utc as f64).ok()?;

        let pass = passes.passes.into_iter().next().unwrap();

        Some(Pass {
            start: pass.aos.unwrap().time,
            end: pass.los.unwrap().time,
        })
    }

    /// Predict all passes of the satellite over the ground station, starting from a given time and within a specified time window.
    pub fn next_passes(&self, from: DateTime<Utc>, window: Duration) -> Option<Passes> {
        let oe = ObserverElements {
            observer: &self.observer,
            elements: &self.elements,
            constants: &self.constants,
        };

        let start_utc = from.timestamp() as u64;
        let stop_utc = start_utc + window.as_secs();

        let passes = get_passes(&oe, start_utc as f64, stop_utc as f64).ok()?;

        Some(passes)
    }
}

/// Calcula la frecuencia de downlink (recepción en estación terrena)
///
/// # Arguments
/// * `freq_tx_sat` - Frecuencia de transmisión del satélite en Hz
/// * `range_rate` - Velocidad radial en m/s (positivo = alejándose, negativo = acercándose)
///
/// # Returns
/// Frecuencia que debe sintonizar el receptor en Hz
pub fn doppler_downlink(freq_tx_sat: f64, range_rate: f64) -> f64 {
    let doppler_shift = -freq_tx_sat * (range_rate / SPEED_OF_LIGHT);
    freq_tx_sat + doppler_shift
}

/// Calcula la frecuencia de uplink (transmisión desde estación terrena)
///
/// # Arguments
/// * `freq_rx_sat` - Frecuencia de recepción del satélite en Hz
/// * `range_rate` - Velocidad radial en m/s (positivo = alejándose, negativo = acercándose)
///
/// # Returns
/// Frecuencia a la que debe transmitir la estación terrena en Hz
pub fn doppler_uplink(freq_rx_sat: f64, range_rate: f64) -> f64 {
    // Para uplink, necesitamos pre-compensar el Doppler
    // Si el satélite se aleja (range_rate > 0), debemos transmitir a mayor frecuencia
    // Si el satélite se acerca (range_rate < 0), debemos transmitir a menor frecuencia
    let doppler_shift = freq_rx_sat * (range_rate / SPEED_OF_LIGHT);
    freq_rx_sat + doppler_shift
}
