use banderwagon::{msm::MSMPrecompWnaf, Element, Fr, Zero};
use once_cell::sync::Lazy;

use crate::{constants::VERKLE_NODE_WIDTH, crs::CRS};

pub static DEFAULT_COMMITER: Lazy<Committer> = Lazy::new(Committer::new);

pub struct Committer {
    precomp: MSMPrecompWnaf,
}

impl Committer {
    fn new() -> Self {
        Self {
            precomp: MSMPrecompWnaf::new(CRS.as_slice(), 12),
        }
    }

    // Commit to a lagrange polynomial, evaluations.len() must equal the size of the CRS at the moment
    pub fn commit_lagrange(&self, evaluations: &[Fr]) -> Element {
        // Preliminary benchmarks indicate that the parallel version is faster
        // for vectors of length 64 or more
        if evaluations.len() >= 64 {
            self.precomp.mul_par(evaluations)
        } else {
            self.precomp.mul(evaluations)
        }
    }

    pub fn scalar_mul(&self, index: usize, value: Fr) -> Element {
        self.precomp.mul_index(value, index)
    }

    pub fn commit_sparse(&self, evaluations: Vec<(usize, Fr)>) -> Element {
        // TODO consider if 64 is good value
        if evaluations.len() >= 64 {
            let mut dense = [Fr::zero(); VERKLE_NODE_WIDTH];
            for (index, value) in evaluations {
                dense[index] = value;
            }
            self.commit_lagrange(&dense)
        } else {
            let mut result = Element::zero();
            for (index, value) in evaluations {
                result += self.scalar_mul(index, value)
            }
            result
        }
    }
}
