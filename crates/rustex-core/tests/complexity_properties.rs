//! Property-based testing specifically for complexity calculation algorithms.
//!
//! This module contains targeted property tests for complexity metrics
//! to ensure they behave correctly across a wide range of code patterns.

use proptest::prelude::*;
use rustex_core::complexity::*;
use syn::{self, Item};

/// Strategy for generating function-like Rust code
fn function_code() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple function
        Just(r"fn simple() -> i32 { 42 }".to_string()),
        // Function with parameters
        Just(r"fn with_params(a: i32, b: String) -> bool { a > 0 }".to_string()),
        // Function with if statement
        Just(r"fn with_if(x: i32) -> i32 { if x > 0 { x } else { -x } }".to_string()),
        // Function with loop
        Just(r#"fn with_loop() -> i32 { for i in 0..10 { println!("{}", i); } 42 }"#.to_string()),
        // Function with match
        Just(r"fn with_match(x: Option<i32>) -> i32 { match x { Some(v) => v, None => 0 } }".to_string()),
        // Nested control flow
        Just(r#"fn nested(x: i32) -> i32 { 
            if x > 0 { 
                for i in 0..x { 
                    if i % 2 == 0 { 
                        println!("{}", i); 
                    } 
                } 
                x 
            } else { 
                0 
            } 
        }"#.to_string()),
        // Function with multiple returns
        Just(r"fn multi_return(x: i32) -> i32 {
            if x < 0 { return -1; }
            if x == 0 { return 0; }
            if x > 100 { return 100; }
            x
        }".to_string()),
        // Async function
        Just(r#"async fn async_func() -> Result<i32, String> {
            if true { Ok(42) } else { Err("error".to_string()) }
        }"#.to_string()),
    ]
}

/// Strategy for generating various control flow patterns
fn control_flow_pattern() -> impl Strategy<Value = String> {
    prop_oneof![
        // if-else chain
        (1..=5usize).prop_map(|count| {
            let mut code = String::from("fn if_chain(x: i32) -> i32 {\n");
            for i in 0..count {
                if i == 0 {
                    code.push_str(&format!("    if x == {} {{ {} }}\n", i, i));
                } else if i == count - 1 {
                    code.push_str(&format!("    else {{ {} }}\n", i));
                } else {
                    code.push_str(&format!("    else if x == {} {{ {} }}\n", i, i));
                }
            }
            code.push_str("}\n");
            code
        }),
        // nested loops
        (1..=3usize).prop_map(|depth| {
            let mut code = String::from("fn nested_loops() {\n");
            for i in 0..depth {
                code.push_str(&format!("{}for _ in 0..10 {{\n", "    ".repeat(i + 1)));
            }
            code.push_str(&format!("{}println!(\"deep\");\n", "    ".repeat(depth + 1)));
            for i in (0..depth).rev() {
                code.push_str(&format!("{}}}\n", "    ".repeat(i + 1)));
            }
            code.push_str("}\n");
            code
        }),
        // match expressions
        (1..=5usize).prop_map(|arms| {
            let mut code = String::from("fn match_expr(x: i32) -> i32 {\n    match x {\n");
            for i in 0..arms {
                code.push_str(&format!("        {} => {},\n", i, i * 2));
            }
            code.push_str("        _ => 0,\n    }\n}\n");
            code
        }),
    ]
}

