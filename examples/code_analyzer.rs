//! Code Analyzer Example
//! 
//! This example demonstrates how to use RustEx to perform comprehensive
//! code analysis, including complexity metrics, documentation coverage,
//! and quality assessments.

use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase, ElementType, Visibility};
use std::path::PathBuf;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 RustEx Code Analyzer");
    println!("═══════════════════════");

    // Use code analysis configuration
    let config = ExtractorConfig::for_use_case(ConfigUseCase::CodeAnalysis);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    println!("📊 Analyzing project...");
    let project_ast = extractor.extract_project()?;
    
    // Perform comprehensive analysis
    let analysis = perform_code_analysis(&project_ast);
    
    // Display results
    display_analysis_results(&analysis);
    
    // Generate report
    generate_analysis_report(&project_ast, &analysis)?;
    
    Ok(())
}

#[derive(Debug)]
struct CodeAnalysis {
    complexity_distribution: HashMap<u32, u32>,
    high_complexity_functions: Vec<ComplexFunction>,
    documentation_coverage: f64,
    file_analysis: Vec<FileAnalysis>,
    quality_score: f64,
    recommendations: Vec<String>,
}

#[derive(Debug)]
#[allow(dead_code)] // Used in analysis reporting
struct ComplexFunction {
    name: String,
    complexity: u32,
    file: PathBuf,
    line: u32,
    function_type: String,
}

#[derive(Debug, Clone)]
struct FileAnalysis {
    path: PathBuf,
    elements_count: usize,
    avg_complexity: f64,
    documentation_ratio: f64,
    quality_issues: Vec<String>,
}

fn perform_code_analysis(project_ast: &rustex_core::ProjectAst) -> CodeAnalysis {
    let mut complexity_distribution: HashMap<u32, u32> = HashMap::new();
    let mut high_complexity_functions = Vec::new();
    let mut total_public_items = 0;
    let mut documented_items = 0;
    let mut file_analyses = Vec::new();
    
    for file in &project_ast.files {
        let mut file_complexity_sum = 0.0;
        let mut file_complexity_count = 0;
        let mut file_public_items = 0;
        let mut file_documented_items = 0;
        let mut quality_issues = Vec::new();
        
        for element in &file.elements {
            // Complexity analysis
            if let Some(complexity) = element.complexity {
                *complexity_distribution.entry(complexity).or_insert(0) += 1;
                file_complexity_sum += complexity as f64;
                file_complexity_count += 1;
                
                // Identify high complexity functions
                if complexity > 10 {
                    high_complexity_functions.push(ComplexFunction {
                        name: element.name.clone(),
                        complexity,
                        file: file.relative_path.clone(),
                        line: element.location.line_start as u32,
                        function_type: format!("{:?}", element.element_type),
                    });
                }
                
                // Quality issues
                if complexity > 15 {
                    quality_issues.push(format!(
                        "Very high complexity in {}: {} ({})",
                        element.name, complexity, format!("{:?}", element.element_type)
                    ));
                }
            }
            
            // Documentation analysis
            if element.visibility == Visibility::Public {
                total_public_items += 1;
                file_public_items += 1;
                
                if !element.doc_comments.is_empty() {
                    documented_items += 1;
                    file_documented_items += 1;
                } else {
                    quality_issues.push(format!(
                        "Missing documentation for public {}: {}",
                        format!("{:?}", element.element_type).to_lowercase(),
                        element.name
                    ));
                }
            }
            
            // Function-specific analysis
            if element.element_type == ElementType::Function {
                if let Some(metrics) = &element.complexity_metrics {
                    if metrics.parameter_count > 7 {
                        quality_issues.push(format!(
                            "Too many parameters in {}: {}",
                            element.name, metrics.parameter_count
                        ));
                    }
                    
                    if metrics.lines_of_code > 50 {
                        quality_issues.push(format!(
                            "Function {} is too long: {} lines",
                            element.name, metrics.lines_of_code
                        ));
                    }
                }
            }
        }
        
        let file_avg_complexity = if file_complexity_count > 0 {
            file_complexity_sum / file_complexity_count as f64
        } else {
            0.0
        };
        
        let file_doc_ratio = if file_public_items > 0 {
            file_documented_items as f64 / file_public_items as f64
        } else {
            1.0
        };
        
        file_analyses.push(FileAnalysis {
            path: file.relative_path.clone(),
            elements_count: file.elements.len(),
            avg_complexity: file_avg_complexity,
            documentation_ratio: file_doc_ratio,
            quality_issues,
        });
    }
    
    // Sort high complexity functions by complexity
    high_complexity_functions.sort_by_key(|f| std::cmp::Reverse(f.complexity));
    
    // Calculate overall documentation coverage
    let documentation_coverage = if total_public_items > 0 {
        documented_items as f64 / total_public_items as f64
    } else {
        1.0
    };
    
    // Calculate quality score (0-100)
    let quality_score = calculate_quality_score(&project_ast.metrics, documentation_coverage);
    
    // Generate recommendations
    let recommendations = generate_recommendations(
        &project_ast.metrics,
        documentation_coverage,
        &high_complexity_functions,
    );
    
    CodeAnalysis {
        complexity_distribution,
        high_complexity_functions,
        documentation_coverage,
        file_analysis: file_analyses,
        quality_score,
        recommendations,
    }
}

