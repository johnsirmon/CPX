# CPX install and use guide

This guide is for people who just want to install CPX and use it safely.

CPX is a command-line tool. The easiest way to install it is to download the release archive for your platform, extract it, and run the included `cpx` program. CPX does not currently use a Windows installer, MSI, or Linux package-manager package.

## What CPX does

CPX helps you prepare a text case for AI use without sending raw customer identifiers downstream.

It creates two main outputs:

- a projection file you can share with the downstream AI workflow
- a vault file that must stay local because it contains the encrypted mapping back to the original values

## Before you start

You need:

- a CPX binary for your platform: `cpx.exe` on Windows or `cpx` on Linux
- a plain-text case file such as `case.txt`
- a passphrase you will keep private on your own machine

Preferred source:

- download the current CPX release archive for your platform from the repository release page

The release archive name includes the version, for example:

- Windows: `cpx-windows-x86_64-v0.1.0.zip`
- Linux: `cpx-linux-x86_64-v0.1.0.tar.gz`

If someone on your team distributes CPX internally, ask them for the current approved CPX release archive for your platform.

## Install on Windows

### Simple install

1. Create a folder for CPX, for example `C:\Tools\CPX`.
2. Download and extract the Windows release archive.
3. Copy `cpx.exe` from the extracted folder into `C:\Tools\CPX`, or keep the extracted folder where it is.
4. Open PowerShell.
5. Run this command to confirm CPX starts:

```powershell
C:\Tools\CPX\cpx.exe --help
```

If you see help text with the commands `ingest`, `project`, and `rehydrate`, CPX is installed correctly.

### Optional: make `cpx` work from any folder

If you do not want to type the full path every time:

1. Open the Start menu and search for `Environment Variables`.
2. Open `Edit the system environment variables`.
3. Select `Environment Variables`.
4. Under `User variables`, select `Path`, then select `Edit`.
5. Select `New` and add `C:\Tools\CPX`.
6. Select `OK` on each window.
7. Close PowerShell and open it again.
8. Run:

```powershell
cpx --help
```

## Install on Linux

### Simple install

1. Open a terminal.
2. Create a local tools folder:

```bash
mkdir -p "$HOME/.local/bin"
```

3. Download and extract the Linux release archive.
4. Copy the `cpx` binary into that folder.
5. Make it executable:

```bash
chmod +x "$HOME/.local/bin/cpx"
```

6. Confirm CPX starts:

```bash
"$HOME/.local/bin/cpx" --help
```

If you see help text with the commands `ingest`, `project`, and `rehydrate`, CPX is installed correctly.

### Optional: make `cpx` work from any folder

If `~/.local/bin` is not already on your `PATH`, add it.

For `bash`:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
source "$HOME/.bashrc"
```

For `zsh`:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
source "$HOME/.zshrc"
```

Then test:

```bash
cpx --help
```

## First-time setup for everyday use

Create a working folder for one case. Keep all files for that case together.

Example folder contents after a normal run:

```text
case.txt
projection.txt
case.cpxvault
ai-answer.txt
trusted-answer.txt
```

Important:

- share `projection.txt`
- do not share `case.cpxvault`
- keep your passphrase private

If you did not add CPX to your `PATH`, use the full path to the program in the commands below.

Examples:

- Windows: `C:\Tools\CPX\cpx.exe`
- Linux: `$HOME/.local/bin/cpx`

## How to use CPX on Windows

These steps are written to be predictable for non-technical users. They use explicit file names so you do not need to guess where CPX wrote anything.

### Step 1: prepare a folder

1. Create a new folder such as `C:\CPX-Work\MyCase`.
2. Save your case text inside that folder as `case.txt`.
3. Open PowerShell in that folder.

### Step 2: set a passphrase for this terminal session

```powershell
$env:CPX_PASSPHRASE = "choose-a-passphrase-you-will-remember"
```

### Step 3: create the safe projection

```powershell
cpx project .\case.txt --output .\projection.txt --vault-output .\case.cpxvault
```

After this finishes, you should have:

- `projection.txt`
- `case.cpxvault`

### Step 4: send only the projection downstream

Send `projection.txt` to your AI workflow or copy its contents into your AI tool.

Do not send:

- the original `case.txt`
- the vault file `case.cpxvault`

### Step 5: save the symbolic AI answer

When the AI returns a symbolic answer, save that answer as `ai-answer.txt` in the same folder.

### Step 6: restore the original values locally

```powershell
cpx rehydrate .\ai-answer.txt --vault .\case.cpxvault --output .\trusted-answer.txt
```

Now `trusted-answer.txt` contains the restored local values.

## How to use CPX on Linux

### Step 1: prepare a folder

```bash
mkdir -p "$HOME/cpx-work/my-case"
cd "$HOME/cpx-work/my-case"
```

Save your case text in that folder as `case.txt`.

### Step 2: set a passphrase for this terminal session

```bash
export CPX_PASSPHRASE="choose-a-passphrase-you-will-remember"
```

### Step 3: create the safe projection

```bash
cpx project ./case.txt --output ./projection.txt --vault-output ./case.cpxvault
```

After this finishes, you should have:

- `projection.txt`
- `case.cpxvault`

### Step 4: send only the projection downstream

Send `projection.txt` to your AI workflow.

Do not send:

- the original `case.txt`
- the vault file `case.cpxvault`

### Step 5: save the symbolic AI answer

Save the symbolic answer as `ai-answer.txt` in the same folder.

### Step 6: restore the original values locally

```bash
cpx rehydrate ./ai-answer.txt --vault ./case.cpxvault --output ./trusted-answer.txt
```

Now `trusted-answer.txt` contains the restored local values.

## Quick command summary

Use these commands when you just want the basics:

- `cpx ingest case.txt` checks that CPX can read the file and prints a short summary
- `cpx project case.txt --output projection.txt --vault-output case.cpxvault` creates the safe file you can share and the local vault you must keep private
- `cpx rehydrate ai-answer.txt --vault case.cpxvault --output trusted-answer.txt` restores the original values locally

## If you are testing from a source checkout instead of an installed binary

This is mainly for contributors, not normal end users.

On Windows:

```powershell
.\scripts\bootstrap-windows.ps1
cargo run -p cpx-cli -- --help
```

On Windows or Linux, you can replace `cpx` in the examples above with:

```text
cargo run -p cpx-cli --
```

Example:

```powershell
cargo run -p cpx-cli -- project .\case.txt --output .\projection.txt --vault-output .\case.cpxvault
```

## If something goes wrong

Start with these checks:

1. Confirm `cpx --help` works.
2. Confirm your case file exists and is plain text.
3. Confirm the `CPX_PASSPHRASE` variable is set in the current terminal.
4. Confirm you are using the vault created from the same case.

Then see:

- [`troubleshooting.md`](troubleshooting.md)
- [`cli-reference.md`](cli-reference.md)
- [`..\quickstart.md`](../quickstart.md)

## For maintainers

If you need to create or publish CPX release archives, see [`release-process.md`](release-process.md).
