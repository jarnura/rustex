//! AST visitor implementations for code element extraction.

use crate::ast_data::*;
use crate::complexity::ComplexityCalculator;
use crate::config::ExtractorConfig;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::spanned::Spanned;

/// Visitor for extracting code elements from Rust AST.
pub struct CodeElementVisitor {
    /// Collected code elements
    elements: Vec<CodeElement>,
    /// Path to the file being visited
    file_path: PathBuf,
    /// Configuration for extraction
    config: ExtractorConfig,
    /// Hierarchy builder for tracking relationships
    hierarchy_builder: crate::ast_data::HierarchyBuilder,
    /// Cross-reference resolver for tracking element relationships
    cross_ref_resolver: crate::ast_data::CrossReferenceResolver,
    /// Current element ID stack for reference tracking
    current_element_stack: Vec<String>,
    /// Namespace resolver for handling imports and qualified names
    namespace_resolver: crate::ast_data::NamespaceResolver,
}

impl CodeElementVisitor {
    /// Create a new visitor for the given file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file being visited
    /// * `config` - Configuration for extraction behavior
    pub fn new(file_path: PathBuf, config: &ExtractorConfig) -> Self {
        // Extract module path from file path
        let module_path = Self::extract_module_path(&file_path);
        
        Self {
            elements: Vec::new(),
            file_path,
            config: config.clone(),
            hierarchy_builder: crate::ast_data::HierarchyBuilder::new(module_path.clone()),
            cross_ref_resolver: crate::ast_data::CrossReferenceResolver::new(),
            current_element_stack: Vec::new(),
            namespace_resolver: crate::ast_data::NamespaceResolver::new(module_path),
        }
    }

    /// Extract collected elements and cross-references from the visitor.
    pub fn into_elements_and_references(mut self) -> (Vec<CodeElement>, Vec<crate::ast_data::CrossReference>) {
        // Post-process to update parent-child relationships
        self.update_parent_child_relationships();
        
        // Update namespace information with resolved imports
        self.update_namespace_information();
        
        // Resolve cross-references
        self.cross_ref_resolver.resolve_references();
        
        let cross_references = self.cross_ref_resolver.get_cross_references().to_vec();
        (self.elements, cross_references)
    }
    
    /// Extract collected elements from the visitor (for backward compatibility).
    pub fn into_elements(self) -> Vec<CodeElement> {
        let (elements, _) = self.into_elements_and_references();
        elements
    }
    
    /// Extract module path from file path.
    fn extract_module_path(file_path: &Path) -> String {
        if let Some(file_name) = file_path.file_stem().and_then(|n| n.to_str()) {
            if file_name == "main" || file_name == "lib" {
                "crate".to_string()
            } else {
                format!("crate::{}", file_name)
            }
        } else {
            "crate".to_string()
        }
    }
    
    /// Update parent-child relationships after all elements are collected.
    fn update_parent_child_relationships(&mut self) {
        let mut parent_to_children: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        
        // Group children by parent
        for element in &self.elements {
            if let Some(parent_id) = &element.hierarchy.parent_id {
                parent_to_children
                    .entry(parent_id.clone())
                    .or_default()
                    .push(element.id.clone());
            }
        }
        
        // Update parent elements with their children
        for element in &mut self.elements {
            if let Some(children) = parent_to_children.get(&element.id) {
                element.hierarchy.children_ids = children.clone();
            }
        }
    }
    
    /// Track a cross-reference from the current element.
    fn track_reference(&mut self, reference_type: crate::ast_data::ReferenceType, reference_text: String, location: proc_macro2::Span) {
        if let Some(current_element_id) = self.current_element_stack.last() {
            let context = crate::ast_data::ReferenceContext::new(
                false, // This is a usage, not a definition
                self.cross_ref_resolver.current_scope(),
            );
            
            let cross_ref = crate::ast_data::CrossReference::new(
                current_element_id.clone(),
                reference_type,
                reference_text,
                self.create_location(location),
                context,
            );
            
            self.cross_ref_resolver.add_reference(cross_ref);
        }
    }
    
    /// Register an element for cross-reference resolution.
    fn register_element(&mut self, name: &str, element_id: &str) {
        // Register both simple name and qualified name
        self.cross_ref_resolver.register_element(name.to_string(), element_id.to_string());
        
        // Register with current scope prefix if we're in a nested scope
        let scope = self.cross_ref_resolver.current_scope();
        if !scope.is_empty() {
            let qualified_name = format!("{}::{}", scope, name);
            self.cross_ref_resolver.register_element(qualified_name, element_id.to_string());
        }
    }
    
    /// Enter a new element scope for cross-reference tracking.
    fn enter_element_scope(&mut self, element_id: String) {
        self.current_element_stack.push(element_id.clone());
        self.cross_ref_resolver.enter_scope(element_id);
    }
    
