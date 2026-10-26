# United-Editer

One Windows app for parametric photo development and pixel editing, on one engine: PhotoCraft's
layered document engine at the core, with LightCraft's raw decoder, develop pipeline and catalog
ported into it. A raw file loads as an editable layer; photos stay parametric, and stacking,
frequency separation and retouching are stored as instructions, not pixels.

Personal fork, built in sessions. Not affiliated with the upstream projects.

## Layout (phase 0)

| Path | What | Upstream |
| --- | --- | --- |
| `lightcraft/` | LightCraft 0.7.1, unchanged, full history | [storytold/lightcraft](https://github.com/storytold/lightcraft) via `jorgerjimenez/lightcraft_901220` |
| `photocraft/` | PhotoCraft 0.8.0, unchanged, full history | [storytold/photocraft](https://github.com/storytold/photocraft) via `jorgerjimenez/photocraft_901220` |
| `tools/baseline.ps1` | Time and peak RAM of both apps on your own files | |
| `.github/workflows/windows.yml` | Windows build, tests and downloadable zips | |

Both apps are still separate Cargo workspaces. They become one workspace in phase 1.

## Getting a build

1. Open the **Actions** tab, then the latest **Windows** run on `main`.
2. Under **Artifacts**, download `lightcraft-windows-x64` and `photocraft-windows-x64`.
3. Unzip both into one folder. The builds are unsigned, so Windows SmartScreen asks once per exe:
   **More info → Run anyway**.

## Measuring the baseline

```powershell
powershell -ExecutionPolicy Bypass -File tools\baseline.ps1 -Folder "<folder with photos>" -Bin "<folder with the unzipped builds>" -Limit 5
```

It renders up to 5 files of each format with both apps (LightCraft on GPU and CPU) and writes
a CSV with seconds and peak RAM per run. Your photos are only read.

## Licences

Both upstream projects are MIT OR Apache-2.0; their licence and notice files are kept in each
subdirectory.
