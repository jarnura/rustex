//! Comprehensive benchmark suite for RustEx performance testing.
//!
//! This suite benchmarks all critical performance paths including:
//! - AST parsing and extraction
//! - Complexity calculation algorithms  
//! - File discovery and filtering
//! - Output formatting
//! - Memory usage patterns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rustex_core::{
    complexity::ComplexityCalculator, visitors::CodeElementVisitor, AstExtractor, ExtractorConfig,
    FilterConfig, OutputFormat,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Generate Rust code samples of varying complexity for benchmarking.
struct CodeSamples {
    simple_function: String,
    complex_function: String,
    large_struct: String,
    complex_enum: String,
    large_trait: String,
    nested_modules: String,
    real_world_file: String,
}

impl CodeSamples {
    fn new() -> Self {
        Self {
            simple_function: Self::generate_simple_function(),
            complex_function: Self::generate_complex_function(),
            large_struct: Self::generate_large_struct(),
            complex_enum: Self::generate_complex_enum(),
            large_trait: Self::generate_large_trait(),
            nested_modules: Self::generate_nested_modules(),
            real_world_file: Self::generate_real_world_file(),
        }
    }

    fn generate_simple_function() -> String {
        r#"
/// Simple function for basic benchmarking
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(x: f64, y: f64) -> f64 {
    x * y
}

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
        "#
        .to_string()
    }

    fn generate_complex_function() -> String {
        r#"
/// Complex function with multiple control flow paths
pub fn complex_algorithm(data: &[i32], threshold: i32, mode: ProcessingMode) -> Result<Vec<ProcessedItem>, ProcessingError> {
    let mut results = Vec::new();
    let mut stats = Statistics::new();
    
    for (index, &value) in data.iter().enumerate() {
        let processed = match mode {
            ProcessingMode::Standard => {
                if value > threshold {
                    match validate_value(value) {
                        Ok(validated) => {
                            for i in 0..validated {
                                if i % 2 == 0 {
                                    stats.increment_even();
                                } else {
                                    stats.increment_odd();
                                }
                                
                                let result = match calculate_result(i, validated) {
                                    Some(r) if r > 0 => r,
                                    Some(_) => return Err(ProcessingError::NegativeResult),
                                    None => continue,
                                };
                                
                                if should_include_result(&result, &stats) {
                                    results.push(ProcessedItem {
                                        original_index: index,
                                        processed_value: result,
                                        metadata: create_metadata(i, validated),
                                    });
                                }
                            }
                            validated
                        }
                        Err(e) => return Err(ProcessingError::ValidationFailed(e)),
                    }
                } else {
                    0
                }
            }
            ProcessingMode::Advanced => {
                let mut accumulated = 0;
                for multiplier in 1..=value {
                    accumulated += multiplier * threshold;
                    if accumulated > MAX_ACCUMULATION {
                        break;
                    }
                }
                accumulated
            }
            ProcessingMode::Experimental => {
                return Err(ProcessingError::UnsupportedMode);
            }
        };
        
        if processed > 0 {
            results.push(ProcessedItem {
                original_index: index,
                processed_value: processed,
                metadata: HashMap::new(),
            });
        }
    }
    
    if results.is_empty() {
        Err(ProcessingError::NoResults)
    } else {
        Ok(results)
    }
}
        "#.to_string()
    }

    fn generate_large_struct() -> String {
        let mut code = String::from("/// Large struct with many fields\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LargeDataStructure {\n");

        for i in 0..50 {
            code.push_str(&format!("    /// Field number {}\n", i));
            code.push_str(&format!(
                "    pub field_{}: {},\n",
                i,
                match i % 5 {
                    0 => "String",
                    1 => "i32",
                    2 => "Vec<u8>",
                    3 => "Option<f64>",
                    _ => "HashMap<String, Value>",
                }
            ));
        }

        code.push_str("}\n\nimpl LargeDataStructure {\n");
        for i in 0..20 {
            code.push_str(&format!(
                r#"
    pub fn get_field_{}(&self) -> &{} {{
        &self.field_{}
    }}
    
    pub fn set_field_{}(&mut self, value: {}) {{
        self.field_{} = value;
    }}
"#,
                i,
                match i % 5 {
                    0 => "String",
                    1 => "i32",
                    2 => "Vec<u8>",
                    3 => "Option<f64>",
                    _ => "HashMap<String, Value>",
                },
                i,
                i,
                match i % 5 {
                    0 => "String",
                    1 => "i32",
                    2 => "Vec<u8>",
                    3 => "Option<f64>",
                    _ => "HashMap<String, Value>",
                },
                i
            ));
        }
        code.push_str("}\n");

        code
    }

