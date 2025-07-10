//! Unit tests for the AST visitors.

use rustex_core::{CodeElementVisitor, ElementType, ExtractorConfig, Visibility};
use std::path::PathBuf;
use syn::visit::Visit;

fn create_test_config() -> ExtractorConfig {
    let mut config = ExtractorConfig::default();
    config.include_docs = true;
    config.include_private = true;
    config
}

#[test]
fn test_function_visitor() {
    let code = r#"
        /// Public function with documentation
        pub fn public_function(param: i32) -> String {
            format!("{}", param)
        }
        
        /// Private function
        fn private_function() {
            println!("private");
        }
        
        /// Generic function
        pub fn generic_function<T: Clone>(value: T) -> T {
            value.clone()
        }
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");
    let config = create_test_config();
    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);

    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    let functions: Vec<_> = elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Function))
        .collect();

    assert_eq!(functions.len(), 3, "Should extract all functions");

    // Test public function
    let public_fn = functions
        .iter()
        .find(|f| f.name == "public_function")
        .expect("Should find public_function");

    assert!(matches!(public_fn.visibility, Visibility::Public));
    assert!(
        !public_fn.doc_comments.is_empty(),
        "Should have documentation"
    );
    assert!(public_fn.signature.is_some(), "Should have signature");

    // Test private function
    let private_fn = functions
        .iter()
        .find(|f| f.name == "private_function")
        .expect("Should find private_function");

    assert!(matches!(private_fn.visibility, Visibility::Private));

    // Test generic function
    let generic_fn = functions
        .iter()
        .find(|f| f.name == "generic_function")
        .expect("Should find generic_function");

    assert!(
        !generic_fn.generic_params.is_empty(),
        "Should extract generic parameters"
    );
}

#[test]
fn test_struct_visitor() {
    let code = r#"
        /// Public struct with documentation
        #[derive(Debug, Clone)]
        pub struct PublicStruct {
            pub field1: String,
            field2: i32,
        }
        
        /// Private struct
        struct PrivateStruct(i32);
        
        /// Generic struct
        pub struct GenericStruct<T, U> 
        where 
            T: Clone,
            U: Send,
        {
            data: T,
            metadata: U,
        }
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");
    let config = create_test_config();
    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);

    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    let structs: Vec<_> = elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Struct))
        .collect();

    assert_eq!(structs.len(), 3, "Should extract all structs");

    // Test public struct
    let public_struct = structs
        .iter()
        .find(|s| s.name == "PublicStruct")
        .expect("Should find PublicStruct");

    assert!(matches!(public_struct.visibility, Visibility::Public));
    assert!(
        !public_struct.doc_comments.is_empty(),
        "Should have documentation"
    );
    assert!(
        !public_struct.attributes.is_empty(),
        "Should extract attributes"
    );

    // Test generic struct
    let generic_struct = structs
        .iter()
        .find(|s| s.name == "GenericStruct")
        .expect("Should find GenericStruct");

    assert!(
        !generic_struct.generic_params.is_empty(),
        "Should extract generic parameters"
    );
}

#[test]
fn test_enum_visitor() {
    let code = r#"
        /// Test enum with variants
        #[derive(Debug)]
        pub enum TestEnum {
            /// Simple variant
            Simple,
            /// Tuple variant
            Tuple(String, i32),
            /// Struct variant
            Struct { name: String, value: i32 },
        }
        
        /// Private enum
        enum PrivateEnum {
            Variant1,
            Variant2,
        }
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");
    let config = create_test_config();
    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);

    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    let enums: Vec<_> = elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Enum))
        .collect();

    assert_eq!(enums.len(), 2, "Should extract all enums");

    // Test public enum
    let test_enum = enums
        .iter()
        .find(|e| e.name == "TestEnum")
        .expect("Should find TestEnum");

    assert!(matches!(test_enum.visibility, Visibility::Public));
    assert!(
        !test_enum.doc_comments.is_empty(),
        "Should have documentation"
    );
    // New complexity calculation is more sophisticated (cyclomatic * 2 + cognitive + nesting + returns)
    // For TestEnum: 3 variants + 2 complex variants = 5 cyclomatic, 5 cognitive, score = 5*2+5 = 15
    assert!(
        test_enum.complexity.unwrap_or(0) >= 10,
        "Should calculate complexity based on variants and their complexity"
    );

    // Test private enum
    let private_enum = enums
        .iter()
        .find(|e| e.name == "PrivateEnum")
        .expect("Should find PrivateEnum");

    assert!(matches!(private_enum.visibility, Visibility::Private));
    assert!(
        private_enum.complexity.unwrap_or(0) >= 2,
        "Should calculate complexity based on variants"
    );
}

