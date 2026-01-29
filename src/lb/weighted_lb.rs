use crate::stats::store::{
    StatsStore, SummaryItem
};

use crate::config::resolver::{
    Group, GroupMember
};

use crate::config::providers::{
    ModelInfo
};

use super::{BalanceScope, BalanceLevel, Choice, LBError};

pub struct WeightedLoadBalancer {
    pub stats: &'static dyn StatsStore
}

impl WeightedLoadBalancer {
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