    fn generate_complex_enum() -> String {
        r#"
/// Complex enum with various variant types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexDataType {
    /// Simple unit variant
    Empty,
    
    /// Single value variant
    Single(i32),
    
    /// Tuple variant with multiple fields
    Tuple(String, i32, f64, bool),
    
    /// Named fields variant
    Structured {
        id: u64,
        name: String,
        metadata: HashMap<String, serde_json::Value>,
        tags: Vec<String>,
        created_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Nested variant
    Nested {
        inner: Box<ComplexDataType>,
        level: u32,
        context: Option<String>,
    },
    
    /// Generic variant
    Generic(Box<dyn std::any::Any + Send + Sync>),
    
    /// Collection variants
    List(Vec<ComplexDataType>),
    Map(HashMap<String, ComplexDataType>),
    
    /// Result-like variants
    Success { data: Vec<u8>, checksum: u32 },
    Error { code: i32, message: String, stack_trace: Option<String> },
    
    /// State variants
    Pending { started_at: std::time::Instant },
    Processing { progress: f32, estimated_completion: Option<std::time::Duration> },
    Completed { duration: std::time::Duration, result_size: usize },
    Failed { error: String, retry_count: u32 },
}

impl ComplexDataType {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }
    
    pub fn size_hint(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Single(_) => 4,
            Self::Tuple(s, _, _, _) => s.len() + 13,
            Self::Structured { name, metadata, tags, .. } => {
                name.len() + metadata.len() * 10 + tags.iter().map(|t| t.len()).sum::<usize>() + 24
            }
            Self::Nested { inner, .. } => inner.size_hint() + 8,
            Self::List(items) => items.iter().map(|i| i.size_hint()).sum::<usize>(),
            Self::Map(map) => map.iter().map(|(k, v)| k.len() + v.size_hint()).sum::<usize>(),
            _ => 100, // Rough estimate for other variants
        }
    }
}
        "#
        .to_string()
    }

    fn generate_large_trait() -> String {
        let mut code =
            String::from("/// Large trait with many methods\npub trait LargeProcessingTrait {\n");

        for i in 0..30 {
            code.push_str(&format!(
                r#"
    /// Process method number {}
    fn process_{}(&self, input: &[u8]) -> Result<Vec<u8>, ProcessingError>;
    
    /// Transform method number {} with default implementation
    fn transform_{}(&self, data: &mut [u8]) -> usize {{
        let mut count = 0;
        for (index, byte) in data.iter_mut().enumerate() {{
            if index % {} == 0 {{
                *byte = (*byte).wrapping_add({});
                count += 1;
            }}
        }}
        count
    }}
"#,
                i,
                i,
                i,
                i,
                i + 1,
                i + 1
            ));
        }

        code.push_str("}\n");
        code
    }

    fn generate_nested_modules() -> String {
        r#"
/// Top level module with nested submodules
pub mod processing {
    pub mod algorithms {
        pub mod sorting {
            pub fn quick_sort<T: Ord>(arr: &mut [T]) {
                if arr.len() <= 1 { return; }
                let pivot = partition(arr);
                quick_sort(&mut arr[0..pivot]);
                quick_sort(&mut arr[pivot + 1..]);
            }
            
            fn partition<T: Ord>(arr: &mut [T]) -> usize {
                let len = arr.len();
                arr.swap(0, len / 2);
                let mut i = 1;
                for j in 1..len {
                    if arr[j] <= arr[0] {
                        arr.swap(i, j);
                        i += 1;
                    }
                }
                arr.swap(0, i - 1);
                i - 1
            }
        }
        
        pub mod searching {
            pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
                let mut left = 0;
                let mut right = arr.len();
                
                while left < right {
                    let mid = left + (right - left) / 2;
                    match arr[mid].cmp(target) {
                        std::cmp::Ordering::Equal => return Some(mid),
                        std::cmp::Ordering::Less => left = mid + 1,
                        std::cmp::Ordering::Greater => right = mid,
                    }
                }
                None
            }
        }
    }
    
    pub mod io {
        pub mod readers {
            use std::io::{Read, Result};
            
            pub struct BufferedReader<R: Read> {
                inner: R,
                buffer: Vec<u8>,
                pos: usize,
                cap: usize,
            }
            
            impl<R: Read> BufferedReader<R> {
                pub fn new(inner: R) -> Self {
                    Self::with_capacity(8192, inner)
                }
                
                pub fn with_capacity(capacity: usize, inner: R) -> Self {
                    Self {
                        inner,
                        buffer: vec![0; capacity],
                        pos: 0,
                        cap: 0,
                    }
                }
            }
        }
        
        pub mod writers {
            use std::io::{Write, Result};
            
            pub struct BufferedWriter<W: Write> {
                inner: W,
                buffer: Vec<u8>,
            }
            
            impl<W: Write> BufferedWriter<W> {
                pub fn new(inner: W) -> Self {
                    Self::with_capacity(8192, inner)
                }
                
                pub fn with_capacity(capacity: usize, inner: W) -> Self {
                    Self {
                        inner,
                        buffer: Vec::with_capacity(capacity),
                    }
                }
            }
        }
    }
}
        "#
        .to_string()
    }

    fn generate_real_world_file() -> String {
        r#"
//! Real-world style Rust file with mixed content
//! 
//! This simulates a typical production Rust file with various constructs

use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

/// Configuration for the processing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Maximum number of concurrent workers
    pub max_workers: usize,
    /// Timeout for individual tasks
    pub task_timeout: Duration,
    /// Buffer size for queue
    pub buffer_size: usize,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Custom settings
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get(),
            task_timeout: Duration::from_secs(30),
            buffer_size: 1024,
            enable_metrics: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// Status of a processing task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running { started_at: std::time::SystemTime },
    Completed { duration: Duration, result_size: usize },
    Failed { error: String, retry_count: u32 },
    Cancelled,
}

