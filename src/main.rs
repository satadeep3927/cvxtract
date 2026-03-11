mod core;

use core::Model;

#[tokio::main]
async fn main() {
    let mut model = Model::from_local();

    let response = model.generate("What is the capital of France?").await;
    println!("Response: {}", response);
}
