use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    println!("Install Script Generator");

    let project_name = prompt("Project Name");
    let download_url = prompt("Binary download URL (Direct link to .zip or .exe)");
    if !download_url.starts_with("http") {
        println!(            
            "\x1b[1;33mWarning:\x1b[0m Your URL doesn't start with http/https. The installer might fail."
        );
    }
    let default_path = prompt("Default install path (e.g., $env:USERPROFILE)");

    let install_dir_path = PathBuf::from(&default_path).join(&project_name);
    let install_dir = install_dir_path.to_string_lossy();
    let ps_script = format!(
        r#"# PowerShell Installer for {project_name}
        $ProgressPreference = 'SilentlyContinue'
        $url = "{download_url}"
        $installDir = "{install_dir}"
        $tempFile = "$env:TEMP\{project_name}_installer_temp"
        
        # Determine file extension from URL
        $extension = [System.IO.Path]::GetExtension($url)
        $destFile = $tempFile + $extension
        
        Write-Host "--- Starting Installation for {project_name} ---" -ForegroundColor Cyan
        
        if (!(Test-Path $installDir)) {{
            Write-Host "[1/4] Creating directory: $installDir"
            New-Item -ItemType Directory -Force -Path $installDir | Out-Null
        }}
        
        Write-Host "[2/4] Downloading binaries..." -ForegroundColor Yellow
        try {{
            Invoke-WebRequest -Uri $url -OutFile $destFile -ErrorAction Stop
        }} catch {{
            Write-Host "ERROR: Failed to download." -ForegroundColor Red
            exit
        }}
        
        if ($extension -eq ".zip") {{
            Write-Host "[3/4] Extracting ZIP..." -ForegroundColor Yellow
            Expand-Archive -Path $destFile -DestinationPath $installDir -Force
            Remove-Item -Path $destFile
        }} else {{
            Write-Host "[3/4] Moving binary..." -ForegroundColor Yellow
            Move-Item -Path $destFile -Destination "$installDir\{project_name}$extension" -Force
        }}
        
        # 4. Add to PATH (Permanent for User)
        Write-Host "[4/4] Adding $installDir to User PATH..." -ForegroundColor Yellow
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -split ";" -notcontains $installDir) {{
            $newPath = "$currentPath;$installDir"
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Host "PATH updated successfully. You will have to either restart your terminal
            or open a new one to be able to use {project_name}" -ForegroundColor Gray
        }} else {{
            Write-Host "Directory already in PATH." -ForegroundColor Gray
        }}
        
        Write-Host "Done!" -ForegroundColor Green
        Pause
"#,
        project_name = project_name,
        download_url = download_url,
        install_dir = install_dir,
    );

    let safe_name = project_name.replace(" ", "_");
    let filename = format!("{}-Installer.ps1", safe_name);

    match save_to_file(&filename, &ps_script) {
        Ok(_) => {
            println!(
                "\n\x1b[1;32mSUCCESS:\x1b[0m Script generated as \x1b[1m{}\x1b[0m",
                filename
            );
            println!(
                "\x1b[33mNote:\x1b[0m Run with 'PowerShell -ExecutionPolicy Bypass -File .\\{}'",
                filename
            );
        }
        Err(e) => eprintln!("\n\x1b[1;31mERROR:\x1b[0m Could not save file: {}", e),
    }
}

fn prompt(prompt: &str) -> String {
    loop {
        print!("\x1b[1;34m»\x1b[0m {}: ", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let trimmed = input.trim().to_string();

        if !trimmed.is_empty() {
            return trimmed;
        }

        println!("\x1b[1;31m  Input cannot be empty.\x1b[0m");
    }
}

fn save_to_file(filename: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
