use thiserror::Error;

use crate::stats::store::{
    FetchError, StatsStore, SummaryItem
};

use crate::config::resolver::{
    Base, Variant, Group, GroupMember
};

use crate::config::providers::{
    self, ModelInfo
};

pub struct WeightedLoadBalancer<'a> {
    pub stats: &'a dyn StatsStore
}

pub enum Choice<'b> {
    Base(&'b Base),
    Variant(&'b Variant)
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
    // Load balances over all usages of the same model (Model is the shared resource, usage of
    // another model under the same provider does not count)
    // Will aggregate numbers of Provider + Model
    Model,

    // Load balances over all models under the same provider. (Provider is the shared resource)
    Provider,
    // Will aggregate numbers of Provider

    // Load balances over variant. (Variant is the shared resource, usage of referenced
    // provider/model outside the variant do not count)
    // Will aggregate numbers of Provider + Model + Variant
    Variant,
}

pub enum BalanceScope {
    Group, // Apply LB level over members of the group
    Global // Apply LB level over global usage
}

impl<'a> WeightedLoadBalancer<'a> {
    pub fn choose<'b>(
        &self,
        group: &'b Group,
        scope: BalanceScope,
        level: BalanceLevel,
    ) -> Result<Choice<'b>, LBError> {


        let model_infos: Vec<ModelInfo> = group.members.iter().map(|member| {
            match member {
                GroupMember::Base(base, _) => base.model_info.clone(),
                GroupMember::Variant(variant, _) => variant.model_info.clone()
            }
        }).collect::<Result<Vec<_>, _>>()?;

        let summaries: Vec<SummaryItem> = group.members.iter().zip(model_infos.iter()).map(
            |(member, model_info)| {
                let group_filter = if let BalanceScope::Group = scope {
                    // Only aggregate numbers within usage in the group
                    Some(group.name.clone())
                } else {
                    None
                };

                // LB will be at least on the LLM provider level
                // i.e., stats aggregate invocations under different models of the provider
                let provider_filter = Some(model_info.provider.clone());

                // LB on provider + model
                let model_filter = if matches!(level, BalanceLevel::Model | BalanceLevel::Variant) {
                    Some(model_info.model.clone())
                } else {
                    None
                };
                // LB on provider + model + variant
                let variant_filter = if matches!(level, BalanceLevel::Variant) {
                    if let GroupMember::Variant(variant, _) = member  {
                        Some(variant.name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // self.stats.summary(provider_filter, model_filter, variant_filter, group_filter, Some(true))
                let summaries = self.stats.summary(provider_filter, model_filter, variant_filter, group_filter, Some(true));
                match summaries {
                    Ok(mut summaries) => {
                        if summaries.is_empty() {
                            Ok(
                                SummaryItem {
                                    provider: model_info.provider.clone(),
                                    model: model_info.model.clone(),
                                    count: 0,
                                    prompt_tokens: 0,
                                    completion_tokens: 0,
                                    tps: 0
                                }
                            )
                        } else if summaries.len() > 1 {
                            Err(LBError::Other("Unexpected size of summaries"))
                        } else {
                            Ok(summaries.remove(0))
                        }
                    },
                    Err(err) => {
                        Err(LBError::FetchError(err))
                    }
                }
            }
        ).collect::<Result<Vec<_>, _>>()?;


        //calculate the sum of all weights and tokens
        // let total_weight: u32 = models.values().map(|m| m.attributes.weight).sum();
        let total_weight: u32 = group.members.iter().map(|member| member.weight()).sum();
        let total_tokens: u64 = summaries.iter().map(|item| item.prompt_tokens + item.completion_tokens).sum::<u32>() as u64;

        let members_with_summaries = group.members.iter().zip(summaries.iter());

        let result = if total_tokens == 0 {
            // just return the one with the maximum weight
            members_with_summaries
                .max_by_key(|m| m.0.weight())
                .ok_or(LBError::Other("AAA"))?
                .0
        } else {
            members_with_summaries
            .max_by(|a, b| {
                let deficit_a = Self::calculate_deficit(a.0, a.1, total_weight, total_tokens);
                let deficit_b = Self::calculate_deficit(b.0, b.1, total_weight, total_tokens);
                deficit_a.partial_cmp(&deficit_b).unwrap()
            }).ok_or(LBError::Other("AAA"))?
            .0
        };

        let choice =  match result {
            GroupMember::Base(base, _) => {
                Choice::Base(base)
            },
            GroupMember::Variant(base, _) => {
                Choice::Variant(base)
            },
        };

        Ok(choice)

    }

    fn calculate_deficit(member: &GroupMember, summary: &SummaryItem, total_weight: u32, total_tokens: u64) -> f64 {
        let target_ratio = member.weight() as f64 / total_weight as f64;
        let actual_ratio = (summary.prompt_tokens + summary.completion_tokens) as f64
            / total_tokens as f64;
        target_ratio - actual_ratio
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn create_model(provider: &str, model: &str, prompt_tokens: u64, completion_tokens: u64, weight: u32) -> Model {
//         Model {
//             usage: ModelUsage {
//                 provider: provider.to_string(),
//                 model: model.to_string(),
//                 prompt_tokens,
//                 completion_tokens,
//                 total_usages: 0,
//                 avg_tps: 0,
//             },
//             attributes: SelectionAttributes { weight },
//         }
//     }
//
//     fn create_balancer() -> WeightedLoadBalancer {
//         WeightedLoadBalancer {}
//     }
//
//     #[test]
//     fn test_empty_models() {
//         let balancer = create_balancer();
//         let models: SelectableModels = HashMap::new();
//         assert_eq!(balancer.select(&models), None);
//     }
//
//     #[test]
//     fn test_single_model() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
//
//         assert_eq!(balancer.select(&models), Some("gpt-4".to_string()));
//     }
//
//     #[test]
//     fn test_zero_tokens_selects_highest_weight() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 0, 0, 3));
//         models.insert("gemini".to_string(), create_model("google", "gemini", 0, 0, 2));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_all_zero_weights() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 50, 0));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 50, 0));
//
//         assert_eq!(balancer.select(&models), None);
//     }
//
//     #[test]
//     fn test_equal_weights_selects_underutilized() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 100, 1));
//  /        models.insert("claude".to_string(), create_model("anthropic", "claude", 50, 50, 1));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_weighted_selection_underweight_model() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 50, 50, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 50, 50, 2));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_weighted_selection_already_balanced() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 50, 50, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 100, 2));
//
//         let result = balancer.select(&models);
//         assert!(result.is_some());
//     }
//
//     #[test]
//     fn test_heavily_overused_model() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 800, 100, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 50, 50, 1));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_three_models_complex_weights() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 100, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 100, 2));
//         models.insert("gemini".to_string(), create_model("google", "gemini", 100, 100, 3));
//
//         assert_eq!(balancer.select(&models), Some("gemini".to_string()));
//     }
//
//     #[test]
//     fn test_only_prompt_tokens() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 200, 0, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 0, 1));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_only_completion_tokens() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 200, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 0, 100, 1));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_large_token_counts() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 1_000_000_000, 500_000_000, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 500_000_000, 250_000_000, 1));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_one_model_zero_weight() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 100, 0));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 100, 1));
//
//         assert_eq!(balancer.select(&models), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_convergence_simulation() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("model_a".to_string(), create_model("provider", "a", 0, 0, 1));
//         models.insert("model_b".to_string(), create_model("provider", "b", 0, 0, 2));
//
//         let mut a_tokens: u64 = 0;
//         let mut b_tokens: u64 = 0;
//         let tokens_per_request: u64 = 100;
//
//         for _ in 0..100 {
//             let selected = balancer.select(&models).unwrap();
//
//             if selected == "model_a" {
//                 a_tokens += tokens_per_request;
//                 models.get_mut("model_a").unwrap().usage.prompt_tokens = a_tokens;
//             } else {
//                 b_tokens += tokens_per_request;
//                 models.get_mut("model_b").unwrap().usage.prompt_tokens = b_tokens;
//             }
//         }
//
//         let ratio = b_tokens as f64 / a_tokens as f64;
//         assert!(ratio > 1.8 && ratio < 2.2, "Expected ratio ~2.0, got {}", ratio);
//     }
//
//     #[test]
//     fn test_balancer_reusable() {
//         let balancer = create_balancer();
//
//         let mut models1 = HashMap::new();
//         models1.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
//         assert_eq!(balancer.select(&models1), Some("gpt-4".to_string()));
//
//         let mut models2 = HashMap::new();
//         models2.insert("claude".to_string(), create_model("anthropic", "claude", 0, 0, 1));
//         assert_eq!(balancer.select(&models2), Some("claude".to_string()));
//     }
//
//     #[test]
//     fn test_same_balancer_different_states() {
//         let balancer = create_balancer();
//         let mut models = HashMap::new();
//         models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
//         models.insert("claude".to_string(), create_model("anthropic", "claude", 0, 0, 1));
//
//         // First selection with no tokens
//         let first = balancer.select(&models);
//         assert!(first.is_some());
//
//         // Update tokens and select again
//         models.get_mut("gpt-4").unwrap().usage.prompt_tokens = 1000;
//         let second = balancer.select(&models);
//         assert_eq!(second, Some("claude".to_string()));
//     }
// }
