//! Dataclass for storing the different cable types

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::f64::consts::PI;

use crate::materials::{Material, MATERIALS_DB};

/// Layer struct
#[derive(Debug, Clone)]
pub struct Layer{
    ///Material type
    pub material: Material,
    /// Diameter
    pub diameter: f64,
}

impl Layer{
    /// Initialize new layer
    pub fn new(material:Material, diameter: f64) -> Self{
        Self { material, diameter}
    }
}

#[derive(Debug, Clone)]
/// Cable struct
pub struct Cable{
    /// Cable Reference
    pub cable_ref: String,
    /// Outer diameter
    pub outer: Layer,
    /// Dielectric type
    pub dielectric: Layer,
    /// Inner diameter
    pub inner: Layer,
}

impl Cable{
    /// Initialize new cable
    pub fn new(cable_ref:String, outer:Layer, dielectric: Layer, inner: Layer) -> Self{
        Self { cable_ref, outer, dielectric,inner }
    }
    /// Calculate outer area 
    pub fn outer_area(&self) -> f64 {
        let r1 = self.outer.diameter / 2.0;
        let r2 = self.dielectric.diameter / 2.0;
        PI * (r1 * r1 - r2 * r2)
    }
    /// Calculate dielectric area 
    pub fn dielectric_area(&self) -> f64 {
        let r1 = self.dielectric.diameter / 2.0;
        let r2 = self.inner.diameter / 2.0;
        PI * (r1 * r1 - r2 * r2)
    }
    /// Calculate inner area 
    pub fn inner_area(&self) -> f64 {
        let r = self.inner.diameter / 2.0;
        PI * r * r
    }
}

// ----- DB type -----

/// Cables database
#[derive(Debug, Clone)]
pub struct CablesDB {
    /// Dictionary
    pub db: HashMap<String, Cable>,
}

impl CablesDB {
    /// Initialization function
    pub fn new(cables: Vec<Cable>) -> Self {
        let mut db = HashMap::new();
        for cable in cables {
            db.insert(cable.cable_ref.clone(), cable);
        }
        Self { db }
    }
    /// Getter function
    pub fn get(&self, id: &str) -> Option<&Cable> {
        self.db.get(id)
    }
}

// ----- Helper to fetch & clone a Material from the materials DB -----

fn mat(id: &str) -> Material {
    MATERIALS_DB
        .get(id)
        .unwrap_or_else(|| panic!("Unknown material id: {id}"))
        .clone()
}

// ----- Prebuilt cables (global, like your Python module-level objects) -----
/// Static cable database
pub static CABLES_DB: Lazy<CablesDB> = Lazy::new(|| {
    let radiall_nbti_cable = Cable::new(
        "Radiall NbTi".to_string(),
        Layer::new(mat("NbTi"), 0.9e-3),
        Layer::new(mat("PTFE"), 0.66e-3),
        Layer::new(mat("NbTi"), 0.203e-3),
    );

    let sc_119_50_scn_cn = Cable::new(
        "SC-119/50-SCN-CN".to_string(),
        Layer::new(mat("SCN"), 1.19e-3),
        Layer::new(mat("PTFE"), 0.94e-3),
        Layer::new(mat("CN"), 0.287e-3),
    );

    let sc_86_50_scn_cn = Cable::new(
        "SC-86/50-SCN-CN".to_string(),
        Layer::new(mat("SCN"), 0.86e-3),
        Layer::new(mat("PTFE"), 0.66e-3),
        Layer::new(mat("CN"), 0.203e-3),
    );

    let sc_86_50_nbti_nbti = Cable::new(
        "SC-86/50-NbTi-NbTi".to_string(),
        Layer::new(mat("NbTi"), 0.90e-3),
        Layer::new(mat("PTFE"), 0.66e-3),
        Layer::new(mat("NbTi"), 0.203e-3),
    );

    let phbr_36awg = Cable::new(
        "PhBr_36AWG".to_string(),
        Layer::new(mat("PhBr"), 0.127e-3),
        Layer::new(mat("PTFE"), 0.127e-3),
        Layer::new(mat("PhBr"), 0.127e-3),
    );

    let nbti_36awg = Cable::new(
        "NbTi_36AWG".to_string(),
        Layer::new(mat("NbTi"), 0.127e-3),
        Layer::new(mat("NbTi"), 0.127e-3),
        Layer::new(mat("NbTi"), 0.127e-3),
    );

    CablesDB::new(vec![
        radiall_nbti_cable,
        sc_119_50_scn_cn,
        sc_86_50_scn_cn,
        sc_86_50_nbti_nbti,
        phbr_36awg,
        nbti_36awg,
    ])
});