proptest! {
    /// Test that complexity metrics are consistent across different parsing attempts
    #[test]
    fn test_complexity_consistency(code in function_code()) {
        if let Ok(parsed) = syn::parse_file(&code) {
            let functions: Vec<_> = parsed.items.iter()
                .filter_map(|item| match item {
                    Item::Fn(func) => Some(func),
                    _ => None,
                })
                .collect();

            for func in functions {
                let metrics1 = ComplexityCalculator::calculate_function_complexity(func);
                let metrics2 = ComplexityCalculator::calculate_function_complexity(func);
                
                // Metrics should be deterministic
                prop_assert_eq!(metrics1.cyclomatic, metrics2.cyclomatic);
                prop_assert_eq!(metrics1.cognitive, metrics2.cognitive);
                prop_assert_eq!(metrics1.nesting_depth, metrics2.nesting_depth);
                prop_assert_eq!(metrics1.lines_of_code, metrics2.lines_of_code);
                prop_assert_eq!(metrics1.parameter_count, metrics2.parameter_count);
                prop_assert_eq!(metrics1.return_count, metrics2.return_count);
            }
        }
    }

    /// Test complexity bounds and invariants
    #[test]
    fn test_complexity_bounds(code in function_code()) {
        if let Ok(parsed) = syn::parse_file(&code) {
            for item in &parsed.items {
                if let Item::Fn(func) = item {
                    let metrics = ComplexityCalculator::calculate_function_complexity(func);
                    
                    // Basic bounds
                    prop_assert!(metrics.cyclomatic >= 1, "Cyclomatic complexity must be at least 1");
                    prop_assert!(metrics.lines_of_code > 0, "Function must have at least one line");
                    // Note: cognitive, nesting_depth, parameter_count, return_count are unsigned types so >= 0 is implicit
                    
                    // Logical relationships
                    prop_assert!(metrics.cognitive >= metrics.cyclomatic - 1, 
                               "Cognitive complexity should generally be >= cyclomatic - 1");
                    
                    // Halstead metrics bounds - these are unsigned types so >= 0 is implicit
                    prop_assert!(metrics.halstead.n1 <= metrics.halstead.big_n1);
                    prop_assert!(metrics.halstead.n2 <= metrics.halstead.big_n2);
                    
                    // Overall score should be reasonable
                    let overall = metrics.overall_score();
                    prop_assert!(overall >= 1, "Overall score should be at least 1");
                    prop_assert!(overall <= 1000, "Overall score should be reasonable (< 1000)");
                }
            }
        }
    }

    /// Test that more complex code generally has higher complexity scores
    #[test]
    fn test_complexity_ordering(pattern in control_flow_pattern()) {
        let simple_code = "fn simple() -> i32 { 42 }";
        
        if let (Ok(simple_parsed), Ok(complex_parsed)) = (
            syn::parse_file(simple_code),
            syn::parse_file(&pattern)
        ) {
            let simple_func = simple_parsed.items.iter()
                .find_map(|item| match item {
                    Item::Fn(func) => Some(func),
                    _ => None,
                });
            
            let complex_func = complex_parsed.items.iter()
                .find_map(|item| match item {
                    Item::Fn(func) => Some(func),
                    _ => None,
                });
            
            if let (Some(simple), Some(complex)) = (simple_func, complex_func) {
                let simple_metrics = ComplexityCalculator::calculate_function_complexity(simple);
                let complex_metrics = ComplexityCalculator::calculate_function_complexity(complex);
                
                // Complex code should generally have higher or equal complexity
                prop_assert!(complex_metrics.overall_score() >= simple_metrics.overall_score(),
                           "Complex pattern should have >= complexity than simple function");
            }
        }
    }

    /// Test complexity level categorization
    #[test]
    fn test_complexity_levels(code in function_code()) {
        if let Ok(parsed) = syn::parse_file(&code) {
            for item in &parsed.items {
                if let Item::Fn(func) = item {
                    let metrics = ComplexityCalculator::calculate_function_complexity(func);
                    let level = metrics.complexity_level();
                    let score = metrics.overall_score();
                    
                    // Verify level boundaries are consistent
                    match level {
                        ComplexityLevel::Low => prop_assert!(score <= 10),
                        ComplexityLevel::Medium => prop_assert!(score > 10 && score <= 20),
                        ComplexityLevel::High => prop_assert!(score > 20 && score <= 50),
                        ComplexityLevel::VeryHigh => prop_assert!(score > 50),
                    }
                }
            }
        }
    }

    /// Test Halstead metrics properties
    #[test]
    fn test_halstead_properties(code in function_code()) {
        if let Ok(parsed) = syn::parse_file(&code) {
            for item in &parsed.items {
                if let Item::Fn(func) = item {
                    let metrics = ComplexityCalculator::calculate_function_complexity(func);
                    let halstead = &metrics.halstead;
                    
                    // Basic Halstead properties
                    prop_assert!(halstead.vocabulary >= halstead.n1 + halstead.n2);
                    prop_assert!(halstead.length == halstead.big_n1 + halstead.big_n2);
                    
                    if halstead.vocabulary > 0 {
                        prop_assert!(halstead.volume >= 0.0);
                        prop_assert!(halstead.difficulty >= 0.0);
                        prop_assert!(halstead.effort >= 0.0);
                    }
                    
                    // Sanity checks for computed metrics
                    if halstead.n2 > 0 {
                        prop_assert!(halstead.difficulty >= 0.5); // Minimum difficulty
                    }
                }
            }
        }
    }
}

// Targeted tests for specific complexity scenarios

#[cfg(test)]
mod specific_complexity_tests {
    use super::*;

