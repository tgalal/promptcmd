use std::{collections::HashMap};
use log::debug;

pub struct Model {
    pub usage: ModelUsage,
    pub attributes: SelectionAttributes
}

pub struct ModelUsage {
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_usages: u32,
    pub avg_tps: u32
}

pub struct SelectionAttributes {
    pub weight: u32
}

pub type SelectableModels = HashMap<String, Model>;

#[derive(Default)]
pub struct WeightedLoadBalancer;

impl WeightedLoadBalancer {
    pub fn select(&self, models: &SelectableModels) -> Option<String> {
        if models.is_empty() {
            debug!("No models to choose from");
            return None;
        }

        //calculate the sum of all weights and tokens
        let total_weight: u32 = models.values().map(|m| m.attributes.weight).sum();
        let total_tokens: u64 = models.values().map(|m | m.usage.prompt_tokens + m .usage.completion_tokens).sum();

        if total_weight == 0 {
            debug!("Total weight is zero, aborting");
            return None;
        }

        if total_tokens == 0 {
            // just return the one with the maximum weight
            return models
                .iter()
                .max_by_key(|(_, m)| m.attributes.weight)
                .map(|(id, _)| id.clone());
        }

        models
        .iter()
        .max_by(|(_, a), (_, b)| {
            let deficit_a = Self::calculate_deficit(a, total_weight, total_tokens);
            let deficit_b = Self::calculate_deficit(b, total_weight, total_tokens);
            deficit_a.partial_cmp(&deficit_b).unwrap()
        })
        .map(|(id, _)| id.clone())

    }

    fn calculate_deficit(model: &Model, total_weight: u32, total_tokens: u64) -> f64 {
        let target_ratio = model.attributes.weight as f64 / total_weight as f64;
        let actual_ratio = (model.usage.prompt_tokens + model.usage.completion_tokens) as f64
            / total_tokens as f64;
        target_ratio - actual_ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_model(provider: &str, model: &str, prompt_tokens: u64, completion_tokens: u64, weight: u32) -> Model {
        Model {
            usage: ModelUsage {
                provider: provider.to_string(),
                model: model.to_string(),
                prompt_tokens,
                completion_tokens,
                total_usages: 0,
                avg_tps: 0,
            },
            attributes: SelectionAttributes { weight },
        }
    }

    fn create_balancer() -> WeightedLoadBalancer {
        WeightedLoadBalancer {}
    }

    #[test]
    fn test_empty_models() {
        let balancer = create_balancer();
        let models: SelectableModels = HashMap::new();
        assert_eq!(balancer.select(&models), None);
    }

    #[test]
    fn test_single_model() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));

        assert_eq!(balancer.select(&models), Some("gpt-4".to_string()));
    }

    #[test]
    fn test_zero_tokens_selects_highest_weight() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 0, 0, 3));
        models.insert("gemini".to_string(), create_model("google", "gemini", 0, 0, 2));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_all_zero_weights() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 50, 0));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 50, 0));

        assert_eq!(balancer.select(&models), None);
    }

    #[test]
    fn test_equal_weights_selects_underutilized() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 100, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 50, 50, 1));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_weighted_selection_underweight_model() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 50, 50, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 50, 50, 2));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_weighted_selection_already_balanced() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 50, 50, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 100, 2));

        let result = balancer.select(&models);
        assert!(result.is_some());
    }

    #[test]
    fn test_heavily_overused_model() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 800, 100, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 50, 50, 1));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_three_models_complex_weights() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 100, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 100, 2));
        models.insert("gemini".to_string(), create_model("google", "gemini", 100, 100, 3));

        assert_eq!(balancer.select(&models), Some("gemini".to_string()));
    }

    #[test]
    fn test_only_prompt_tokens() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 200, 0, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 0, 1));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_only_completion_tokens() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 200, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 0, 100, 1));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_large_token_counts() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 1_000_000_000, 500_000_000, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 500_000_000, 250_000_000, 1));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_one_model_zero_weight() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 100, 100, 0));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 100, 100, 1));

        assert_eq!(balancer.select(&models), Some("claude".to_string()));
    }

    #[test]
    fn test_convergence_simulation() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("model_a".to_string(), create_model("provider", "a", 0, 0, 1));
        models.insert("model_b".to_string(), create_model("provider", "b", 0, 0, 2));

        let mut a_tokens: u64 = 0;
        let mut b_tokens: u64 = 0;
        let tokens_per_request: u64 = 100;

        for _ in 0..100 {
            let selected = balancer.select(&models).unwrap();

            if selected == "model_a" {
                a_tokens += tokens_per_request;
                models.get_mut("model_a").unwrap().usage.prompt_tokens = a_tokens;
            } else {
                b_tokens += tokens_per_request;
                models.get_mut("model_b").unwrap().usage.prompt_tokens = b_tokens;
            }
        }

        let ratio = b_tokens as f64 / a_tokens as f64;
        assert!(ratio > 1.8 && ratio < 2.2, "Expected ratio ~2.0, got {}", ratio);
    }

    #[test]
    fn test_balancer_reusable() {
        let balancer = create_balancer();

        let mut models1 = HashMap::new();
        models1.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
        assert_eq!(balancer.select(&models1), Some("gpt-4".to_string()));

        let mut models2 = HashMap::new();
        models2.insert("claude".to_string(), create_model("anthropic", "claude", 0, 0, 1));
        assert_eq!(balancer.select(&models2), Some("claude".to_string()));
    }

    #[test]
    fn test_same_balancer_different_states() {
        let balancer = create_balancer();
        let mut models = HashMap::new();
        models.insert("gpt-4".to_string(), create_model("openai", "gpt-4", 0, 0, 1));
        models.insert("claude".to_string(), create_model("anthropic", "claude", 0, 0, 1));

        // First selection with no tokens
        let first = balancer.select(&models);
        assert!(first.is_some());

        // Update tokens and select again
        models.get_mut("gpt-4").unwrap().usage.prompt_tokens = 1000;
        let second = balancer.select(&models);
        assert_eq!(second, Some("claude".to_string()));
    }
}
