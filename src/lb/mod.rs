mod weighted_lb;
use std::fmt::Display;

pub use weighted_lb::WeightedLoadBalancer;

use thiserror::Error;

use crate::stats::store::{FetchError};
use crate::config::resolver::{Base, Variant};
use crate::config::providers;

pub enum Choice<'b> {
    Base(&'b Base),
    Variant(&'b Variant)
}

impl<'a> Display for Choice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Choice::Base(base) => write!(f, "{base}"),
            Choice::Variant(variant) => write!(f, "{variant}"),
        }
    }
}

#[derive(Debug, Error)]
pub enum LBError {
    #[error("LBError:{0}")]
    Other(&'static str),
    #[error("ToModelInfoError: {0}")]
    ModelInfoError(#[from] providers::error::ToModelInfoError),
    #[error("FetchError: {0}")]
    FetchError(#[from] FetchError),
}

pub enum BalanceLevel {
    // Load balances over all usages of the same model (Model is the shared resource in the scope,
    // usage of another model under the same provider does not count) Will aggregate numbers of
    // Provider + Model
    Model,
    // Load balances over all models under the same provider. (Provider is the shared resource in
    // the scope)
    Provider,
    // Load balances over variant. (Variant is the shared resource in the scope, usage of
    // referenced provider/model outside the variant do not count) Will aggregate numbers of
    // Provider + Model + Variant
    Variant,
}

pub enum BalanceScope {
    Group, // Apply LB level over members of the group
    Global // Apply LB level over global usage
}