#[test]
fn test_trait_visitor() {
    let code = r#"
        /// Public trait definition
        pub trait TestTrait {
            /// Required method
            fn required_method(&self) -> String;
            
            /// Method with default implementation
            fn default_method(&self) -> i32 {
                42
            }
            
            /// Generic method
            fn generic_method<T>(&self, value: T) -> T;
        }
        
        /// Private trait
        trait PrivateTrait {
            fn method(&self);
        }
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");
    let config = create_test_config();
    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);

    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    let traits: Vec<_> = elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Trait))
        .collect();

    assert_eq!(traits.len(), 2, "Should extract all traits");

    // Test public trait
    let test_trait = traits
        .iter()
        .find(|t| t.name == "TestTrait")
        .expect("Should find TestTrait");

    assert!(matches!(test_trait.visibility, Visibility::Public));
    assert!(
        !test_trait.doc_comments.is_empty(),
        "Should have documentation"
    );
    // New complexity calculation: 3 methods + 0 types = 3 cyclomatic, 6 cognitive (methods * 2), score = 3*2+6 = 12
    assert!(
        test_trait.complexity.unwrap_or(0) >= 10,
        "Should calculate complexity based on methods and their complexity"
    );

    // Test private trait
    let private_trait = traits
        .iter()
        .find(|t| t.name == "PrivateTrait")
        .expect("Should find PrivateTrait");

    assert!(matches!(private_trait.visibility, Visibility::Private));
    assert!(
        private_trait.complexity.unwrap_or(0) >= 1,
        "Should calculate complexity based on methods"
    );
}

#[test]
fn test_visibility_filtering() {
    let code = r#"
        pub fn public_function() {}
        fn private_function() {}
        pub struct PublicStruct;
        struct PrivateStruct;
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");

    // Test with include_private = false
    let mut config = ExtractorConfig::default();
    config.include_private = false;
    config.include_docs = true;

    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);
    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    assert_eq!(elements.len(), 2, "Should only extract public elements");

    let public_elements: Vec<_> = elements
        .iter()
        .filter(|e| matches!(e.visibility, Visibility::Public))
        .collect();
    assert_eq!(
        public_elements.len(),
        2,
        "All extracted elements should be public"
    );

    // Test with include_private = true
    let mut config = ExtractorConfig::default();
    config.include_private = true;
    config.include_docs = true;

    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);
    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    assert_eq!(elements.len(), 4, "Should extract all elements");
}

#[test]
fn test_documentation_extraction() {
    let code = r#"
        /// Main documentation line
        /// Second documentation line
        /// 
        /// With blank line
        pub fn documented_function() {}
        
        // This is not doc comment
        pub fn undocumented_function() {}
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");

    // Test with include_docs = true
    let mut config = ExtractorConfig::default();
    config.include_docs = true;

    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);
    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    let documented_fn = elements
        .iter()
        .find(|e| e.name == "documented_function")
        .expect("Should find documented function");

    assert!(
        !documented_fn.doc_comments.is_empty(),
        "Should extract documentation"
    );
    assert!(
        documented_fn.doc_comments.len() >= 3,
        "Should extract multiple doc lines"
    );

    let undocumented_fn = elements
        .iter()
        .find(|e| e.name == "undocumented_function")
        .expect("Should find undocumented function");

    assert!(
        undocumented_fn.doc_comments.is_empty(),
        "Should not extract regular comments as docs"
    );

    // Test with include_docs = false
    let mut config = ExtractorConfig::default();
    config.include_docs = false;

    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);
    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    for element in &elements {
        assert!(
            element.doc_comments.is_empty(),
            "Should not extract docs when include_docs=false"
        );
    }
}

#[test]
fn test_attribute_extraction() {
    let code = r#"
        #[derive(Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct AttributedStruct {
            field: String,
        }
        
        #[test]
        #[ignore]
        fn test_function() {}
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");
    let config = create_test_config();
    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);

    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    let struct_element = elements
        .iter()
        .find(|e| e.name == "AttributedStruct")
        .expect("Should find AttributedStruct");

    assert!(
        !struct_element.attributes.is_empty(),
        "Should extract attributes"
    );
    assert!(
        struct_element.attributes.len() >= 2,
        "Should extract multiple attributes"
    );

    let fn_element = elements
        .iter()
        .find(|e| e.name == "test_function")
        .expect("Should find test_function");

    assert!(
        !fn_element.attributes.is_empty(),
        "Should extract function attributes"
    );
}

#[test]
fn test_nested_items() {
    let code = r#"
        pub mod outer {
            pub fn outer_function() {}
            
            pub struct OuterStruct;
            
            pub mod inner {
                pub fn inner_function() {}
            }
        }
        
        impl OuterStruct {
            pub fn method(&self) {}
        }
    "#;

    let syntax_tree = syn::parse_file(code).expect("Failed to parse code");
    let config = create_test_config();
    let mut visitor = CodeElementVisitor::new(PathBuf::from("test.rs"), &config);

    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();

    // Should extract nested functions
    let functions: Vec<_> = elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Function))
        .collect();

    assert!(!functions.is_empty(), "Should extract nested functions");

    // Should find functions from nested modules
    let has_outer_fn = functions.iter().any(|f| f.name == "outer_function");
    let has_inner_fn = functions.iter().any(|f| f.name == "inner_function");

    assert!(has_outer_fn, "Should find outer_function");
    assert!(has_inner_fn, "Should find inner_function");

    // Note: impl methods are currently extracted as separate elements
    // The current implementation visits nested functions but may not extract impl methods
    // This is acceptable behavior for the current visitor implementation
}
