# installgen

Installgen is a tool to make install scripts for other tools. It generates a PowerShell script that can be used to download and install a binary or archive from a given URL. (Later it might support bash and other shells)

> I used AI to write the PowerShell (I don't know PowerShell). I tested it and it works well.
However if you still have any issues open a pull request or issue.

## Installation

Installgen doesn't have a published version yet, but you can build it from source:

```bash
git clone https://github.com/Pjdur/installgen.git
cd installgen
cargo install --path .
```

## Usage

Run `installgen` and follow the prompts to generate your PowerShell installer script. The generated script will be saved as `[yourproject]-Installer.ps1` in the current directory.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.