    /// Exit the current element scope.
    fn exit_element_scope(&mut self) {
        self.current_element_stack.pop();
        self.cross_ref_resolver.exit_scope();
    }
    
    /// Process imports and populate namespace resolver.
    pub fn process_imports(&mut self, imports: &[crate::ast_data::ImportInfo]) {
        for import in imports {
            self.namespace_resolver.add_use_statement(import);
        }
    }
    
    /// Update namespace information for all elements.
    fn update_namespace_information(&mut self) {
        for element in &mut self.elements {
            // Add import aliases to namespace
            let canonical_path = &element.hierarchy.namespace.canonical_path;
            let aliases = self.namespace_resolver.get_aliases_for_path(canonical_path);
            
            for alias in aliases {
                element.hierarchy.namespace.add_alias(alias);
            }
            
            // Register element in cross-reference resolver with all its names
            for name in element.hierarchy.namespace.get_reference_names() {
                self.cross_ref_resolver.register_element(name, element.id.clone());
            }
        }
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

        let element_name = node.sig.ident.to_string();
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Function, &element_name);
        let visibility = self.get_visibility(&node.vis);
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Function, &element_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Function,
            name: element_name.clone(),
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
            hierarchy,
        };
        
        // Register the element for cross-reference resolution
        self.register_element(&element_name, &element_id);
        
        // For functions with bodies, we may want to enter scope for nested items
        self.hierarchy_builder.enter_scope(element_id.clone());
        self.enter_element_scope(element_id.clone());

        self.elements.push(element);

        // Continue visiting nested items
        syn::visit::visit_item_fn(self, node);
        
        // Exit scope after visiting function body
        self.exit_element_scope();
        self.hierarchy_builder.exit_scope();
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

        let element_name = node.ident.to_string();
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Struct, &element_name);
        let visibility = self.get_visibility(&node.vis);
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Struct, &element_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Struct,
            name: element_name.clone(),
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
            hierarchy,
        };
        
        // For structs, we may want to enter scope for impl blocks
        self.hierarchy_builder.enter_scope(element_id.clone());

        self.elements.push(element);
        syn::visit::visit_item_struct(self, node);
        
        // Exit scope after visiting struct
        self.hierarchy_builder.exit_scope();
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

        let element_name = node.ident.to_string();
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Enum, &element_name);
        let visibility = self.get_visibility(&node.vis);
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Enum, &element_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Enum,
            name: element_name.clone(),
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
            hierarchy,
        };
        
        // For enums, we may want to enter scope for variant methods
        self.hierarchy_builder.enter_scope(element_id.clone());

        self.elements.push(element);
        syn::visit::visit_item_enum(self, node);
        
        // Exit scope after visiting enum
        self.hierarchy_builder.exit_scope();
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

        let element_name = node.ident.to_string();
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Trait, &element_name);
        let visibility = self.get_visibility(&node.vis);
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Trait, &element_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Trait,
            name: element_name.clone(),
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
            hierarchy,
        };
        
        // For traits, we may want to enter scope for trait methods
        self.hierarchy_builder.enter_scope(element_id.clone());

        self.elements.push(element);
        syn::visit::visit_item_trait(self, node);
        
        // Exit scope after visiting trait
        self.hierarchy_builder.exit_scope();
    }
    
    /// Visit impl blocks and extract information.
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        // Get the type being implemented
        let impl_name = if let Some((_bang, trait_path, _for_token)) = &node.trait_ {
            format!("{} for {}", quote::quote!(#trait_path), quote::quote!(#node.self_ty))
        } else {
            format!("impl {}", quote::quote!(#node.self_ty))
        };
        
        let docs = self.extract_doc_comments(&node.attrs);
        let item = syn::Item::Impl(node.clone());
        let complexity_metrics = ComplexityCalculator::calculate_structural_complexity(&item);
        let complexity_score = complexity_metrics.overall_score();
        
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Impl, &impl_name);
        let visibility = Visibility::Public; // Impl blocks don't have visibility modifiers
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Impl, &impl_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Impl,
            name: impl_name,
            signature: None,
            visibility: Visibility::Public, // Impl blocks don't have visibility modifiers
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.impl_token.span),
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
            hierarchy,
        };
        
        self.hierarchy_builder.enter_scope(element_id.clone());
        self.elements.push(element);
        syn::visit::visit_item_impl(self, node);
        
        // Exit scope after visiting impl block
        self.hierarchy_builder.exit_scope();
    }
    
    /// Visit module items and extract information.
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if !self.config.include_private
            && matches!(self.get_visibility(&node.vis), Visibility::Private)
        {
            return;
        }
        
        let module_name = node.ident.to_string();
        let docs = self.extract_doc_comments(&node.attrs);
        
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Module, &module_name);
        let visibility = self.get_visibility(&node.vis);
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Module, &module_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Module,
            name: module_name.clone(),
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
            complexity: Some(1), // Modules have base complexity
            complexity_metrics: None,
            dependencies: vec![],
            generic_params: vec![],
            metadata: std::collections::HashMap::new(),
            hierarchy,
        };
        
        // Enter module scope
        self.hierarchy_builder.enter_module(&module_name);
        self.hierarchy_builder.enter_scope(element_id.clone());
        self.elements.push(element);
        
        syn::visit::visit_item_mod(self, node);
        
        // Exit module scope
        self.hierarchy_builder.exit_scope();
        self.hierarchy_builder.exit_module();
    }
    
    /// Visit impl item functions (methods).
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if !self.config.include_private
            && matches!(self.get_visibility(&node.vis), Visibility::Private)
        {
            return;
        }
        
        let signature = format!("{}", quote::quote!(#node.sig));
        let docs = self.extract_doc_comments(&node.attrs);
        let complexity_metrics = ComplexityCalculator::calculate_method_complexity(node);
        let complexity_score = complexity_metrics.overall_score();
        
        let element_name = node.sig.ident.to_string();
        let element_id = self.hierarchy_builder.generate_id(&ElementType::Function, &element_name);
        let visibility = self.get_visibility(&node.vis);
        let hierarchy = self.hierarchy_builder.build_hierarchy(&ElementType::Function, &element_name, &visibility);
        
        let element = CodeElement {
            id: element_id.clone(),
            element_type: ElementType::Function,
            name: element_name.clone(),
            signature: Some(signature),
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.sig.ident.span()),
            attributes: node
                .attrs
                .iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(complexity_score),
            complexity_metrics: Some(complexity_metrics),
            dependencies: vec![],
            generic_params: node
                .sig
                .generics
                .params
                .iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
            hierarchy,
        };
        
        self.hierarchy_builder.enter_scope(element_id.clone());
        self.elements.push(element);
        syn::visit::visit_impl_item_fn(self, node);
        
        // Exit scope after visiting method
        self.hierarchy_builder.exit_scope();
    }
    
    /// Visit expressions to track cross-references.
    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        match expr {
            // Function calls
            syn::Expr::Call(call_expr) => {
                if let syn::Expr::Path(path_expr) = &*call_expr.func {
                    if let Some(ident) = path_expr.path.get_ident() {
                        self.track_reference(
                            crate::ast_data::ReferenceType::FunctionCall,
                            ident.to_string(),
                            ident.span(),
                        );
                    } else {
                        // Handle qualified function calls like std::println!
                        let path_str = format!("{}", quote::quote!(#path_expr.path));
                        self.track_reference(
                            crate::ast_data::ReferenceType::FunctionCall,
                            path_str,
                            path_expr.path.span(),
                        );
                    }
                }
            }
            
            // Method calls
            syn::Expr::MethodCall(method_call) => {
                self.track_reference(
                    crate::ast_data::ReferenceType::FunctionCall,
                    method_call.method.to_string(),
                    method_call.method.span(),
                );
            }
            
            // Path expressions (variable access, type usage)
            syn::Expr::Path(path_expr) => {
                if let Some(ident) = path_expr.path.get_ident() {
                    self.track_reference(
                        crate::ast_data::ReferenceType::VariableAccess,
                        ident.to_string(),
                        ident.span(),
                    );
                }
            }
            
            // Macro invocations
            syn::Expr::Macro(macro_expr) => {
                if let Some(ident) = macro_expr.mac.path.get_ident() {
                    self.track_reference(
                        crate::ast_data::ReferenceType::MacroInvocation,
                        ident.to_string(),
                        ident.span(),
                    );
                }
            }
            
            _ => {}
        }
        
        // Continue visiting nested expressions
        syn::visit::visit_expr(self, expr);
    }
    
    /// Visit types to track type usage references.
    fn visit_type(&mut self, type_expr: &'ast syn::Type) {
        if let syn::Type::Path(type_path) = type_expr {
            if let Some(ident) = type_path.path.get_ident() {
                self.track_reference(
                    crate::ast_data::ReferenceType::TypeUsage,
                    ident.to_string(),
                    ident.span(),
                );
            } else {
                // Handle qualified type paths
                let path_str = format!("{}", quote::quote!(#type_path.path));
                self.track_reference(
                    crate::ast_data::ReferenceType::TypeUsage,
                    path_str,
                    type_path.path.span(),
                );
            }
        }
        
        // Continue visiting nested types
        syn::visit::visit_type(self, type_expr);
    }
}
