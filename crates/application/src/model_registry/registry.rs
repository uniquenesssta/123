use football_model_api::{ModelDescriptor, PredictionModel};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ModelRegistry {
    models: HashMap<String, Arc<dyn PredictionModel>>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn register(&mut self, model: Arc<dyn PredictionModel>) {
        self.models
            .insert(model.descriptor().model_id.clone(), model);
    }

    pub fn get(&self, model_id: &str) -> Option<Arc<dyn PredictionModel>> {
        self.models.get(model_id).cloned()
    }

    pub fn descriptors(&self) -> Vec<ModelDescriptor> {
        let mut values: Vec<ModelDescriptor> = self
            .models
            .values()
            .map(|model| model.descriptor())
            .collect();
        values.sort_by(|left, right| left.model_id.cmp(&right.model_id));
        values
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
