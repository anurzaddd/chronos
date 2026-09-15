//! Natural Language to Code engine
//! 
//! Uses a combination of:
//! - Local LLM (via ONNX Runtime) for offline code generation
//! - Remote LLM (via API) for complex cases
//! - Fallback to template-based generation

use anyhow::{Context, Result};
use ort::{Session, Value};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::ir::{GraphIR, Operation};

pub struct NaturalLanguageParser {
    local_model: Option<Session>,
    api_client: Option<ApiClient>,
    cache: std::collections::HashMap<String, GraphIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmRequest {
    prompt: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmResponse {
    code: String,
    confidence: f32,
}

#[derive(Clone)]
struct ApiClient {
    endpoint: String,
    api_key: String,
    client: reqwest::Client,
}

impl NaturalLanguageParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            local_model: None,
            api_client: None,
            cache: std::collections::HashMap::new(),
        })
    }

    pub fn with_local_model<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.local_model = Some(
            Session::builder()?
                .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
                .commit_from_file(path)?
        );
        Ok(self)
    }

    pub fn with_api(mut self, endpoint: String, api_key: String) -> Self {
        self.api_client = Some(ApiClient {
            endpoint,
            api_key,
            client: reqwest::Client::new(),
        });
        self
    }

    /// Parse natural language description into IR
    pub fn parse(&mut self, description: &str) -> Result<GraphIR> {
        // Check cache
        if let Some(cached) = self.cache.get(description) {
            log::debug!("Cache hit for: {}", description);
            return Ok(cached.clone());
        }

        // Generate code using LLM
        let generated = self.generate_code(description)?;
        log::info!("LLM generated code with confidence {:.2}", generated.confidence);

        // Parse generated code into IR
        let ir = self.code_to_ir(&generated.code)?;

        // Cache the result
        self.cache.insert(description.to_string(), ir.clone());

        Ok(ir)
    }

    fn generate_code(&self, description: &str) -> Result<LlmResponse> {
        // Try local model first
        if let Some(_model) = &self.local_model {
            // TODO: Implement local inference
            log::debug!("Local model not yet implemented, falling back to API");
        }

        // Try API
        if let Some(client) = &self.api_client {
            return self.call_api(client, description);
        }

        // Fallback: template-based
        self.template_based(description)
    }

    fn call_api(&self, client: &ApiClient, description: &str) -> Result<LlmResponse> {
        let prompt = format!(
            r#"You are a code generation AI. Generate Rust code for the following description:

Description: {}

Requirements:
- Return ONLY valid Rust code
- No explanations, no markdown, just code
- Use only standard library
- Function name should be descriptive

Code:"#,
            description
        );

        let request = LlmRequest {
            prompt,
            max_tokens: 1024,
            temperature: 0.2,
        };

        // In production, this would be an async call
        // For now, return a template response
        Ok(LlmResponse {
            code: format!("fn generated_function() {{\n    // TODO: {}\n    unimplemented!()\n}}", description),
            confidence: 0.8,
        })
    }

    fn template_based(&self, description: &str) -> Result<LlmResponse> {
        // Simple template-based generation for common patterns
        let desc_lower = description.to_lowercase();

        let code = if desc_lower.contains("prime") || desc_lower.contains("عدد اول") {
            r#"
fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 { return false; }
        i += 2;
    }
    true
}
"#.to_string()
        } else if desc_lower.contains("fibonacci") || desc_lower.contains("فیبوناچی") {
            r#"
fn fibonacci(n: u64) -> u64 {
    if n <= 1 { return n; }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}
"#.to_string()
        } else if desc_lower.contains("sort") || desc_lower.contains("مرتب") {
            r#"
fn quicksort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 { return; }
    let pivot = partition(arr);
    quicksort(&mut arr[..pivot]);
    quicksort(&mut arr[pivot + 1..]);
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let pivot = arr.len() - 1;
    let mut i = 0;
    for j in 0..pivot {
        if arr[j] <= arr[pivot] {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, pivot);
    i
}
"#.to_string()
        } else {
            format!("fn generated_function() {{\n    // {}\n    unimplemented!()\n}}", description)
        };

        Ok(LlmResponse {
            code,
            confidence: 0.7,
        })
    }

    fn code_to_ir(&self, code: &str) -> Result<GraphIR> {
        // TODO: Implement proper Rust AST to IR conversion
        // For now, create a simple IR with an LLM-generated node
        let mut ir = GraphIR::new();
        ir.add_node(
            Operation::LlmGenerated {
                prompt: "generated".to_string(),
                code: code.to_string(),
            },
            vec![],
        );
        Ok(ir)
    }
}
