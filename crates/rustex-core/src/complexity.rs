//! Code complexity analysis algorithms.
//!
//! This module provides various complexity metrics for Rust code, including:
//! - Cyclomatic complexity
//! - Cognitive complexity  
//! - Halstead complexity
//! - Nesting depth analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::{
    visit::{self, Visit},
    Block, Expr, Stmt,
};

/// Comprehensive complexity metrics for a code element.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity (traditional branch counting)
    pub cyclomatic: u32,
    /// Cognitive complexity (human readability focused)
    pub cognitive: u32,
    /// Halstead complexity metrics
    pub halstead: HalsteadMetrics,
    /// Maximum nesting depth
    pub nesting_depth: u32,
    /// Number of lines of code
    pub lines_of_code: u32,
    /// Parameter count
    pub parameter_count: u32,
    /// Return points count
    pub return_count: u32,
}

/// Halstead complexity metrics based on operators and operands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    /// Number of distinct operators
    pub n1: u32,
    /// Number of distinct operands  
    pub n2: u32,
    /// Total number of operators
    pub big_n1: u32,
    /// Total number of operands
    pub big_n2: u32,
    /// Program vocabulary (n1 + n2)
    pub vocabulary: u32,
    /// Program length (N1 + N2)
    pub length: u32,
    /// Calculated length (n1 * log2(n1) + n2 * log2(n2))
    pub calculated_length: f64,
    /// Volume (length * log2(vocabulary))
    pub volume: f64,
    /// Difficulty (n1/2 * N2/n2)
    pub difficulty: f64,
    /// Effort (difficulty * volume)
    pub effort: f64,
}

impl ComplexityMetrics {
    /// Calculate overall complexity score as weighted sum of different metrics.
    pub fn overall_score(&self) -> u32 {
        // Weighted combination: cyclomatic gets highest weight, cognitive and nesting also important
        (self.cyclomatic * 2 + self.cognitive + self.nesting_depth + self.return_count).max(1)
    }

    /// Determine complexity level based on overall score.
    pub fn complexity_level(&self) -> ComplexityLevel {
        match self.overall_score() {
            1..=10 => ComplexityLevel::Low,
            11..=20 => ComplexityLevel::Medium,
            21..=50 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }
}

/// Complexity level categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Complexity calculator that visits AST nodes to compute metrics.
#[derive(Debug)]
pub struct ComplexityCalculator {
    /// Current nesting depth
    nesting_depth: u32,
    /// Maximum nesting depth encountered
    max_nesting_depth: u32,
    /// Cyclomatic complexity counter
    cyclomatic: u32,
    /// Cognitive complexity counter  
    cognitive: u32,
    /// Cognitive complexity nesting increment
    cognitive_nesting: u32,
    /// Return statement counter
    return_count: u32,
    /// Halstead operators and operands
    halstead_operators: HashMap<String, u32>,
    halstead_operands: HashMap<String, u32>,
    /// Line count approximation
    lines_of_code: u32,
    /// Parameter count
    parameter_count: u32,
}

impl ComplexityCalculator {
    /// Create a new complexity calculator.
    pub fn new() -> Self {
        Self {
            nesting_depth: 0,
            max_nesting_depth: 0,
            cyclomatic: 1, // Base complexity starts at 1
            cognitive: 0,
            cognitive_nesting: 0,
            return_count: 0,
            halstead_operators: HashMap::new(),
            halstead_operands: HashMap::new(),
            lines_of_code: 0,
            parameter_count: 0,
        }
    }

    /// Calculate complexity for a function item.
    pub fn calculate_function_complexity(item_fn: &syn::ItemFn) -> ComplexityMetrics {
        let mut calc = Self::new();

        // Count parameters
        calc.parameter_count = item_fn.sig.inputs.len() as u32;

        // Calculate line span (simplified approach)
        calc.lines_of_code = 1; // Will be improved with proper span handling

        // Visit function body
        calc.visit_block(&item_fn.block);

        calc.finish()
    }

    /// Calculate complexity for a method in an impl block.
    pub fn calculate_method_complexity(method: &syn::ImplItemFn) -> ComplexityMetrics {
        let mut calc = Self::new();

        // Count parameters
        calc.parameter_count = method.sig.inputs.len() as u32;

        // Calculate line span (simplified approach)
        calc.lines_of_code = 1; // Will be improved with proper span handling

        // Visit method body
        calc.visit_block(&method.block);

        calc.finish()
    }

