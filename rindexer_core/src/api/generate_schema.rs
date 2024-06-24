use crate::api::generate_operations::{generate_operations, GenerateOperationsError};
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum GenerateTypingsError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("File system error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No data in response")]
    NoData,

    #[error("Failed to generate operations: {0}")]
    GenerateOperationsError(GenerateOperationsError),
}

pub async fn generate_graphql_queries(
    endpoint: &str,
    generate_path: &Path,
) -> Result<(), GenerateTypingsError> {
    let client = Client::new();
    let introspection_query = r#"
    {
      __schema {
        types {
          name
          fields {
            name
            args {
              name
              type {
                name
                kind
                ofType {
                  name
                  kind
                  ofType {
                    name
                    kind
                    ofType {
                      name
                      kind
                    }
                  }
                }
              }
            }
            type {
              name
              kind
              ofType {
                name
                kind
                fields {
                  name
                  type {
                    name
                    kind
                    ofType {
                      name
                      kind
                      ofType {
                        name
                        kind
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    "#;

    let res = client
        .post(endpoint)
        .json(&serde_json::json!({ "query": introspection_query }))
        .send()
        .await?
        .json::<Value>()
        .await?;

    let schema = res["data"]["__schema"].clone();
    if schema.is_null() {
        return Err(GenerateTypingsError::NoData);
    }

    let schema_str = serde_json::to_string_pretty(&schema)?;

    let schema_path = generate_path.join("schema.graphql");
    let mut file = File::create(schema_path)?;
    file.write_all(schema_str.as_bytes())?;

    generate_operations(&schema, generate_path)
        .map_err(GenerateTypingsError::GenerateOperationsError)?;

    Ok(())
}