    #[test]
    fn test_if_else_complexity() {
        let code = r"
            fn if_else(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        20
                    } else {
                        10
                    }
                } else {
                    0
                }
            }
        ";
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Should have cyclomatic complexity of 4 (3 decision points + 1)
        assert_eq!(metrics.cyclomatic, 4);
        assert!(metrics.nesting_depth >= 2);
        assert!(metrics.cognitive > metrics.cyclomatic); // Nested conditions increase cognitive complexity
    }

    #[test]
    fn test_loop_complexity() {
        let code = r#"
            fn loops() {
                for i in 0..10 {
                    for j in 0..i {
                        if i % 2 == 0 {
                            println!("{}", j);
                        }
                    }
                }
            }
        "#;
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Should account for loops and nested if
        assert!(metrics.cyclomatic >= 3); // 2 loops + 1 if + 1 base
        assert!(metrics.nesting_depth >= 3); // for -> for -> if
        assert!(metrics.cognitive > 5); // High due to nesting
    }

    #[test]
    fn test_match_complexity() {
        let code = r"
            fn match_example(x: Option<Result<i32, String>>) -> i32 {
                match x {
                    Some(Ok(value)) if value > 0 => value,
                    Some(Ok(value)) => -value,
                    Some(Err(_)) => -1,
                    None => 0,
                }
            }
        ";
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Should account for match arms and guard
        assert!(metrics.cyclomatic >= 4); // Multiple match arms
        assert!(metrics.parameter_count == 1);
    }

    #[test]
    fn test_empty_function_complexity() {
        let code = "fn empty() {}";
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Empty function should have minimal complexity
        assert_eq!(metrics.cyclomatic, 1);
        assert_eq!(metrics.cognitive, 0);
        assert_eq!(metrics.nesting_depth, 0);
        assert_eq!(metrics.parameter_count, 0);
        assert_eq!(metrics.return_count, 0);
        assert!(metrics.lines_of_code >= 1);
    }

    #[test]
    fn test_function_with_many_parameters() {
        let code = r"
            fn many_params(
                a: i32, b: String, c: bool, d: Vec<i32>, 
                e: Option<String>, f: Result<i32, String>
            ) -> bool {
                a > 0 && !b.is_empty() && c && !d.is_empty()
            }
        ";
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        assert_eq!(metrics.parameter_count, 6);
        assert!(metrics.halstead.big_n2 > 0); // Many operands due to parameters
    }

    #[test]
    fn test_complexity_calculator_structural() {
        let code = r"
            struct TestStruct {
                field: i32,
            }
            
            impl TestStruct {
                fn method(&self) -> i32 {
                    self.field * 2
                }
            }
            
            fn standalone() -> i32 { 42 }
        ";
        
        let _parsed = syn::parse_file(code).unwrap();
        let structural_metrics = ComplexityCalculator::calculate_structural_complexity(&Item::Struct(
            syn::parse_str("struct TestStruct { field: i32, }").unwrap()
        ));
        
        // Structural complexity should be minimal for simple struct
        assert!(structural_metrics.overall_score() >= 1);
        assert_eq!(structural_metrics.cyclomatic, 1); // Base complexity
    }
}

// Edge case tests for complexity calculation

#[cfg(test)]
mod complexity_edge_cases {
    use super::*;

    #[test]
    fn test_deeply_nested_complexity() {
        // Generate deeply nested function
        let mut code = String::from("fn deeply_nested(x: i32) -> i32 {\n");
        let depth = 20;
        
        for i in 0..depth {
            code.push_str(&format!("{}if x > {} {{\n", "    ".repeat(i + 1), i));
        }
        
        code.push_str(&format!("{}return {};\n", "    ".repeat(depth + 1), depth));
        
        for i in (0..depth).rev() {
            code.push_str(&format!("{}}}\n", "    ".repeat(i + 1)));
        }
        code.push_str("    0\n}\n");
        
        let parsed = syn::parse_file(&code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Should handle deep nesting without overflow
        assert!(metrics.nesting_depth >= depth as u32);
        assert!(metrics.cyclomatic > depth as u32);
        assert!(metrics.cognitive > depth as u32 * 2); // Nesting penalty
        assert!(metrics.complexity_level() == ComplexityLevel::VeryHigh);
    }

    #[test]
    fn test_recursive_function_complexity() {
        let code = r"
            fn fibonacci(n: u32) -> u32 {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
        ";
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Recursive calls should be counted in operands
        assert!(metrics.halstead.big_n2 > 3);
        assert_eq!(metrics.cyclomatic, 2); // One decision point
        assert_eq!(metrics.parameter_count, 1);
    }

    #[test]
    fn test_macro_invocation_complexity() {
        let code = r#"
            fn with_macros() {
                println!("Hello");
                vec![1, 2, 3].iter().for_each(|x| {
                    if *x > 1 {
                        println!("{}", x);
                    }
                });
            }
        "#;
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Should handle macro invocations gracefully
        assert!(metrics.cyclomatic >= 1);
        assert!(metrics.halstead.big_n1 > 0);
    }

    #[test]
    fn test_generic_function_complexity() {
        let code = r#"
            fn generic_function<T, U>(
                input: T, 
                converter: impl Fn(T) -> U
            ) -> Option<U> 
            where 
                T: Clone + Send,
                U: std::fmt::Debug,
            {
                if std::mem::size_of::<T>() > 0 {
                    Some(converter(input))
                } else {
                    None
                }
            }
        "#;
        
        let parsed = syn::parse_file(code).unwrap();
        let func = parsed.items.iter()
            .find_map(|item| match item {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap();
        
        let metrics = ComplexityCalculator::calculate_function_complexity(func);
        
        // Generic functions should still have measurable complexity
        assert_eq!(metrics.parameter_count, 2);
        assert_eq!(metrics.cyclomatic, 2); // One if statement
        assert!(metrics.halstead.big_n2 > 0);
    }
}