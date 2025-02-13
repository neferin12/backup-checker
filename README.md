# Backup Checker
Check if all files where backed up. Even finds renamed files (uses shasums).

## Usage
To check if all files were copied from the old to the new directory:
```bash
backup-checker [OPTIONS] --old-folder <OLD_FOLDER> --new-folder <NEW_FOLDER>
```
| Option                 | Default | Description                                                                        |
|------------------------|---------|------------------------------------------------------------------------------------|
| `-o`, `--old-folder`   |         | The folder with the original files                                                 |
| `-n`, `--new-folder`   |         | The folder with the backed up files                                                |
| `-d`, `--max-depth`    | 1000    | Max depth is the maximal depth to which the directories are traversed recursively. |
| `-h`, `--help`         |         | Print help                                                                         |
| `-V`, `--version`      |         | Print version                                                                      |

## Build
```bash
cross build -r
```
### Cross Compilation
```bash
cross build -r --target x86_64-pc-windows-msvc
```