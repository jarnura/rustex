//! Output format implementations for different target formats.

use rustex_core::{ProjectAst, OutputFormat};
use crate::rag::{RagFormatter, RagConfig};
use anyhow::Result;

/// Format project AST according to the specified output format.
pub fn format_project_ast(
    project_ast: &ProjectAst,
    format: &OutputFormat,
    pretty: bool,
) -> Result<String> {
    match format {
        OutputFormat::Json => {
            if pretty {
                Ok(serde_json::to_string_pretty(project_ast)?)
            } else {
                Ok(serde_json::to_string(project_ast)?)
            }
        }
        OutputFormat::MessagePack => {
            let data = rmp_serde::to_vec(project_ast)?;
            use base64::Engine;
            Ok(base64::engine::general_purpose::STANDARD.encode(data))
        }
        OutputFormat::Markdown => {
            format_as_markdown(project_ast)
        }
        OutputFormat::GraphQL => {
            format_as_graphql_schema(project_ast)
        }
        OutputFormat::Rag => {
            let formatter = RagFormatter::default();
            let rag_doc = formatter.format(project_ast)?;
            crate::rag::format_as_json(&rag_doc, pretty)
        }
    }
}

/// Format project AST as Markdown documentation.
pub fn format_as_markdown(project_ast: &ProjectAst) -> Result<String> {
    let mut markdown = String::new();
    
    // Add header
    markdown.push_str(&format!("# {}\n\n", project_ast.project.name));
    markdown.push_str(&format!("**Version:** {}\n", project_ast.project.version));
    markdown.push_str(&format!("**Rust Edition:** {}\n\n", project_ast.project.rust_edition));
    
    // Add table of contents
    markdown.push_str("## Table of Contents\n\n");
    for file in &project_ast.files {
        markdown.push_str(&format!("- [{}](#{})\n", 
            file.relative_path.display(),
            file.relative_path.to_string_lossy().replace(['/', '.'], "")
        ));
    }
    markdown.push('\n');
    
    // Add project metrics
    markdown.push_str("## Project Metrics\n\n");
    markdown.push_str(&format!("- **Total Files:** {}\n", project_ast.metrics.total_files));
    markdown.push_str(&format!("- **Total Lines:** {}\n", project_ast.metrics.total_lines));
    markdown.push_str(&format!("- **Total Functions:** {}\n", project_ast.metrics.total_functions));
    markdown.push_str(&format!("- **Total Structs:** {}\n", project_ast.metrics.total_structs));
    markdown.push_str(&format!("- **Total Enums:** {}\n", project_ast.metrics.total_enums));
    markdown.push_str(&format!("- **Total Traits:** {}\n", project_ast.metrics.total_traits));
    markdown.push_str(&format!("- **Average Complexity:** {:.2}\n\n", project_ast.metrics.complexity_average));
    
    // Add file documentation
    for file in &project_ast.files {
        markdown.push_str(&format!("## {}\n\n", file.relative_path.display()));
        
        if !file.elements.is_empty() {
            for element in &file.elements {
                // Add element documentation
                markdown.push_str(&format!("### {:?} `{}`\n\n", 
                    element.element_type,
                    element.name
                ));
                
                // Add documentation comments
                if !element.doc_comments.is_empty() {
                    for doc in &element.doc_comments {
                        markdown.push_str(&format!("{}\n", doc));
                    }
                    markdown.push('\n');
                }
                
                // Add signature
                if let Some(signature) = &element.signature {
                    markdown.push_str("```rust\n");
                    markdown.push_str(signature);
                    markdown.push_str("\n```\n\n");
                }
                
                // Add metadata
                markdown.push_str("**Details:**\n");
                markdown.push_str(&format!("- **Location:** {}:{}-{}\n", 
                    file.relative_path.display(),
                    element.location.line_start,
                    element.location.line_end
                ));
                markdown.push_str(&format!("- **Visibility:** {:?}\n", element.visibility));
                if let Some(complexity) = element.complexity {
                    markdown.push_str(&format!("- **Complexity:** {}\n", complexity));
                }
                markdown.push('\n');
            }
        } else {
            markdown.push_str("*No documented elements in this file.*\n\n");
        }
    }
    
    Ok(markdown)
}

