use banderwagon::{Element, Fr};

use super::node::NodeTrait;

pub struct CommitmentNode {
    commitment: Element,
    commitment_hash: Fr,
}

impl CommitmentNode {
    pub fn new(commitment: Element) -> Self {
        Self {
            commitment,
            commitment_hash: commitment.map_to_scalar_field(),
        }
    }
}

impl NodeTrait for CommitmentNode {
    fn commitment_write(&mut self) -> Element {
        self.commitment
    }

    fn commitment(&self) -> Element {
        self.commitment
    }

    fn commitment_hash_write(&mut self) -> Fr {
        self.commitment_hash
    }

    fn commitment_hash(&self) -> Fr {
        self.commitment_hash
    }
}
