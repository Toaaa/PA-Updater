use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use reqwest::Client;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sets the path where the zip file should be downloaded and extracted to (default: current directory)
    #[clap(short, long)]
    path: Option<String>,

    /// Downloads a specific version (default: latest)
    #[clap(short, long)]
    download_version: Option<String>,
}

async fn download_file(url: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut response = client.get(url).send().await?;

    if response.status().is_success() {
        let mut file = File::create(file_path)?;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk)?;
        }
        println!("File downloaded successfully");
    } else {
        println!("Failed to download the file from '{}': {}", url, response.status());
    }

    Ok(())
}

async fn extract_zip(zip_file: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_file)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.name();
        println!("Extracting {}...", file_path);

        // Modify the destination path to use the target directory
        let dest_path = target_dir.join(file_path);

        if let Some(parent_dir) = dest_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        if file.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            let mut outfile = File::create(&dest_path)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    if zip_file.exists() {
        fs::remove_file(&zip_file)?;
    }

    println!("ZIP file extracted successfully to: {}", target_dir.display());
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let zip_url = match args.download_version {
        Some(version) => format!("https://pa.toaaa.de/{}/Project-Apparatus.zip", version),
        None => "https://pa.toaaa.de/latest/Project-Apparatus-latest.zip".to_string(),
    };

    let zip_name = "Project-Apparatus.zip";
    let zip_path = Path::new(&zip_name);

    if zip_path.exists() {
        println!("ZIP file already exists, overwrite...");
        fs::remove_file(&zip_path).unwrap();
    }


    let mut path = args.path.as_deref().map_or_else(|| std::env::current_dir().unwrap(), |s| Path::new(s).to_path_buf());
    path.push(zip_path);

    let target_dir = args.path.map_or_else(|| std::env::current_dir().unwrap(), |s| Path::new(&s).to_path_buf());

    download_file(&zip_url, &path).await.unwrap();
    extract_zip(&path, &target_dir).await.unwrap();

    Ok(())

    // println!("ZIP file extracted successfully to: {}", path.display());
}