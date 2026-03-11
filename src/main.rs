mod core;

use core::providers::local::Local;

#[tokio::main]
async fn main() {
    println!("CVXtract - Resume Extraction with Local AI");
    println!("==========================================");
    
    let mut local = Local::new();
    
    // Test with a simple prompt (auto-initializes model)
    let test_prompt = "Analyze this resume excerpt: 'John Smith, Software Engineer with 5 years experience in Python and React.'";
    
    let response = local.generate(test_prompt).await;
    
    println!("AI Response:");
    println!("----------------------------------------");
    println!("{}", response);
    println!("----------------------------------------");
    
    if !response.contains("Error") {
        println!("✓ Local AI model is working correctly!");
    } else {
        println!("✗ AI model encountered an error.");
    }
    
    println!("\nPress Enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
