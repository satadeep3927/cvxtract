use clap::Parser;
use cvxtract::{Extractor, Model};
use std::path::PathBuf;
use std::process;

/// Extract structured resume data from a CV file using GitHub Copilot.
#[derive(Parser)]
#[command(name = "cvxtract", version, about, long_about = None)]
struct Cli {
    /// Path to the CV file (PDF, DOCX, HTML, or TXT)
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Save extracted data to a JSON file instead of printing to stdout
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let model = match Model::from_copilot(None) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let file = cli.file.to_string_lossy().into_owned();
    eprintln!("Extracting: {file}");

    let mut extractor = Extractor::new(Some(model));
    let resume = match extractor.extract_resume(file.into()).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let json = serde_json::to_string_pretty(&resume).expect("serialisation failed");

    match cli.output {
        Some(path) => {
            std::fs::write(&path, &json).unwrap_or_else(|e| {
                eprintln!("error: could not write {}: {e}", path.display());
                process::exit(1);
            });
            eprintln!("Saved to {}", path.display());
        }
        None => print_resume(&resume),
    }
}

fn print_resume(r: &cvxtract::Resume) {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {}", r.name);
    if let Some(e) = &r.email {
        println!("  Email    : {e}");
    }
    if let Some(p) = &r.phone {
        println!("  Phone    : {p}");
    }
    if let Some(l) = &r.location {
        println!("  Location : {l}");
    }
    if let Some(li) = &r.linkedin {
        println!("  LinkedIn : {li}");
    }
    if let Some(gh) = &r.github {
        println!("  GitHub   : {gh}");
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if let Some(s) = &r.summary {
        println!("\nSUMMARY\n  {s}");
    }

    if !r.experience.is_empty() {
        println!("\nEXPERIENCE");
        for job in &r.experience {
            let start = date_str(job.duration.start.as_ref());
            let end = job
                .duration
                .end
                .as_ref()
                .map_or("Present".into(), |d| date_str(Some(d)));
            println!(
                "  • {} @ {}  [{start} – {end}]",
                job.role,
                job.company.as_deref().unwrap_or("—")
            );
            for h in &job.highlights {
                println!("      – {h}");
            }
        }
    }

    if !r.education.is_empty() {
        println!("\nEDUCATION");
        for edu in &r.education {
            let start = date_str(edu.duration.start.as_ref());
            let end = edu
                .duration
                .end
                .as_ref()
                .map_or("Present".into(), |d| date_str(Some(d)));
            let degree = [edu.degree.as_deref(), edu.field.as_deref()]
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            println!("  • {}  [{start} – {end}]", edu.institution);
            if !degree.is_empty() {
                println!("    {degree}");
            }
        }
    }

    if !r.skills.is_empty() {
        println!("\nSKILLS");
        for sg in &r.skills {
            let label = sg.category.as_deref().unwrap_or("General");
            println!("  {label}: {}", sg.items.join(", "));
        }
    }

    if !r.certifications.is_empty() {
        println!("\nCERTIFICATIONS");
        for c in &r.certifications {
            let issuer = c.issuer.as_deref().unwrap_or("");
            println!("  • {}  {issuer}", c.name);
        }
    }

    if !r.languages.is_empty() {
        println!("\nLANGUAGES");
        let list: Vec<String> = r
            .languages
            .iter()
            .map(|l| match &l.proficiency {
                Some(p) => format!("{} ({})", l.language, p),
                None => l.language.clone(),
            })
            .collect();
        println!("  {}", list.join("  ·  "));
    }

    println!();
}

fn date_str(d: Option<&cvxtract::PartialDate>) -> String {
    match d {
        None => "?".into(),
        Some(pd) => match (pd.year, pd.month) {
            (Some(y), Some(m)) => format!("{m:02}/{y}"),
            (Some(y), None) => format!("{y}"),
            _ => "?".into(),
        },
    }
}
