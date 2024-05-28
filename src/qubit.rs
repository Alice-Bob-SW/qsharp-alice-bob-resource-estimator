// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//!
//! Cat qubits are characterized by:
//! <pre>
//! - Their average number of photons |α|²
//! - The physical error rate κ₁/κ₂
//! </pre>
//! Here, κ₁/κ₂=10e-5 is assumed, and |α|² is a parameter to be optimized.

/// Basic struct for cat qubits
pub struct CatQubit {
    // The physical error rate is computed as κ₁/κ₂, the ratio between the one and
    // two photon loss rates
    pub(crate) k1_k2: f64,
}

impl Default for CatQubit {
    /// Set κ₁/κ₂ to a default value
    fn default() -> Self {
        // By default, we assume k1_k2 of 1e-5, arXiv:2302.06639 (p. 2).
        Self { k1_k2: 1e-5 }
    }
}

impl CatQubit {
    /// Default instanciation
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