/// A task in the processing queue
#[derive(Debug)]
pub struct Task {
    pub id: u64,
    pub priority: u8,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub status: TaskStatus,
    pub created_at: Instant,
}

impl Task {
    pub fn new(id: u64, payload: Vec<u8>) -> Self {
        Self {
            id,
            priority: 0,
            payload,
            metadata: HashMap::new(),
            status: TaskStatus::Pending,
            created_at: Instant::now(),
        }
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn execute(&mut self) -> Result<Vec<u8>> {
        self.status = TaskStatus::Running { started_at: std::time::SystemTime::now() };
        let start = Instant::now();
        
        // Simulate complex processing
        let mut result = Vec::new();
        for chunk in self.payload.chunks(64) {
            let processed = self.process_chunk(chunk)?;
            result.extend_from_slice(&processed);
            
            // Check for cancellation
            if matches!(self.status, TaskStatus::Cancelled) {
                return Err(anyhow::anyhow!("Task was cancelled"));
            }
        }
        
        let duration = start.elapsed();
        self.status = TaskStatus::Completed { 
            duration, 
            result_size: result.len() 
        };
        
        Ok(result)
    }
    
    fn process_chunk(&self, chunk: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(chunk.len() * 2);
        
        for &byte in chunk {
            match byte {
                0x00..=0x7F => {
                    // ASCII processing
                    output.push(byte);
                    if byte.is_ascii_alphabetic() {
                        output.push(byte ^ 0x20); // Toggle case
                    }
                }
                0x80..=0xFF => {
                    // Extended ASCII processing
                    output.push(byte);
                    output.push(byte.wrapping_sub(0x80));
                }
            }
        }
        
        Ok(output)
    }
}

/// Thread-safe task queue
pub struct TaskQueue {
    tasks: Arc<Mutex<BTreeMap<u64, Task>>>,
    pending: Arc<Mutex<HashSet<u64>>>,
    running: Arc<RwLock<HashSet<u64>>>,
    completed: Arc<RwLock<HashSet<u64>>>,
    config: ProcessingConfig,
    metrics: Arc<Mutex<ProcessingMetrics>>,
}

impl TaskQueue {
    pub fn new(config: ProcessingConfig) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(BTreeMap::new())),
            pending: Arc::new(Mutex::new(HashSet::new())),
            running: Arc::new(RwLock::new(HashSet::new())),
            completed: Arc::new(RwLock::new(HashSet::new())),
            config,
            metrics: Arc::new(Mutex::new(ProcessingMetrics::default())),
        }
    }
    
    pub fn enqueue(&self, task: Task) -> Result<()> {
        let task_id = task.id;
        
        {
            let mut tasks = self.tasks.lock().unwrap();
            let mut pending = self.pending.lock().unwrap();
            
            if tasks.contains_key(&task_id) {
                return Err(anyhow::anyhow!("Task {} already exists", task_id));
            }
            
            tasks.insert(task_id, task);
            pending.insert(task_id);
        }
        
        if self.config.enable_metrics {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.tasks_enqueued += 1;
        }
        
        Ok(())
    }
    
    pub fn dequeue(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        let mut pending = self.pending.lock().unwrap();
        
        // Get highest priority pending task
        let task_id = pending.iter()
            .filter_map(|&id| tasks.get(&id).map(|task| (id, task.priority)))
            .max_by_key(|(_, priority)| *priority)
            .map(|(id, _)| id)?;
            
        pending.remove(&task_id);
        tasks.remove(&task_id)
    }
    
    pub fn get_status(&self, task_id: u64) -> Option<TaskStatus> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&task_id).map(|task| task.status.clone())
    }
    
    pub fn cancel_task(&self, task_id: u64) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        
        if let Some(task) = tasks.get_mut(&task_id) {
            if matches!(task.status, TaskStatus::Pending | TaskStatus::Running { .. }) {
                task.status = TaskStatus::Cancelled;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Cannot cancel task in status {:?}", task.status))
            }
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }
    
    pub fn get_metrics(&self) -> ProcessingMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

