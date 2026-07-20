use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn main() {
    println!("Install Script Generator");

    let project_name = prompt("Project Name");
    let download_url = prompt("Binary download URL (Direct link to .zip, .exe, or binary)");

    if !download_url.starts_with("http") {
        println!(
            "\x1b[1;33mWarning:\x1b[0m Your URL doesn't start with http/https. The installer might fail."
        );
    }

    // Platform-specific default path suggestion
    #[cfg(windows)]
    let default_hint = "$env:USERPROFILE\\AppData\\Local";

    #[cfg(unix)]
    let default_hint = "$HOME/.local/bin";

    let default_path = prompt(&format!(
        "Default install path base (e.g., {})",
        default_hint
    ));

    let install_dir_path = PathBuf::from(&default_path).join(&project_name);
    let install_dir = install_dir_path.to_string_lossy();

    // Generate script content based on target OS
    let script_content = generate_script(&project_name, &download_url, &install_dir);
    let extension = get_script_extension();
    let safe_name = project_name.replace(" ", "_");
    let filename = format!("{}-installer.{}", safe_name, extension);

    match save_to_file(&filename, &script_content) {
        Ok(_) => {
            println!(
                "\n\x1b[1;32mSUCCESS:\x1b[0m Script generated as \x1b[1m{}\x1b[0m",
                filename
            );
            print_usage_instructions(&filename);
        }
        Err(e) => eprintln!("\n\x1b[1;31mERROR:\x1b[0m Could not save file: {}", e),
    }
}

fn generate_script(project_name: &str, download_url: &str, install_dir: &str) -> String {
    #[cfg(windows)]
    {
        // Use r##"..."## to allow "#" and """ inside the PowerShell script
        format!(
            r##"# PowerShell Installer for {project_name}
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
    exit 1
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
    Write-Host "PATH updated successfully. Restart terminal to use {project_name}." -ForegroundColor Gray
}} else {{
    Write-Host "Directory already in PATH." -ForegroundColor Gray
}}

Write-Host "Done!" -ForegroundColor Green
Pause
"##,
            project_name = project_name,
            download_url = download_url,
            install_dir = install_dir,
        )
    }

    #[cfg(unix)]
    {
        format!(
            r##"#!/bin/bash
# Bash Installer for {project_name}
set -e

URL="{download_url}"
INSTALL_DIR="{install_dir}"
TEMP_FILE="/tmp/{project_name}_installer_temp"

echo -e "\e[36m--- Starting Installation for {project_name} ---\e[0m"

# 1. Create Directory
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "\e[37m[1/4] Creating directory: $INSTALL_DIR\e[0m"
    mkdir -p "$INSTALL_DIR"
else
    echo -e "\e[37m[1/4] Directory already exists: $INSTALL_DIR\e[0m"
fi

# 2. Download
echo -e "\e[33m[2/4] Downloading binaries...\e[0m"
EXTENSION="${{URL##*.}}"
DEST_FILE="$TEMP_FILE.$EXTENSION"

if command -v curl &> /dev/null; then
    curl -L -o "$DEST_FILE" "$URL"
elif command -v wget &> /dev/null; then
    wget -O "$DEST_FILE" "$URL"
else
    echo -e "\e[31mERROR: Neither curl nor wget found.\e[0m"
    exit 1
fi

if [ $? -ne 0 ]; then
    echo -e "\e[31mERROR: Failed to download.\e[0m"
    exit 1
fi

# 3. Extract or Move
if [ "$EXTENSION" == "zip" ]; then
    echo -e "\e[33m[3/4] Extracting ZIP...\e[0m"
    if command -v unzip &> /dev/null; then
        unzip -o "$DEST_FILE" -d "$INSTALL_DIR"
        rm "$DEST_FILE"
    else
        echo -e "\e[31mERROR: 'unzip' not found.\e[0m"
        exit 1
    fi
else
    echo -e "\e[33m[3/4] Moving binary...\e[0m"
    mv "$DEST_FILE" "$INSTALL_DIR/{project_name}.$EXTENSION"
fi

chmod +x "$INSTALL_DIR"/* 2>/dev/null || true

# 4. Add to PATH
echo -e "\e[33m[4/4] Adding $INSTALL_DIR to User PATH...\e[0m"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_RC=""
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -f "$HOME/.profile" ]; then
        SHELL_RC="$HOME/.profile"
    else
        SHELL_RC="$HOME/.bashrc"
        touch "$SHELL_RC"
    fi

    echo "" >> "$SHELL_RC"
    echo "# Added by {project_name} installer on $(date)" >> "$SHELL_RC"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
    
    echo -e "\e[90mPATH updated in $SHELL_RC. Restart terminal or run 'source $SHELL_RC'.\e[0m"
else
    echo -e "\e[90mDirectory already in PATH.\e[0m"
fi

echo -e "\e[32mDone!\e[0m"
read -p "Press enter to exit..."
"##,
            project_name = project_name,
            download_url = download_url,
            install_dir = install_dir,
        )
    }
}

fn get_script_extension() -> &'static str {
    #[cfg(windows)]
    {
        "ps1"
    }
    #[cfg(unix)]
    {
        "sh"
    }
}

fn print_usage_instructions(filename: &str) {
    #[cfg(windows)]
    {
        println!(
            "\x1b[33mDone!\x1b[0m Run with 'PowerShell -ExecutionPolicy Bypass -File .\\{}'",
            filename
        );
    }
    #[cfg(unix)]
    {
        println!(
            "\x1b[33mDone!\x1b[0m Run with: './{}'",
            filename
        );
    }
}

fn prompt(prompt_text: &str) -> String {
    loop {
        print!("\x1b[1;34m»\x1b[0m {}: ", prompt_text);
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

    // On Unix, set executable permission immediately
    #[cfg(unix)]
    {
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }

    Ok(())
}
