/// A complex function to test our complexity calculation
pub fn complex_function(x: i32, y: Option<String>) -> Result<String, &'static str> {
    if x < 0 {
        return Err("negative input");
    }
    
    let result = match y {
        Some(s) if s.len() > 10 => {
            for i in 0..x {
                if i % 2 == 0 {
                    println!("Even: {}", i);
                } else {
                    println!("Odd: {}", i);
                }
            }
            s.to_uppercase()
        }
        Some(s) => s,
        None => "default".to_string(),
    };
    
    if result.len() > 5 {
        Ok(result)
    } else {
        Err("too short")
    }
}

/// A complex enum with various variant types
#[derive(Debug, Clone)]
pub enum ComplexEnum {
    Simple,
    Tuple(String, i32, bool),
    Struct { name: String, age: u32, active: bool },
    Generic(Box<ComplexEnum>),
    Nested { data: Vec<(String, Option<i32>)> },
}

/// A trait with multiple methods
pub trait ComplexTrait {
    fn required_method(&self) -> String;
    
    fn default_method(&self) -> i32 {
        42
    }
    
    fn generic_method<T: Clone>(&self, item: T) -> T {
        item.clone()
    }
    
    fn complex_method(&self, input: &str) -> Result<Vec<String>, String> {
        if input.is_empty() {
            return Err("empty input".to_string());
        }
        
        let mut results = Vec::new();
        for word in input.split_whitespace() {
            if word.len() > 3 {
                results.push(word.to_uppercase());
            } else {
                results.push(word.to_lowercase());
            }
        }
        
        Ok(results)
    }
}