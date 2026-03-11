use cvxtract::{Extractor, Model};

#[tokio::main]
async fn main() {
    let mut extractor = Extractor::new(Some(Model::from_local()));

    let resume = extractor
        .extract_resume(
            r"C:\Users\BIT1053\Downloads\Vikas_A_ERP_Specialist_&_Solution_Architect.pdf"
                .into(),
        )
        .await;

    match resume {
        Ok(r) => println!("{r:#?}"),
        Err(e) => eprintln!("Extraction failed: {e}"),
    }
}