    /// Calculate complexity for trait method (may not have body).
    pub fn calculate_trait_method_complexity(method: &syn::TraitItemFn) -> ComplexityMetrics {
        let mut calc = Self::new();

        // Count parameters
        calc.parameter_count = method.sig.inputs.len() as u32;

        // Calculate line span (simplified approach)
        calc.lines_of_code = 1; // Will be improved with proper span handling

        // Visit method body if it exists
        if let Some(block) = method.default.as_ref() {
            calc.visit_block(block);
        }

        calc.finish()
    }

    /// Calculate basic structural complexity for non-function items.
    pub fn calculate_structural_complexity(item: &syn::Item) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics::default();

        match item {
            syn::Item::Struct(item_struct) => {
                // Struct complexity based on field count and generic parameters
                let field_count = match &item_struct.fields {
                    syn::Fields::Named(fields) => fields.named.len(),
                    syn::Fields::Unnamed(fields) => fields.unnamed.len(),
                    syn::Fields::Unit => 0,
                } as u32;

                let generic_count = item_struct.generics.params.len() as u32;
                metrics.cyclomatic = 1 + field_count / 3 + generic_count; // Rough heuristic
                metrics.cognitive = field_count / 2 + generic_count;
            }

            syn::Item::Enum(item_enum) => {
                // Enum complexity based on variant count and complexity
                let variant_count = item_enum.variants.len() as u32;
                let complex_variants = item_enum
                    .variants
                    .iter()
                    .filter(|v| !matches!(v.fields, syn::Fields::Unit))
                    .count() as u32;

                metrics.cyclomatic = variant_count;
                metrics.cognitive = variant_count + complex_variants;
            }

            syn::Item::Trait(item_trait) => {
                // Trait complexity based on method count and associated types
                let method_count = item_trait
                    .items
                    .iter()
                    .filter(|item| matches!(item, syn::TraitItem::Fn(_)))
                    .count() as u32;

                let type_count = item_trait
                    .items
                    .iter()
                    .filter(|item| matches!(item, syn::TraitItem::Type(_)))
                    .count() as u32;

                metrics.cyclomatic = method_count + type_count;
                metrics.cognitive = method_count * 2 + type_count; // Methods are more complex
            }

            syn::Item::Impl(item_impl) => {
                // Impl block complexity based on method count
                let method_count = item_impl
                    .items
                    .iter()
                    .filter(|item| matches!(item, syn::ImplItem::Fn(_)))
                    .count() as u32;

                metrics.cyclomatic = method_count;
                metrics.cognitive = method_count;
            }

            _ => {
                // Default complexity for other items
                metrics.cyclomatic = 1;
                metrics.cognitive = 1;
            }
        }

        // Calculate line span (simplified approach)
        metrics.lines_of_code = 1; // Will be improved with proper span handling

        metrics
    }

    /// Finish calculation and return final metrics.
    fn finish(self) -> ComplexityMetrics {
        let halstead = self.calculate_halstead_metrics();

        ComplexityMetrics {
            cyclomatic: self.cyclomatic,
            cognitive: self.cognitive,
            halstead,
            nesting_depth: self.max_nesting_depth,
            lines_of_code: self.lines_of_code,
            parameter_count: self.parameter_count,
            return_count: self.return_count,
        }
    }

    /// Calculate Halstead complexity metrics from collected operators and operands.
    fn calculate_halstead_metrics(&self) -> HalsteadMetrics {
        let n1 = self.halstead_operators.len() as u32;
        let n2 = self.halstead_operands.len() as u32;
        let big_n1: u32 = self.halstead_operators.values().sum();
        let big_n2: u32 = self.halstead_operands.values().sum();

        let vocabulary = n1 + n2;
        let length = big_n1 + big_n2;

        let calculated_length = if n1 > 0 && n2 > 0 {
            (n1 as f64) * (n1 as f64).log2() + (n2 as f64) * (n2 as f64).log2()
        } else {
            0.0
        };

        let volume = if vocabulary > 0 {
            (length as f64) * (vocabulary as f64).log2()
        } else {
            0.0
        };

        let difficulty = if n2 > 0 {
            ((n1 as f64) / 2.0) * ((big_n2 as f64) / (n2 as f64))
        } else {
            0.0
        };

        let effort = difficulty * volume;

        HalsteadMetrics {
            n1,
            n2,
            big_n1,
            big_n2,
            vocabulary,
            length,
            calculated_length,
            volume,
            difficulty,
            effort,
        }
    }