/// Processing metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    pub tasks_enqueued: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_cancelled: u64,
    pub total_processing_time: Duration,
    pub average_task_size: f64,
    pub peak_memory_usage: usize,
}

impl ProcessingMetrics {
    pub fn success_rate(&self) -> f64 {
        if self.tasks_enqueued == 0 {
            0.0
        } else {
            self.tasks_completed as f64 / self.tasks_enqueued as f64
        }
    }
    
    pub fn failure_rate(&self) -> f64 {
        if self.tasks_enqueued == 0 {
            0.0
        } else {
            self.tasks_failed as f64 / self.tasks_enqueued as f64
        }
    }
}

/// Trait for custom task processors
pub trait TaskProcessor: Send + Sync {
    fn process(&self, task: &mut Task) -> Result<Vec<u8>>;
    fn can_handle(&self, task: &Task) -> bool;
    fn priority(&self) -> u8;
}

/// Default task processor implementation
pub struct DefaultProcessor;

impl TaskProcessor for DefaultProcessor {
    fn process(&self, task: &mut Task) -> Result<Vec<u8>> {
        task.execute()
    }
    
    fn can_handle(&self, _task: &Task) -> bool {
        true
    }
    
    fn priority(&self) -> u8 {
        0
    }
}
        "#
        .to_string()
    }
}

/// Create a temporary directory with test files for benchmarking.
fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let samples = CodeSamples::new();

    // Create various test files
    fs::write(src_dir.join("simple.rs"), samples.simple_function).unwrap();
    fs::write(src_dir.join("complex.rs"), samples.complex_function).unwrap();
    fs::write(src_dir.join("large_struct.rs"), samples.large_struct).unwrap();
    fs::write(src_dir.join("complex_enum.rs"), samples.complex_enum).unwrap();
    fs::write(src_dir.join("large_trait.rs"), samples.large_trait).unwrap();
    fs::write(src_dir.join("nested_modules.rs"), samples.nested_modules).unwrap();
    fs::write(src_dir.join("real_world.rs"), samples.real_world_file).unwrap();

    // Create Cargo.toml
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "benchmark-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
"#,
    )
    .unwrap();

    temp_dir
}

/// Benchmark AST parsing for different code complexities.
fn bench_ast_parsing(c: &mut Criterion) {
    let samples = CodeSamples::new();

    let mut group = c.benchmark_group("ast_parsing");

    // Benchmark parsing different code complexities
    let test_cases = vec![
        ("simple_function", &samples.simple_function),
        ("complex_function", &samples.complex_function),
        ("large_struct", &samples.large_struct),
        ("complex_enum", &samples.complex_enum),
        ("large_trait", &samples.large_trait),
        ("nested_modules", &samples.nested_modules),
        ("real_world_file", &samples.real_world_file),
    ];

    for (name, code) in test_cases {
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), code, |b, code| {
            b.iter(|| syn::parse_file(black_box(code)).expect("Failed to parse code"));
        });
    }

    group.finish();
}

