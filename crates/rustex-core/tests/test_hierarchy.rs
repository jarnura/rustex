//! Tests for hierarchical AST structure.

use rustex_core::*;
use std::path::PathBuf;
use syn::visit::Visit;

#[test]
fn test_hierarchical_structure() {
    let code = r#"
        pub mod my_module {
            pub struct MyStruct {
                field: i32,
            }
            
            impl MyStruct {
                pub fn new() -> Self {
                    Self { field: 0 }
                }
                
                pub fn get_field(&self) -> i32 {
                    self.field
                }
            }
            
            pub fn module_function() {
                println!("Module function");
            }
        }
        
        pub fn top_level_function() {
            println!("Top level");
        }
    "#;
    
    // Parse the code
    let syntax_tree = syn::parse_file(code).expect("Failed to parse test code");
    
    // Create visitor
    let config = ExtractorConfig::default();
    let file_path = PathBuf::from("test_hierarchy.rs");
    let mut visitor = CodeElementVisitor::new(file_path, &config);
    
    // Visit the AST
    visitor.visit_file(&syntax_tree);
    let elements = visitor.into_elements();
    
    // Verify we have the expected elements
    assert!(!elements.is_empty(), "Should have extracted elements");
    
    // Find the module
    let module = elements.iter().find(|e| e.element_type == ElementType::Module && e.name == "my_module")
        .expect("Should find my_module");
    
    // Find elements that should be children of the module
    let struct_element = elements.iter().find(|e| e.element_type == ElementType::Struct && e.name == "MyStruct")
        .expect("Should find MyStruct");
    
    let impl_element = elements.iter().find(|e| e.element_type == ElementType::Impl)
        .expect("Should find impl block");
    
    let module_function = elements.iter().find(|e| e.element_type == ElementType::Function && e.name == "module_function")
        .expect("Should find module_function");
    
    // Find methods that should be children of impl
    let new_method = elements.iter().find(|e| e.element_type == ElementType::Function && e.name == "new")
        .expect("Should find new method");
    
    let get_field_method = elements.iter().find(|e| e.element_type == ElementType::Function && e.name == "get_field")
        .expect("Should find get_field method");
    
    // Find top-level function
    let top_level_function = elements.iter().find(|e| e.element_type == ElementType::Function && e.name == "top_level_function")
        .expect("Should find top_level_function");
    
    // Verify hierarchy relationships
    
    // Module should be top-level (no parent)
    assert!(module.hierarchy.parent_id.is_none(), "Module should have no parent");
    assert_eq!(module.hierarchy.nesting_level, 0, "Module should be at nesting level 0");
    
    // Top-level function should also be top-level
    assert!(top_level_function.hierarchy.parent_id.is_none(), "Top-level function should have no parent");
    assert_eq!(top_level_function.hierarchy.nesting_level, 0, "Top-level function should be at nesting level 0");
    
    // Struct should be child of module
    assert_eq!(struct_element.hierarchy.parent_id, Some(module.id.clone()), "Struct should be child of module");
    assert_eq!(struct_element.hierarchy.nesting_level, 1, "Struct should be at nesting level 1");
    
    // Impl should be child of module
    assert_eq!(impl_element.hierarchy.parent_id, Some(module.id.clone()), "Impl should be child of module");
    assert_eq!(impl_element.hierarchy.nesting_level, 1, "Impl should be at nesting level 1");
    
    // Module function should be child of module
    assert_eq!(module_function.hierarchy.parent_id, Some(module.id.clone()), "Module function should be child of module");
    assert_eq!(module_function.hierarchy.nesting_level, 1, "Module function should be at nesting level 1");
    
    // Methods should be children of impl
    assert_eq!(new_method.hierarchy.parent_id, Some(impl_element.id.clone()), "new method should be child of impl");
    assert_eq!(new_method.hierarchy.nesting_level, 2, "new method should be at nesting level 2");
    
    assert_eq!(get_field_method.hierarchy.parent_id, Some(impl_element.id.clone()), "get_field method should be child of impl");
    assert_eq!(get_field_method.hierarchy.nesting_level, 2, "get_field method should be at nesting level 2");
    
    // Check qualified names
    assert!(module.hierarchy.qualified_name.contains("my_module"), "Module qualified name should contain module name");
    assert!(struct_element.hierarchy.qualified_name.contains("MyStruct"), "Struct qualified name should contain struct name");
    assert!(new_method.hierarchy.qualified_name.contains("new"), "Method qualified name should contain method name");
    
    // Verify parent-child relationships are bidirectional
    let module_children = &module.hierarchy.children_ids;
    assert!(module_children.contains(&struct_element.id), "Module should list struct as child");
    assert!(module_children.contains(&impl_element.id), "Module should list impl as child");
    assert!(module_children.contains(&module_function.id), "Module should list function as child");
    
    let impl_children = &impl_element.hierarchy.children_ids;
    assert!(impl_children.contains(&new_method.id), "Impl should list new method as child");
    assert!(impl_children.contains(&get_field_method.id), "Impl should list get_field method as child");
    
    println!("✅ Hierarchical structure test passed!");
    println!("Found {} elements with proper parent-child relationships", elements.len());
}

#[test]  
fn test_hierarchy_builder() {
    let mut builder = HierarchyBuilder::new("crate::test".to_string());
    
    // Test basic ID generation
    let id1 = builder.generate_id(&ElementType::Function, "test_func");
    let id2 = builder.generate_id(&ElementType::Function, "test_func");
    assert_ne!(id1, id2, "Generated IDs should be unique");
    
    // Test hierarchy building without parent
    let hierarchy = builder.build_hierarchy(&ElementType::Function, "top_level", &rustex_core::Visibility::Public);
    assert!(hierarchy.parent_id.is_none(), "Top level should have no parent");
    assert_eq!(hierarchy.nesting_level, 0, "Top level should be at level 0");
    
    // Test entering scope
    builder.enter_scope("parent_id".to_string());
    let child_hierarchy = builder.build_hierarchy(&ElementType::Function, "child", &rustex_core::Visibility::Public);
    assert_eq!(child_hierarchy.parent_id, Some("parent_id".to_string()), "Child should have parent");
    assert_eq!(child_hierarchy.nesting_level, 1, "Child should be at level 1");
    
    // Test module path tracking
    builder.enter_module("submodule");
    let module_hierarchy = builder.build_hierarchy(&ElementType::Function, "module_func", &rustex_core::Visibility::Public);
    assert!(module_hierarchy.module_path.contains("submodule"), "Module path should include submodule");
    
    builder.exit_module();
    let back_hierarchy = builder.build_hierarchy(&ElementType::Function, "back_func", &rustex_core::Visibility::Public);
    assert!(!back_hierarchy.module_path.contains("submodule"), "Module path should not include submodule after exit");
    
    println!("✅ Hierarchy builder test passed!");
}