    /// Record a Halstead operator.
    fn record_operator(&mut self, op: &str) {
        *self.halstead_operators.entry(op.to_string()).or_insert(0) += 1;
    }

    /// Record a Halstead operand.
    fn record_operand(&mut self, operand: &str) {
        *self
            .halstead_operands
            .entry(operand.to_string())
            .or_insert(0) += 1;
    }

    /// Enter a nested scope.
    fn enter_scope(&mut self) {
        self.nesting_depth += 1;
        self.max_nesting_depth = self.max_nesting_depth.max(self.nesting_depth);
        self.cognitive_nesting += 1;
    }

    /// Exit a nested scope.
    fn exit_scope(&mut self) {
        if self.nesting_depth > 0 {
            self.nesting_depth -= 1;
        }
        if self.cognitive_nesting > 0 {
            self.cognitive_nesting -= 1;
        }
    }

    /// Add cyclomatic complexity.
    fn add_cyclomatic(&mut self, increment: u32) {
        self.cyclomatic += increment;
    }

    /// Add cognitive complexity with nesting multiplier.
    fn add_cognitive(&mut self, base_increment: u32) {
        self.cognitive += base_increment + self.cognitive_nesting;
    }
}

impl<'ast> Visit<'ast> for ComplexityCalculator {
    fn visit_block(&mut self, block: &'ast Block) {
        self.enter_scope();
        visit::visit_block(self, block);
        self.exit_scope();
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            // Conditional expressions add to both cyclomatic and cognitive complexity
            Expr::If(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                self.record_operator("if");
            }

            // Match expressions: each arm adds complexity
            Expr::Match(expr_match) => {
                let arm_count = expr_match.arms.len() as u32;
                self.add_cyclomatic(arm_count);
                self.add_cognitive(1); // Base cognitive cost
                self.record_operator("match");

                // Visit arms with increased nesting
                self.enter_scope();
                for arm in &expr_match.arms {
                    visit::visit_arm(self, arm);
                }
                self.exit_scope();
                return; // Skip default visit to avoid double-counting
            }

            // Loop expressions
            Expr::Loop(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                self.record_operator("loop");
            }

            Expr::While(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                self.record_operator("while");
            }

            Expr::ForLoop(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                self.record_operator("for");
            }

            // Logical operators
            Expr::Binary(expr_binary) => match expr_binary.op {
                syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                    self.add_cyclomatic(1);
                    self.record_operator(&format!("{:?}", expr_binary.op));
                }
                _ => {
                    self.record_operator(&format!("{:?}", expr_binary.op));
                }
            },

            // Try expressions (error handling complexity)
            Expr::Try(_) => {
                self.add_cyclomatic(1);
                self.add_cognitive(1);
                self.record_operator("try");
            }

            // Return statements
            Expr::Return(_) => {
                self.return_count += 1;
                self.record_operator("return");
            }

            // Break and continue (early exits)
            Expr::Break(_) => {
                self.add_cognitive(1);
                self.record_operator("break");
            }

            Expr::Continue(_) => {
                self.add_cognitive(1);
                self.record_operator("continue");
            }

            // Method calls and function calls
            Expr::Call(_) => {
                self.record_operator("call");
            }

            Expr::MethodCall(_) => {
                self.record_operator("method_call");
            }

            // Literals as operands
            Expr::Lit(expr_lit) => {
                self.record_operand(&format!("{:?}", expr_lit.lit));
            }

            // Paths (variables, constants) as operands
            Expr::Path(expr_path) => {
                if let Some(ident) = expr_path.path.get_ident() {
                    self.record_operand(&ident.to_string());
                }
            }

            _ => {}
        }

        visit::visit_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if let Stmt::Local(_) = stmt {
            self.record_operator("let");
        }

        visit::visit_stmt(self, stmt);
    }
}

