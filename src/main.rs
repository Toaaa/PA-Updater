use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use reqwest::Client;

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
        println!("Failed to download the file: {}", response.status());
    }

    Ok(())
}

async fn extract_zip(zip_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_file)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.mangled_name();

        let dest_path = file_path;

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
        fs::remove_file(&zip_file).unwrap();
    }

    println!("ZIP file extracted successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let zip_url = "https://pa.toaaa.de/latest/Project-Apparatus-latest.zip";
    let zip_name = "Project-Apparatus-latest.zip";
    let zip_path = Path::new(zip_name);

    if zip_path.exists() {
        println!("ZIP file already exists, overwrite...");
        fs::remove_file(&zip_path).unwrap();
    }
    download_file(zip_url, zip_path).await.unwrap();

    extract_zip(zip_path).await.unwrap();

    Ok(())
}