/// Benchmark complexity calculation algorithms.
fn bench_complexity_calculation(c: &mut Criterion) {
    let samples = CodeSamples::new();

    let mut group = c.benchmark_group("complexity_calculation");

    // Pre-parse the code samples
    let parsed_samples: Vec<(String, syn::File)> = vec![
        (
            "simple_function".to_string(),
            syn::parse_file(&samples.simple_function).unwrap(),
        ),
        (
            "complex_function".to_string(),
            syn::parse_file(&samples.complex_function).unwrap(),
        ),
        (
            "large_struct".to_string(),
            syn::parse_file(&samples.large_struct).unwrap(),
        ),
        (
            "complex_enum".to_string(),
            syn::parse_file(&samples.complex_enum).unwrap(),
        ),
        (
            "large_trait".to_string(),
            syn::parse_file(&samples.large_trait).unwrap(),
        ),
        (
            "nested_modules".to_string(),
            syn::parse_file(&samples.nested_modules).unwrap(),
        ),
        (
            "real_world_file".to_string(),
            syn::parse_file(&samples.real_world_file).unwrap(),
        ),
    ];

    for (name, parsed) in &parsed_samples {
        group.bench_with_input(BenchmarkId::new("complexity", name), parsed, |b, parsed| {
            b.iter(|| {
                for item in &parsed.items {
                    match item {
                        syn::Item::Fn(item_fn) => {
                            black_box(ComplexityCalculator::calculate_function_complexity(item_fn));
                        }
                        item => {
                            black_box(ComplexityCalculator::calculate_structural_complexity(item));
                        }
                    }
                }
            });
        });
    }

    group.finish();
}

/// Benchmark full extraction workflow.
fn bench_full_extraction(c: &mut Criterion) {
    let temp_dir = create_test_project();
    let config = ExtractorConfig::default();

    let mut group = c.benchmark_group("full_extraction");
    group.sample_size(10); // Reduce sample size for expensive operations

    group.bench_function("extract_project", |b| {
        b.iter(|| {
            let extractor = AstExtractor::new(
                black_box(config.clone()),
                black_box(temp_dir.path().to_path_buf()),
            );
            extractor.extract_project().unwrap()
        });
    });

    group.finish();
}

/// Benchmark visitor pattern performance.
fn bench_visitor_performance(c: &mut Criterion) {
    let samples = CodeSamples::new();
    let config = ExtractorConfig::default();

    let mut group = c.benchmark_group("visitor_performance");

    let test_cases = vec![
        (
            "simple_function".to_string(),
            syn::parse_file(&samples.simple_function).unwrap(),
        ),
        (
            "complex_function".to_string(),
            syn::parse_file(&samples.complex_function).unwrap(),
        ),
        (
            "large_struct".to_string(),
            syn::parse_file(&samples.large_struct).unwrap(),
        ),
        (
            "complex_enum".to_string(),
            syn::parse_file(&samples.complex_enum).unwrap(),
        ),
        (
            "real_world_file".to_string(),
            syn::parse_file(&samples.real_world_file).unwrap(),
        ),
    ];

    for (name, parsed) in test_cases {
        group.throughput(Throughput::Elements(parsed.items.len() as u64));
        group.bench_with_input(BenchmarkId::new("visit", name), &parsed, |b, parsed| {
            b.iter(|| {
                let mut visitor =
                    CodeElementVisitor::new(PathBuf::from("test.rs"), black_box(&config));
                syn::visit::visit_file(&mut visitor, black_box(parsed));
                black_box(visitor.into_elements())
            });
        });
    }

    group.finish();
}

