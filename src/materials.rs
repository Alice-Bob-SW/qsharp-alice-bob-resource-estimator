//! Dataclass for storing the different cable materials used in the signal lines


use std::collections::{HashSet, HashMap};
use once_cell::sync::Lazy;

static VALIDITY_RANGE_4K: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["Cu_RRR_100", "PTFE"].into_iter().collect()
});


static VALIDITY_RANGE_2K: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["NbTi", "CN", "SCN", "BeCu", "PhBr"].into_iter().collect()
});


///Material class for the energy estimation
#[derive(Debug, Clone)]
pub struct Material{
    material_id: String,
    fit_params: Vec<f64>,
}
impl Material{
    /// Initialization function
    pub fn new<S: Into<String>>(material_id: S, fit_params: Vec<f64>) -> Self {
        Self{
            material_id: material_id.into(), fit_params
        }
    }
    /// Evaluate the thermal conductivity k(T).
    /// Returns None if T <= 0.0 or inputs are inconsistent.
    pub fn k(&self,t:f64)->Option<f64>{
        if t <= 0.0 {
            return None;
        }
        if t < 4.0 && VALIDITY_RANGE_4K.contains(self.material_id.as_str()) {
            return self.k(4.0).map(|k4| t * k4 / 4.0); // |k4| checks that it is of type Some() and only then proceeds with the calculation
        }
        else if t < 2.0 && VALIDITY_RANGE_2K.contains(self.material_id.as_str()) {
            // Uses k(2 K) from the same material
            return self.k(2.0).map(|k2| t * k2 / 2.0);
        }
        if self.material_id == "Cu_RRR_100" {
            if self.fit_params.len() < 9 {
                return None;
            }
            let fp = &self.fit_params;
            let t_sqrt = t.powf(0.5);
            let t_1p5  = t.powf(1.5);
            let t2     = t * t;
            let num = fp[0]
                + fp[2] * t_sqrt
                + fp[4] * t
                + fp[6] * t_1p5
                + fp[8] * t2;

            let denom = 1.0
                + fp[1] * t_sqrt
                + fp[3] * t
                + fp[5] * t_1p5
                + fp[7] * t2;

            let val = 10f64.powf(num / denom);
            return Some(val);        
        }
        let x = t.log10();
        let p_of_x = poly_eval_descending(&self.fit_params, x);
        Some(10f64.powf(p_of_x))    
    }

}

fn poly_eval_descending(coeffs: &[f64], x: f64) -> f64 {
    if coeffs.is_empty() {
        return 0.0;
    }
    let mut acc = coeffs[0];
    for c in &coeffs[1..] {
        acc = acc * x + *c;
    }
    acc
}

/// Materials class
#[derive(Debug, Clone)]
pub struct MaterialsDB {
    /// Materials class
    pub db: HashMap<String, Material>,
}

impl MaterialsDB{
    /// Instantiate new material type
    pub fn new(materials: Vec<Material>) -> Self {
        let mut db = HashMap::new();
        for material in materials {
            db.insert(material.material_id.clone(), material);
        }
        Self {db}
    }
    /// Getter Method
    pub fn get(&self, id: &str) -> Option<&Material> {
        self.db.get(id)
    }
}
/// Final materials database
pub static MATERIALS_DB: Lazy<MaterialsDB> = Lazy::new(|| {
    let nbti = Material::new(
        "NbTi",
        vec![
            0.02612193, -0.18559454, 0.52131209, -1.01746415, 2.15843516,
            -3.4815713, 2.63612161, 0.67880346, -1.58508579,
        ],
    );

    let cn = Material::new(
        "CN",
        vec![-0.527, 5.985, -28.749, 76.215, -121.477, 117.69, -66.114, 20.499, -3.198],
    );

    let ptfe = Material::new(
        "PTFE",
        vec![0.33829, -4.3135, 23.32, -69.556, 124.69, -136.99, 89.43, -30.677, 2.738],
    );

    let scn = Material::new(
        "SCN",
        vec![-0.025, 1.322, -11.825, 46.383, -96.844, 113.586, -74.184, 25.845, -2.750],
    );

    let becu = Material::new(
        "BeCu",
        vec![0.0, -0.10501, 0.68722, -1.6145, 1.2788, 0.71218, -1.6954, 1.9319, -0.50015],
    );

    let phbr = Material::new(
        "PhBr",
        vec![
            -0.01754878, 0.0701625, 0.05263581, -0.35246336, -0.50597204,
            2.50133348, -2.76954821, 2.34173576, -0.65757732,
        ],
    );

    let cu_rrr_100 = Material::new(
        "Cu_RRR_100",
        vec![2.2154, -0.47461, -0.88068, 0.13871, 0.29505, -0.02043, -0.04831, 0.001281, 0.003207],
    );

    // Return exactly one DB as the last expression (no semicolon):
    MaterialsDB::new(vec![nbti, cn, ptfe, scn, becu, phbr, cu_rrr_100])
});







