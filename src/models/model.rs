use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::ser::SerializeMap;
use serde::de;

use super::ModelMessage;
use crate::input_modeling::UniformRNG;
use crate::utils::error::SimulationError;

/// `Model` wraps `model_type` and provides common ID functionality (a struct
/// field and associated accessor method).  The simulator requires all models
/// to have an ID.
#[derive(Clone)]
pub struct Model {
    id: String,
    inner: Box<dyn AsModel>,
}

impl Model {
    pub fn new(id: String, inner: Box<dyn AsModel>) -> Self {
        Self { id, inner }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

pub trait ModelClone {
    fn clone_box(&self) -> Box<dyn AsModel>;
}

impl<T> ModelClone for T
where
    T: 'static + AsModel + Clone,
{
    fn clone_box(&self) -> Box<dyn AsModel> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn AsModel> {
    fn clone(&self) -> Box<dyn AsModel> {
        self.clone_box()
    }
}

impl Serialize for Model {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let extra_fields: serde_yaml::Value = self.inner.serialize();
        let mut model = serializer.serialize_map(None)?;
        model.serialize_entry("id", &self.id)?;
        model.serialize_entry("type", self.inner.get_type())?;
        if let serde_yaml::Value::Mapping(map) = extra_fields {
            for (key, value) in map.iter() {
                model.serialize_entry(&key, &value)?;
            }
        }
        model.end()
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let model_repr = super::ModelRepr::deserialize(deserializer)?;
        const VARIANTS: &'static [&'static str] = &[
            &"Generator", &"ExclusiveGateway", &"Processor", &"Storage"
        ];
        match &model_repr.model_type[..] {
            "Generator" => {
                let generator = serde_yaml::from_value::<super::Generator>(model_repr.extra).map_err(de::Error::custom)?;
                Ok(Model::new(
                    model_repr.id,
                    Box::new(generator)
                ))
            },
            "ExclusiveGateway" => {
                let exclusive_gateway = serde_yaml::from_value::<super::ExclusiveGateway>(model_repr.extra).map_err(de::Error::custom)?;
                Ok(Model::new(
                    model_repr.id,
                    Box::new(exclusive_gateway)
                ))
            },
            "Processor" => {
                let processor = serde_yaml::from_value::<super::Processor>(model_repr.extra).map_err(de::Error::custom)?;
                Ok(Model::new(
                    model_repr.id,
                    Box::new(processor)
                ))
            },
            "Storage" => {
                let storage = serde_yaml::from_value::<super::Storage>(model_repr.extra).map_err(de::Error::custom)?;
                Ok(Model::new(
                    model_repr.id,
                    Box::new(storage)
                ))
            },
            other => {
                Err(de::Error::unknown_variant(other, VARIANTS))
            }
        }
    }
}

impl AsModel for Model {
    fn status(&self) -> String {
        self.inner.status()
    }

    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.events_ext(uniform_rng, incoming_message)
    }

    fn events_int(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.events_int(uniform_rng)
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.inner.time_advance(time_delta)
    }

    fn until_next_event(&self) -> f64 {
        self.inner.until_next_event()
    }
}

/// The `AsModel` trait defines everything required for a model to operate
/// within the discrete event simulation.  The simulator formalism (Discrete
/// Event System Specification) requires `events_ext`, `events_int`,
/// `time_advance`, and `until_next_event`.  The additional `status` is for
/// facilitation of simulation reasoning, reporting, and debugging.
// #[enum_dispatch]
pub trait AsModel: ModelClone {
    fn get_type(&self) -> &'static str {
        ""
    }
    fn serialize(&self) -> serde_yaml::Value {
        serde_yaml::Value::Null
    }
    fn status(&self) -> String;
    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError>;
    fn events_int(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError>;
    fn time_advance(&mut self, time_delta: f64);
    fn until_next_event(&self) -> f64;
}