/// Format project AST as GraphQL schema.
pub fn format_as_graphql_schema(project_ast: &ProjectAst) -> Result<String> {
    let mut schema = String::new();
    
    // Add header comment
    schema.push_str(&format!("# GraphQL Schema for {}\n", project_ast.project.name));
    schema.push_str(&format!("# Generated from Rust AST\n\n"));
    
    // Add project type
    schema.push_str("type Project {\n");
    schema.push_str("  name: String!\n");
    schema.push_str("  version: String!\n");
    schema.push_str("  rustEdition: String!\n");
    schema.push_str("  files: [File!]!\n");
    schema.push_str("  metrics: ProjectMetrics!\n");
    schema.push_str("}\n\n");
    
    // Add file type
    schema.push_str("type File {\n");
    schema.push_str("  path: String!\n");
    schema.push_str("  elements: [CodeElement!]!\n");
    schema.push_str("}\n\n");
    
    // Add code element type
    schema.push_str("type CodeElement {\n");
    schema.push_str("  id: String!\n");
    schema.push_str("  elementType: ElementType!\n");
    schema.push_str("  name: String!\n");
    schema.push_str("  signature: String\n");
    schema.push_str("  docComments: [String!]!\n");
    schema.push_str("  visibility: Visibility!\n");
    schema.push_str("  complexity: Int\n");
    schema.push_str("  location: Location!\n");
    schema.push_str("  hierarchy: ElementHierarchy!\n");
    schema.push_str("  crossReferences: CrossReferences!\n");
    schema.push_str("}\n\n");
    
    // Add enums
    schema.push_str("enum ElementType {\n");
    schema.push_str("  FUNCTION\n");
    schema.push_str("  STRUCT\n");
    schema.push_str("  ENUM\n");
    schema.push_str("  TRAIT\n");
    schema.push_str("  IMPLEMENTATION\n");
    schema.push_str("  MODULE\n");
    schema.push_str("}\n\n");
    
    schema.push_str("enum Visibility {\n");
    schema.push_str("  PUBLIC\n");
    schema.push_str("  PRIVATE\n");
    schema.push_str("  CRATE\n");
    schema.push_str("  SUPER\n");
    schema.push_str("}\n\n");
    
    // Add supporting types
    schema.push_str("type Location {\n");
    schema.push_str("  lineStart: Int!\n");
    schema.push_str("  lineEnd: Int!\n");
    schema.push_str("  columnStart: Int!\n");
    schema.push_str("  columnEnd: Int!\n");
    schema.push_str("}\n\n");
    
    schema.push_str("type ElementHierarchy {\n");
    schema.push_str("  qualifiedName: String!\n");
    schema.push_str("  modulePath: String!\n");
    schema.push_str("  parentId: String\n");
    schema.push_str("  children: [String!]!\n");
    schema.push_str("}\n\n");
    
    schema.push_str("type CrossReferences {\n");
    schema.push_str("  outgoing: [String!]!\n");
    schema.push_str("  incoming: [String!]!\n");
    schema.push_str("}\n\n");
    
    schema.push_str("type ProjectMetrics {\n");
    schema.push_str("  totalFiles: Int!\n");
    schema.push_str("  totalLines: Int!\n");
    schema.push_str("  totalFunctions: Int!\n");
    schema.push_str("  totalStructs: Int!\n");
    schema.push_str("  totalEnums: Int!\n");
    schema.push_str("  totalTraits: Int!\n");
    schema.push_str("  complexityAverage: Float!\n");
    schema.push_str("}\n\n");
    
    // Add root query
    schema.push_str("type Query {\n");
    schema.push_str("  project: Project!\n");
    schema.push_str("  file(path: String!): File\n");
    schema.push_str("  element(id: String!): CodeElement\n");
    schema.push_str("  elementsByType(elementType: ElementType!): [CodeElement!]!\n");
    schema.push_str("  elementsByComplexity(minComplexity: Int!): [CodeElement!]!\n");
    schema.push_str("}\n");
    
    Ok(schema)
}

/// Create a RAG formatter with custom configuration.
pub fn create_rag_formatter(config: RagConfig) -> RagFormatter {
    RagFormatter::new(config)
}

/// Format project AST using a custom RAG configuration.
pub fn format_as_rag_with_config(
    project_ast: &ProjectAst,
    config: RagConfig,
    pretty: bool,
) -> Result<String> {
    let formatter = RagFormatter::new(config);
    let rag_doc = formatter.format(project_ast)?;
    crate::rag::format_as_json(&rag_doc, pretty)
}

/// Format project AST as JSONL for streaming/embedding processing.
pub fn format_as_rag_jsonl(project_ast: &ProjectAst) -> Result<String> {
    let formatter = RagFormatter::default();
    let rag_doc = formatter.format(project_ast)?;
    crate::rag::format_as_jsonl(&rag_doc)
}

/// Format project AST for embedding models.
pub fn format_for_embeddings(project_ast: &ProjectAst) -> Result<Vec<crate::rag::EmbeddingInput>> {
    let formatter = RagFormatter::default();
    let rag_doc = formatter.format(project_ast)?;
    crate::rag::format_for_embeddings(&rag_doc)
}
