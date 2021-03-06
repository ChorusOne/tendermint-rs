use crate::{components::io::IoError, types::PeerId};

use tendermint::{abci::transaction::Hash, evidence::Evidence};
use tendermint_rpc as rpc;

use contracts::{contract_trait, pre};
use std::collections::HashMap;

/// Interface for reporting evidence to full nodes, typically via the RPC client.
#[contract_trait]
pub trait EvidenceReporter: Send {
    /// Report evidence to all connected full nodes.
    fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError>;
}

/// Production implementation of the EvidenceReporter component, which reports evidence to full
/// nodes via RPC.
#[derive(Clone, Debug)]
pub struct ProdEvidenceReporter {
    peer_map: HashMap<PeerId, tendermint::net::Address>,
}

#[contract_trait]
impl EvidenceReporter for ProdEvidenceReporter {
    #[pre(self.peer_map.contains_key(&peer))]
    fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError> {
        let res = block_on(self.rpc_client_for(peer).broadcast_evidence(e));

        match res {
            Ok(response) => Ok(response.hash),
            Err(err) => Err(IoError::IoError(err)),
        }
    }
}

impl ProdEvidenceReporter {
    /// Constructs a new ProdEvidenceReporter component.
    ///
    /// A peer map which maps peer IDS to their network address must be supplied.
    pub fn new(peer_map: HashMap<PeerId, tendermint::net::Address>) -> Self {
        Self { peer_map }
    }

    // FIXME: Cannot enable precondition because of "autoref lifetime" issue
    // #[pre(self.peer_map.contains_key(&peer))]
    fn rpc_client_for(&self, peer: PeerId) -> rpc::Client {
        let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
        rpc::Client::new(peer_addr)
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
