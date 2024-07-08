use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Hashmap with a model name as key and an endpoint name as value.
type SageMakerEndpoints = HashMap<String, String>;

pub fn load_from_config_file<P: AsRef<Path>>(path: P) -> Result<SageMakerEndpoints, Box<dyn Error>>
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::io::Write;

    use tempfile::tempdir;

    use crate::sagemaker_endpoint_loader::load_from_config_file;

    #[test]
    fn test_load_from_config_file() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let config_path = dir.path().join("config.json");
        let mut config_file = File::create(&config_path)?;
        config_file.write_all(br#"{
            "Llama-3-70B-Instruct": "lmi-llama-3-70b-instruct",
            "Phi-3-mini-4k-instruct": "lmi-llama-phi-3-mini-4k"
        }"#)?;

        let endpoints = load_from_config_file(&config_path)?;
        assert_eq!(endpoints["Llama-3-70B-Instruct"], "lmi-llama-3-70b-instruct");
        assert_eq!(endpoints["Phi-3-mini-4k-instruct"], "lmi-llama-phi-3-mini-4k");

        Ok(())
    }
}