/// Benchmark output formatting performance.
fn bench_output_formatting(c: &mut Criterion) {
    let temp_dir = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, temp_dir.path().to_path_buf());
    let project_ast = extractor.extract_project().unwrap();

    let mut group = c.benchmark_group("output_formatting");
    group.throughput(Throughput::Elements(project_ast.files.len() as u64));

    let output_formats = vec![
        OutputFormat::Json,
        OutputFormat::MessagePack,
        OutputFormat::Markdown,
        OutputFormat::Rag,
    ];

    for format in output_formats {
        group.bench_with_input(
            BenchmarkId::new("format", format!("{:?}", format)),
            &format,
            |b, format| {
                b.iter(|| {
                    let formatted = match format {
                        OutputFormat::Json => {
                            serde_json::to_string(black_box(&project_ast)).unwrap()
                        }
                        OutputFormat::MessagePack => {
                            let bytes = rmp_serde::to_vec(black_box(&project_ast)).unwrap();
                            String::from_utf8_lossy(&bytes).to_string()
                        }
                        OutputFormat::Markdown => {
                            // Simplified markdown formatting for benchmark
                            format!(
                                "# Project: {}\n\nFiles: {}",
                                project_ast.project.name,
                                project_ast.files.len()
                            )
                        }
                        OutputFormat::Rag => {
                            // Simplified RAG formatting for benchmark
                            project_ast
                                .files
                                .iter()
                                .map(|f| format!("File: {:?}", f.path))
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                        _ => String::new(),
                    };
                    black_box(formatted)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark file filtering performance.
fn bench_file_filtering(c: &mut Criterion) {
    let temp_dir = create_test_project();

    // Create many additional files for filtering benchmark
    let src_dir = temp_dir.path().join("src");
    for i in 0..100 {
        fs::write(src_dir.join(format!("file_{}.rs", i)), "pub fn dummy() {}").unwrap();
    }

    let mut group = c.benchmark_group("file_filtering");

    let filter_configs = vec![
        (
            "include_all",
            FilterConfig {
                include: vec!["**/*.rs".to_string()],
                exclude: vec![],
            },
        ),
        (
            "exclude_some",
            FilterConfig {
                include: vec!["**/*.rs".to_string()],
                exclude: vec!["**/file_5*.rs".to_string()],
            },
        ),
        (
            "complex_patterns",
            FilterConfig {
                include: vec!["src/**/*.rs".to_string(), "tests/**/*.rs".to_string()],
                exclude: vec!["target/**".to_string(), "**/test_*.rs".to_string()],
            },
        ),
    ];

    for (name, filter_config) in filter_configs {
        group.bench_with_input(
            BenchmarkId::new("filter", name),
            &filter_config,
            |b, filter_config| {
                b.iter(|| {
                    let mut config = ExtractorConfig::default();
                    config.filters = filter_config.clone();
                    let extractor =
                        AstExtractor::new(config, black_box(temp_dir.path().to_path_buf()));
                    // Since discover_rust_files is private, we'll benchmark the full extraction instead
                    let project_ast = extractor.extract_project().unwrap();
                    black_box(project_ast.files.len())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns.
fn bench_memory_usage(c: &mut Criterion) {
    let samples = CodeSamples::new();

    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(20);

    // Benchmark memory allocation patterns for different sized inputs
    let sizes = vec![1, 10, 50, 100];

    for size in sizes {
        group.bench_with_input(BenchmarkId::new("allocations", size), &size, |b, &size| {
            b.iter(|| {
                let mut all_elements = Vec::new();

                for _ in 0..size {
                    let parsed = syn::parse_file(black_box(&samples.real_world_file)).unwrap();
                    let config = ExtractorConfig::default();
                    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);
                    syn::visit::visit_file(&mut visitor, &parsed);
                    let elements = visitor.into_elements();
                    all_elements.extend(elements);
                }

                black_box(all_elements)
            });
        });
    }

    group.finish();
}

/// Benchmark scalability with project size.
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    let file_counts = vec![1, 5, 10, 25, 50];

    for file_count in file_counts {
        group.bench_with_input(
            BenchmarkId::new("project_size", file_count),
            &file_count,
            |b, &file_count| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let src_dir = temp_dir.path().join("src");
                    fs::create_dir_all(&src_dir).unwrap();

                    let samples = CodeSamples::new();

                    // Create multiple files
                    for i in 0..file_count {
                        let content = match i % 4 {
                            0 => &samples.simple_function,
                            1 => &samples.complex_function,
                            2 => &samples.large_struct,
                            _ => &samples.real_world_file,
                        };
                        fs::write(src_dir.join(format!("file_{}.rs", i)), content).unwrap();
                    }

                    fs::write(
                        temp_dir.path().join("Cargo.toml"),
                        r#"
[package]
name = "scalability-test"
version = "0.1.0"
edition = "2021"
"#,
                    )
                    .unwrap();

                    let config = ExtractorConfig::default();
                    let extractor =
                        AstExtractor::new(config, black_box(temp_dir.path().to_path_buf()));
                    let result = extractor.extract_project().unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ast_parsing,
    bench_complexity_calculation,
    bench_full_extraction,
    bench_visitor_performance,
    bench_output_formatting,
    bench_file_filtering,
    bench_memory_usage,
    bench_scalability
);

criterion_main!(benches);
