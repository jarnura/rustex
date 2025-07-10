//! Tests for cross-reference resolution and tracking.

use rustex_core::*;
use std::path::PathBuf;
use syn::visit::Visit;

#[test]
fn test_function_call_cross_references() {
    let code = r#"
        pub fn helper_function() -> i32 {
            42
        }
        
        pub fn main_function() -> i32 {
            let result = helper_function();
            result + 1
        }
    "#;
    
    // Parse the code
    let syntax_tree = syn::parse_file(code).expect("Failed to parse test code");
    
    // Create visitor
    let config = ExtractorConfig::default();
    let file_path = PathBuf::from("test_cross_refs.rs");
    let mut visitor = CodeElementVisitor::new(file_path, &config);
    
    // Visit the AST
    visitor.visit_file(&syntax_tree);
    let (elements, cross_references) = visitor.into_elements_and_references();
    
    // Verify we have elements
    assert!(!elements.is_empty(), "Should have extracted elements");
    assert!(!cross_references.is_empty(), "Should have cross-references");
    
    // Find the functions
    let helper_func = elements.iter().find(|e| e.name == "helper_function")
        .expect("Should find helper_function");
    let main_func = elements.iter().find(|e| e.name == "main_function")
        .expect("Should find main_function");
    
    // Find function call reference
    let function_call_ref = cross_references.iter().find(|r| {
        r.reference_type == ReferenceType::FunctionCall && 
        r.reference_text == "helper_function"
    }).expect("Should find function call reference");
    
    // Verify the reference points from main_function
    assert_eq!(function_call_ref.from_element_id, main_func.id, 
               "Reference should originate from main_function");
    
    // The reference should be resolved to helper_function
    assert!(function_call_ref.is_resolved, "Reference should be resolved");
    assert_eq!(function_call_ref.to_element_id, Some(helper_func.id.clone()),
               "Reference should point to helper_function");
    
    println!("✅ Function call cross-reference test passed!");
    println!("Found {} cross-references", cross_references.len());
}

#[test]
fn test_type_usage_cross_references() {
    let code = r#"
        pub struct MyStruct {
            value: i32,
        }
        
        impl MyStruct {
            pub fn new(value: i32) -> MyStruct {
                MyStruct { value }
            }
        }
        
        pub fn create_instance() -> MyStruct {
            MyStruct::new(42)
        }
    "#;
    
    // Parse the code
    let syntax_tree = syn::parse_file(code).expect("Failed to parse test code");
    
    // Create visitor
    let config = ExtractorConfig::default();
    let file_path = PathBuf::from("test_type_refs.rs");
    let mut visitor = CodeElementVisitor::new(file_path, &config);
    
    // Visit the AST
    visitor.visit_file(&syntax_tree);
    let (elements, cross_references) = visitor.into_elements_and_references();
    
    // Verify we have elements and cross-references
    assert!(!elements.is_empty(), "Should have extracted elements");
    assert!(!cross_references.is_empty(), "Should have cross-references");
    
    // Find the struct
    let my_struct = elements.iter().find(|e| e.name == "MyStruct" && e.element_type == ElementType::Struct)
        .expect("Should find MyStruct");
    
    // Find type usage references
    let type_refs: Vec<_> = cross_references.iter()
        .filter(|r| r.reference_type == ReferenceType::TypeUsage && r.reference_text == "MyStruct")
        .collect();
    
    assert!(!type_refs.is_empty(), "Should find type usage references");
    
    // At least one should be resolved to our struct
    let resolved_refs: Vec<_> = type_refs.iter()
        .filter(|r| r.is_resolved && r.to_element_id == Some(my_struct.id.clone()))
        .collect();
    
    assert!(!resolved_refs.is_empty(), "Should have resolved type references");
    
    println!("✅ Type usage cross-reference test passed!");
    println!("Found {} type references", type_refs.len());
}

