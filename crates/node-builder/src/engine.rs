use reth_node_api::EngineTypes;
use reth_payload_builder::{
    EthBuiltPayload, EthPayloadBuilderAttributes, OptimismPayloadBuilderAttributes,
};
use reth_rpc_types::engine::{OptimismPayloadAttributes, PayloadAttributes};

/// The types used in the default mainnet ethereum beacon consensus engine.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct EthEngineTypes;

impl EngineTypes for EthEngineTypes {
    type PayloadAttributes = PayloadAttributes;
    type PayloadBuilderAttributes = EthPayloadBuilderAttributes;
    type BuiltPayload = EthBuiltPayload;
}

/// The types used in the optimism beacon consensus engine.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct OptimismEngineTypes;

impl EngineTypes for OptimismEngineTypes {
    type PayloadAttributes = OptimismPayloadAttributes;
    type PayloadBuilderAttributes = OptimismPayloadBuilderAttributes;
    type BuiltPayload = EthBuiltPayload;
}