impl Default for ComplexityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to calculate complexity for any function-like item.
pub fn calculate_complexity_for_item(item: &syn::Item) -> u32 {
    let metrics = ComplexityCalculator::calculate_structural_complexity(item);
    metrics.overall_score()
}

/// Helper function to calculate complexity for function items specifically.
pub fn calculate_function_complexity(item_fn: &syn::ItemFn) -> u32 {
    let metrics = ComplexityCalculator::calculate_function_complexity(item_fn);
    metrics.overall_score()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_function_complexity() {
        let func: syn::ItemFn = parse_quote! {
            fn simple() {
                println!("hello");
            }
        };

        let metrics = ComplexityCalculator::calculate_function_complexity(&func);
        assert_eq!(metrics.cyclomatic, 1); // Base complexity
        assert!(metrics.cognitive <= 2); // Should be low
    }

    #[test]
    fn test_conditional_complexity() {
        let func: syn::ItemFn = parse_quote! {
            fn with_conditions(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        return x * 2;
                    }
                    return x + 1;
                } else {
                    return 0;
                }
            }
        };

        let metrics = ComplexityCalculator::calculate_function_complexity(&func);
        assert!(metrics.cyclomatic >= 3); // Multiple decision points
        assert!(metrics.cognitive >= 3); // Nesting increases cognitive load
        assert_eq!(metrics.return_count, 3); // Three return statements
    }

    #[test]
    fn test_loop_complexity() {
        let func: syn::ItemFn = parse_quote! {
            fn with_loop() {
                for i in 0..10 {
                    if i % 2 == 0 {
                        continue;
                    }
                    println!("{}", i);
                }
            }
        };

        let metrics = ComplexityCalculator::calculate_function_complexity(&func);
        assert!(metrics.cyclomatic >= 2); // Loop + condition
        assert!(metrics.nesting_depth >= 2); // Loop and if nesting
    }

    #[test]
    fn test_match_complexity() {
        let func: syn::ItemFn = parse_quote! {
            fn with_match(x: Option<i32>) -> i32 {
                match x {
                    Some(value) if value > 0 => value,
                    Some(value) => -value,
                    None => 0,
                }
            }
        };

        let metrics = ComplexityCalculator::calculate_function_complexity(&func);
        assert!(metrics.cyclomatic >= 3); // Three match arms
    }

    #[test]
    fn test_struct_complexity() {
        let item: syn::Item = parse_quote! {
            struct Complex<T, U> {
                field1: T,
                field2: U,
                field3: String,
                field4: Vec<i32>,
            }
        };

        let metrics = ComplexityCalculator::calculate_structural_complexity(&item);
        assert!(metrics.cyclomatic >= 2); // Fields and generics contribute
    }

    #[test]
    fn test_enum_complexity() {
        let item: syn::Item = parse_quote! {
            enum Status {
                Pending,
                Processing { progress: f64 },
                Complete(String),
                Failed { error: String, code: i32 },
            }
        };

        let metrics = ComplexityCalculator::calculate_structural_complexity(&item);
        assert_eq!(metrics.cyclomatic, 4); // Four variants
        assert!(metrics.cognitive >= 4); // Complex variants add more
    }

    #[test]
    fn test_complexity_levels() {
        let low = ComplexityMetrics {
            cyclomatic: 2,
            cognitive: 1,
            ..Default::default()
        };
        assert_eq!(low.complexity_level(), ComplexityLevel::Low);

        let high = ComplexityMetrics {
            cyclomatic: 15,
            cognitive: 8,
            nesting_depth: 4,
            ..Default::default()
        };
        assert_eq!(high.complexity_level(), ComplexityLevel::High);
    }

    #[test]
    fn test_halstead_metrics() {
        let func: syn::ItemFn = parse_quote! {
            fn halstead_test(x: i32, y: i32) -> i32 {
                let result = x + y * 2;
                if result > 10 {
                    result - 1
                } else {
                    result + 1
                }
            }
        };

        let metrics = ComplexityCalculator::calculate_function_complexity(&func);
        assert!(metrics.halstead.n1 > 0); // Should have operators
        assert!(metrics.halstead.n2 > 0); // Should have operands
        assert!(metrics.halstead.volume > 0.0); // Should calculate volume
    }
}