fn calculate_quality_score(
    metrics: &rustex_core::ProjectMetrics, 
    doc_coverage: f64
) -> f64 {
    let complexity_score = if metrics.complexity_average > 0.0 {
        (20.0 - metrics.complexity_average.min(20.0)) / 20.0 * 40.0
    } else {
        40.0
    };
    
    let documentation_score = doc_coverage * 30.0;
    
    let size_score = if metrics.total_functions > 0 {
        let avg_lines_per_function = metrics.total_lines as f64 / metrics.total_functions as f64;
        if avg_lines_per_function < 20.0 {
            30.0
        } else if avg_lines_per_function < 50.0 {
            20.0
        } else {
            10.0
        }
    } else {
        30.0
    };
    
    (complexity_score + documentation_score + size_score).min(100.0)
}

fn generate_recommendations(
    metrics: &rustex_core::ProjectMetrics,
    doc_coverage: f64,
    high_complexity_functions: &[ComplexFunction],
) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if doc_coverage < 0.8 {
        recommendations.push(format!(
            "📚 Improve documentation coverage from {:.1}% to at least 80%",
            doc_coverage * 100.0
        ));
    }
    
    if metrics.complexity_average > 10.0 {
        recommendations.push(
            "🔧 Consider refactoring complex functions to reduce cognitive load".to_string()
        );
    }
    
    if !high_complexity_functions.is_empty() {
        recommendations.push(format!(
            "⚠️  {} functions have high complexity (>10). Consider breaking them down",
            high_complexity_functions.len()
        ));
    }
    
    if metrics.total_functions > 100 {
        recommendations.push(
            "📦 Consider organizing code into smaller modules for better maintainability".to_string()
        );
    }
    
    let avg_lines_per_function = if metrics.total_functions > 0 {
        metrics.total_lines as f64 / metrics.total_functions as f64
    } else {
        0.0
    };
    
    if avg_lines_per_function > 25.0 {
        recommendations.push(
            "📏 Functions are averaging more than 25 lines. Consider shorter, focused functions".to_string()
        );
    }
    
    if recommendations.is_empty() {
        recommendations.push("✨ Great job! Your code quality looks excellent".to_string());
    }
    
    recommendations
}

fn display_analysis_results(analysis: &CodeAnalysis) {
    use colored::Colorize;
    
    println!("\n🎯 Analysis Results");
    println!("═══════════════════");
    
    // Overall quality score
    let quality_score_str = if analysis.quality_score >= 80.0 {
        format!("{:.1}/100", analysis.quality_score).green()
    } else if analysis.quality_score >= 60.0 {
        format!("{:.1}/100", analysis.quality_score).yellow()
    } else {
        format!("{:.1}/100", analysis.quality_score).red()
    };
    
    println!("📊 Overall Quality Score: {}", quality_score_str);
    
    // Documentation coverage
    let doc_coverage_str = if analysis.documentation_coverage >= 0.8 {
        format!("{:.1}%", analysis.documentation_coverage * 100.0).green()
    } else if analysis.documentation_coverage >= 0.5 {
        format!("{:.1}%", analysis.documentation_coverage * 100.0).yellow()
    } else {
        format!("{:.1}%", analysis.documentation_coverage * 100.0).red()
    };
    
    println!("📚 Documentation Coverage: {}", doc_coverage_str);
    
    // Complexity distribution
    println!("\n📈 Complexity Distribution:");
    for (complexity, count) in &analysis.complexity_distribution {
        let bar = "█".repeat((*count as usize).min(50));
        println!("  Complexity {}: {} {}", complexity, count, bar);
    }
    
    // High complexity functions
    if !analysis.high_complexity_functions.is_empty() {
        println!("\n⚠️  High Complexity Functions:");
        for func in analysis.high_complexity_functions.iter().take(10) {
            println!("  • {} (complexity: {}) in {}:{}",
                func.name.yellow(),
                func.complexity.to_string().red(),
                func.file.display(),
                func.line
            );
        }
        
        if analysis.high_complexity_functions.len() > 10 {
            println!("  ... and {} more", analysis.high_complexity_functions.len() - 10);
        }
    }
    
    // File-level analysis
    println!("\n📁 File Analysis:");
    let mut sorted_files = analysis.file_analysis.clone();
    sorted_files.sort_by(|a, b| b.avg_complexity.partial_cmp(&a.avg_complexity).unwrap());
    
    for file in sorted_files.iter().take(5) {
        let complexity_str = if file.avg_complexity > 10.0 { 
            format!("{:.1}", file.avg_complexity).red()
        } else if file.avg_complexity > 5.0 { 
            format!("{:.1}", file.avg_complexity).yellow()
        } else { 
            format!("{:.1}", file.avg_complexity).green()
        };
        
        println!("  📄 {} ({} elements, avg complexity: {})",
            file.path.display(),
            file.elements_count,
            complexity_str
        );
        
        if !file.quality_issues.is_empty() {
            for issue in file.quality_issues.iter().take(3) {
                println!("     ⚠️  {}", issue);
            }
            if file.quality_issues.len() > 3 {
                println!("     ... and {} more issues", file.quality_issues.len() - 3);
            }
        }
    }
    
    // Recommendations
    println!("\n💡 Recommendations:");
    for rec in &analysis.recommendations {
        println!("  {}", rec);
    }
}

