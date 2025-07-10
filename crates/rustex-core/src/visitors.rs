//! AST visitor implementations for code element extraction.

use crate::ast_data::*;
use crate::complexity::ComplexityCalculator;
use crate::config::ExtractorConfig;
use std::path::PathBuf;
use syn::visit::Visit;

/// Visitor for extracting code elements from Rust AST.
pub struct CodeElementVisitor {
    /// Collected code elements
    elements: Vec<CodeElement>,
    /// Path to the file being visited
    file_path: PathBuf,
    /// Configuration for extraction
    config: ExtractorConfig,
}

impl CodeElementVisitor {
    /// Create a new visitor for the given file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file being visited
    /// * `config` - Configuration for extraction behavior
    pub fn new(file_path: PathBuf, config: &ExtractorConfig) -> Self {
        Self {
            elements: Vec::new(),
            file_path,
            config: config.clone(),
        }
    }

    /// Extract collected elements from the visitor.
    pub fn into_elements(self) -> Vec<CodeElement> {
        self.elements
    }

    /// Extract documentation comments from attributes.
    fn extract_doc_comments(&self, attrs: &[syn::Attribute]) -> Vec<String> {
        if !self.config.include_docs {
            return vec![];
        }

        attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc") {
                    if let syn::Meta::NameValue(meta) = &attr.meta {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &meta.value
                        {
                            let doc_text = lit_str.value();
                            // Clean up documentation text
                            let cleaned = doc_text
                                .trim()
                                .strip_prefix(' ') // Remove leading space from /// comments
                                .unwrap_or(&doc_text);
                            return Some(cleaned.to_string());
                        }
                    }
                }
                None
            })
            .filter(|doc| !doc.trim().is_empty()) // Filter out empty documentation
            .collect()
    }

    /// Convert syn visibility to our visibility enum.
    fn get_visibility(&self, vis: &syn::Visibility) -> Visibility {
        match vis {
            syn::Visibility::Public(_) => Visibility::Public,
            syn::Visibility::Restricted(restricted) => {
                Visibility::Restricted(format!("{}", quote::quote!(#restricted)))
            }
            syn::Visibility::Inherited => Visibility::Private,
        }
    }

    /// Create location information from a span.
    fn create_location(&self, _span: proc_macro2::Span) -> CodeLocation {
        // Note: proc_macro2::Span doesn't provide line/column info in non-proc-macro context
        // For now, we'll use placeholder values. This will be improved in later tasks.
        CodeLocation {
            line_start: 1,
            line_end: 1,
            char_start: 0,
            char_end: 0,
            file_path: self.file_path.clone(),
        }
    }
}

impl<'ast> Visit<'ast> for CodeElementVisitor {
    /// Visit function items and extract information.
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Skip private items if not configured to include them
        if !self.config.include_private
            && matches!(self.get_visibility(&node.vis), Visibility::Private)
        {
            return;
        }

        let signature = format!("{}", quote::quote!(#node.sig));
        let docs = self.extract_doc_comments(&node.attrs);

        // Calculate detailed complexity metrics
        let complexity_metrics = ComplexityCalculator::calculate_function_complexity(node);
        let complexity_score = complexity_metrics.overall_score();

        let element = CodeElement {
            element_type: ElementType::Function,
            name: node.sig.ident.to_string(),
            signature: Some(signature),
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![], // TODO: Extract inline comments
            location: self.create_location(node.sig.ident.span()),
            attributes: node
                .attrs
                .iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(complexity_score),
            complexity_metrics: Some(complexity_metrics),
            dependencies: vec![], // TODO: Extract function dependencies
            generic_params: node
                .sig
                .generics
                .params
                .iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);

        // Continue visiting nested items
        syn::visit::visit_item_fn(self, node);
    }

    /// Visit struct items and extract information.
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        if !self.config.include_private
            && matches!(self.get_visibility(&node.vis), Visibility::Private)
        {
            return;
        }

        let docs = self.extract_doc_comments(&node.attrs);

        // Calculate structural complexity for struct
        let item = syn::Item::Struct(node.clone());
        let complexity_metrics = ComplexityCalculator::calculate_structural_complexity(&item);
        let complexity_score = complexity_metrics.overall_score();

        let element = CodeElement {
            element_type: ElementType::Struct,
            name: node.ident.to_string(),
            signature: None,
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.ident.span()),
            attributes: node
                .attrs
                .iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(complexity_score),
            complexity_metrics: Some(complexity_metrics),
            dependencies: vec![],
            generic_params: node
                .generics
                .params
                .iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_struct(self, node);
    }

    /// Visit enum items and extract information.
    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        if !self.config.include_private
            && matches!(self.get_visibility(&node.vis), Visibility::Private)
        {
            return;
        }

        let docs = self.extract_doc_comments(&node.attrs);

        // Calculate structural complexity for enum
        let item = syn::Item::Enum(node.clone());
        let complexity_metrics = ComplexityCalculator::calculate_structural_complexity(&item);
        let complexity_score = complexity_metrics.overall_score();

        let element = CodeElement {
            element_type: ElementType::Enum,
            name: node.ident.to_string(),
            signature: None,
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.ident.span()),
            attributes: node
                .attrs
                .iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(complexity_score),
            complexity_metrics: Some(complexity_metrics),
            dependencies: vec![],
            generic_params: node
                .generics
                .params
                .iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_enum(self, node);
    }

    /// Visit trait items and extract information.
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if !self.config.include_private
            && matches!(self.get_visibility(&node.vis), Visibility::Private)
        {
            return;
        }

        let docs = self.extract_doc_comments(&node.attrs);

        // Calculate structural complexity for trait
        let item = syn::Item::Trait(node.clone());
        let complexity_metrics = ComplexityCalculator::calculate_structural_complexity(&item);
        let complexity_score = complexity_metrics.overall_score();

        let element = CodeElement {
            element_type: ElementType::Trait,
            name: node.ident.to_string(),
            signature: None,
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.ident.span()),
            attributes: node
                .attrs
                .iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(complexity_score),
            complexity_metrics: Some(complexity_metrics),
            dependencies: vec![],
            generic_params: node
                .generics
                .params
                .iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_trait(self, node);
    }
}
