mod asc_abi;
mod to_from;

/// Public interface of the crate, receives triggers to be processed.
mod host;
pub use host::RuntimeHostBuilder;

/// Pre-processes modules and manages their threads. Serves as an interface from `host` to `module`.
mod mapping;

/// Deals with wasmi.
mod module;

/// Runtime-agnostic implementation of exports to WASM.
mod host_exports;

use graph::prelude::web3::types::Address;
use graph::prelude::{Store, SubgraphDeploymentStore};

#[derive(Clone, Debug)]
pub(crate) struct UnresolvedContractCall {
    pub contract_name: String,
    pub contract_address: Address,
    pub function_name: String,
    pub function_args: Vec<ethabi::Token>,
}

trait RuntimeStore: Store + SubgraphDeploymentStore {}
impl<S: Store + SubgraphDeploymentStore> RuntimeStore for S {}