fn generate_analysis_report(
    project_ast: &rustex_core::ProjectAst,
    analysis: &CodeAnalysis,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut report = String::new();
    
    // Header
    report.push_str(&format!("# Code Analysis Report: {}\n\n", project_ast.project.name));
    report.push_str(&format!("**Generated:** {}\n", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    report.push_str(&format!("**Version:** {}\n\n", project_ast.project.version));
    
    // Executive Summary
    report.push_str("## Executive Summary\n\n");
    report.push_str(&format!("- **Overall Quality Score:** {:.1}/100\n", analysis.quality_score));
    report.push_str(&format!("- **Documentation Coverage:** {:.1}%\n", analysis.documentation_coverage * 100.0));
    report.push_str(&format!("- **Total Functions:** {}\n", project_ast.metrics.total_functions));
    report.push_str(&format!("- **Average Complexity:** {:.2}\n", project_ast.metrics.complexity_average));
    report.push_str(&format!("- **High Complexity Functions:** {}\n\n", analysis.high_complexity_functions.len()));
    
    // Detailed Metrics
    report.push_str("## Detailed Metrics\n\n");
    report.push_str("### Project Statistics\n\n");
    report.push_str(&format!("| Metric | Value |\n"));
    report.push_str(&format!("|--------|-------|\n"));
    report.push_str(&format!("| Files | {} |\n", project_ast.metrics.total_files));
    report.push_str(&format!("| Lines of Code | {} |\n", project_ast.metrics.total_lines));
    report.push_str(&format!("| Functions | {} |\n", project_ast.metrics.total_functions));
    report.push_str(&format!("| Structs | {} |\n", project_ast.metrics.total_structs));
    report.push_str(&format!("| Enums | {} |\n", project_ast.metrics.total_enums));
    report.push_str(&format!("| Traits | {} |\n\n", project_ast.metrics.total_traits));
    
    // Complexity Analysis
    report.push_str("### Complexity Distribution\n\n");
    report.push_str("| Complexity Level | Count |\n");
    report.push_str("|------------------|-------|\n");
    for (complexity, count) in &analysis.complexity_distribution {
        report.push_str(&format!("| {} | {} |\n", complexity, count));
    }
    report.push('\n');
    
    // High Complexity Functions
    if !analysis.high_complexity_functions.is_empty() {
        report.push_str("### High Complexity Functions\n\n");
        report.push_str("| Function | Complexity | File | Line |\n");
        report.push_str("|----------|------------|------|------|\n");
        for func in &analysis.high_complexity_functions {
            report.push_str(&format!("| `{}` | {} | {} | {} |\n",
                func.name, func.complexity, func.file.display(), func.line));
        }
        report.push('\n');
    }
    
    // File Analysis
    report.push_str("### File Analysis\n\n");
    report.push_str("| File | Elements | Avg Complexity | Doc Coverage |\n");
    report.push_str("|------|----------|----------------|-------------|\n");
    for file in &analysis.file_analysis {
        report.push_str(&format!("| {} | {} | {:.1} | {:.1}% |\n",
            file.path.display(),
            file.elements_count,
            file.avg_complexity,
            file.documentation_ratio * 100.0
        ));
    }
    report.push('\n');
    
    // Recommendations
    report.push_str("## Recommendations\n\n");
    for (i, rec) in analysis.recommendations.iter().enumerate() {
        report.push_str(&format!("{}. {}\n", i + 1, rec));
    }
    report.push('\n');
    
    // Quality Issues by File
    report.push_str("## Quality Issues by File\n\n");
    for file in &analysis.file_analysis {
        if !file.quality_issues.is_empty() {
            report.push_str(&format!("### {}\n\n", file.path.display()));
            for issue in &file.quality_issues {
                report.push_str(&format!("- {}\n", issue));
            }
            report.push('\n');
        }
    }
    
    // Footer
    report.push_str("---\n\n");
    report.push_str("*Generated by RustEx Code Analyzer*\n");
    
    // Save report
    let output_file = "code-analysis-report.md";
    std::fs::write(output_file, report)?;
    
    println!("\n✅ Analysis complete!");
    println!("📄 Report saved to: {}", output_file);
    
    Ok(())
}