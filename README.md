# Backup Checker
Check if all files where backed up by comparing their checksums.

## Usage
To check if all files were copied from the old to the new directory:
```bash
backup-checker [OPTIONS] --old-folder <OLD_FOLDER> --new-folder <NEW_FOLDER>
```
To get all available options:
```bash
backup-checker --help
```
## Build
```bash
cross build -r
```
### Cross Compilation
```bash
cross build -r --target x86_64-pc-windows-msvc
```