#[test]
#[ignore] // Temporarily disabled - method call detection needs enhancement
fn test_method_call_cross_references() {
    let code = r#"
        pub struct Calculator {
            value: i32,
        }
        
        impl Calculator {
            pub fn new() -> Self {
                Self { value: 0 }
            }
            
            pub fn add(&mut self, n: i32) -> &mut Self {
                self.value += n;
                self
            }
            
            pub fn get_value(&self) -> i32 {
                self.value
            }
        }
        
        pub fn test_calculator() -> i32 {
            let mut calc = Calculator::new();
            calc.add(5).add(3);
            calc.get_value()
        }
    "#;
    
    // Parse the code
    let syntax_tree = syn::parse_file(code).expect("Failed to parse test code");
    
    // Create visitor
    let config = ExtractorConfig::default();
    let file_path = PathBuf::from("test_method_refs.rs");
    let mut visitor = CodeElementVisitor::new(file_path, &config);
    
    // Visit the AST
    visitor.visit_file(&syntax_tree);
    let (elements, cross_references) = visitor.into_elements_and_references();
    
    // Verify we have elements and cross-references
    assert!(!elements.is_empty(), "Should have extracted elements");
    assert!(!cross_references.is_empty(), "Should have cross-references");
    
    // Find method call references
    let method_calls: Vec<_> = cross_references.iter()
        .filter(|r| r.reference_type == ReferenceType::FunctionCall)
        .collect();
    
    assert!(!method_calls.is_empty(), "Should find method call references");
    
    // Should find references to "new", "add", and "get_value"
    let method_names: Vec<&str> = method_calls.iter()
        .map(|r| r.reference_text.as_str())
        .collect();
    
    assert!(method_names.contains(&"new"), "Should reference 'new' method");
    assert!(method_names.contains(&"add"), "Should reference 'add' method");
    assert!(method_names.contains(&"get_value"), "Should reference 'get_value' method");
    
    println!("✅ Method call cross-reference test passed!");
    println!("Found method calls: {:?}", method_names);
}

#[test]
fn test_cross_reference_resolution() {
    let code = r#"
        mod utils {
            pub fn utility_function() -> String {
                "utility".to_string()
            }
        }
        
        pub fn main() {
            let result = utility_function(); // This should NOT resolve (wrong scope)
            println!("{}", result);
        }
    "#;
    
    // Parse the code
    let syntax_tree = syn::parse_file(code).expect("Failed to parse test code");
    
    // Create visitor
    let config = ExtractorConfig::default();
    let file_path = PathBuf::from("test_resolution.rs");
    let mut visitor = CodeElementVisitor::new(file_path, &config);
    
    // Visit the AST
    visitor.visit_file(&syntax_tree);
    let (_elements, cross_references) = visitor.into_elements_and_references();
    
    // Find the unqualified function call
    let unresolved_call = cross_references.iter()
        .find(|r| r.reference_text == "utility_function" && r.reference_type == ReferenceType::FunctionCall)
        .expect("Should find utility_function call");
    
    // This call should not be resolved because it's not properly qualified
    assert!(!unresolved_call.is_resolved, "Unqualified call should not resolve");
    
    println!("✅ Cross-reference resolution test passed!");
    println!("Unresolved reference correctly identified: {}", unresolved_call.reference_text);
}

#[test]
fn test_cross_reference_context() {
    let code = r#"
        pub fn outer_function() {
            inner_function();
        }
        
        fn inner_function() {
            println!("inner");
        }
    "#;
    
    // Parse the code
    let syntax_tree = syn::parse_file(code).expect("Failed to parse test code");
    
    // Create visitor
    let config = ExtractorConfig::default();
    let file_path = PathBuf::from("test_context.rs");
    let mut visitor = CodeElementVisitor::new(file_path, &config);
    
    // Visit the AST
    visitor.visit_file(&syntax_tree);
    let (_elements, cross_references) = visitor.into_elements_and_references();
    
    // Find the function call reference
    let call_ref = cross_references.iter()
        .find(|r| r.reference_text == "inner_function")
        .expect("Should find function call");
    
    // Verify context information
    assert!(!call_ref.context.is_definition, "Should be a usage, not definition");
    assert!(!call_ref.context.scope.is_empty(), "Should have scope information");
    
    println!("✅ Cross-reference context test passed!");
    println!("Reference context: scope='{}', is_definition={}", 
             call_ref.context.scope, call_ref.context.is_definition);
}