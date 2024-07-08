use std::error::Error;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Endpoint {
    pub model: String,
    pub endpoint_name: String,
    pub target_model: Option<String>,
    pub inference_component: Option<String>,
    pub backend: String,
}

#[derive(Deserialize, Debug)]
pub struct ModelEndpoints {
    pub models: Vec<Endpoint>,
}

#[derive(Debug)]
pub struct EndpointLoader {
    endpoints: ModelEndpoints,
}


impl EndpointLoader {
    pub fn load<P: AsRef<Path>>(config_file: P) -> Result<EndpointLoader> {
        let config = fs::read_to_string(config_file)?;
        let endpoints: ModelEndpoints = serde_yaml::from_str(config.as_str())?;

        Ok(EndpointLoader {
            endpoints,
        })
    }

    pub fn get_endpoint<S: AsRef<str>>(&self, model: S) -> Option<&Endpoint> {
        self.endpoints.models.iter().find(|x| x.model.as_str() == model.as_ref())
    }
}


#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::EndpointLoader;

    #[test]
    fn test_load_endpoints() -> Result<()> {
        let temp = TempDir::new()?;
        let config_path = temp.path().join("config.yaml");
        fs::write(config_path.as_path(), r"models:
  - model: Llama-3-70B-instruct
    endpoint_name: lmi-llama-3-70B-Instruct
    backend: LMI
  - model: Phi-3-mini-4k-instruct
    endpoint_name: lmi-mme-20240627093303
    target_model: phi-3-mini-4k-instruct.tar.gz
    backend: LMI
  - model: Phi-3-medium-4k-instruct
    endpoint_name: lmi-mme-20240627093303
    target_model: phi-3-mini-4k-instruct.tar.gz
    backend: LMI
")?;
        let endpoints = EndpointLoader::load(config_path.as_path())?;
        assert_eq!(endpoints.get_endpoint("Llama-3-70B-instruct").unwrap().endpoint_name, "lmi-llama-3-70B-Instruct");
        assert_eq!(endpoints.get_endpoint("Phi-3-mini-4k-instruct").unwrap().endpoint_name, "lmi-mme-20240627093303");
        assert_eq!(endpoints.get_endpoint("Phi-3-mini-4k-instruct").unwrap().target_model, Some("phi-3-mini-4k-instruct.tar.gz".to_owned()));
        assert_eq!(endpoints.get_endpoint("Phi-3-medium-4k-instruct").unwrap().endpoint_name, "lmi-mme-20240627093303");
        assert_eq!(endpoints.get_endpoint("Phi-3-medium-4k-instruct").unwrap().target_model, Some("phi-3-mini-4k-instruct.tar.gz".to_owned()));

        Ok(())
    }
}
