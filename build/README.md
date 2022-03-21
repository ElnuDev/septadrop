# Building

In order to have the required development dependencies (C bindings for SFML), install them using your system's package manager. This is `libcsfml-dev` on Debian-based systems, `csfml` on Arch-based systems, and `csfml` on Homebrew if you have the misfortune of running macOS. The package must be at version 2.5.

## Packaging

### Debian

For packaging, `upx` is required to optimize executable file size. It is available under the `upx` package.

To package:
```SH
./debian.sh
```

### Windows

```SH
windows.bat
```

Then compress the `windows` directory. If you haven't configured your system for packaging, it will give guidance on setup.

In [windows.bat](windows.bat#L5) there is an option to enable packing with UPX by setting `SET USE_UPX="y"`. This drastically reduces the file size of the resulting executable by more than half. However, this option is off by default because Windows tends to flag executables packed with UPX as viruses. If you want to enable UPX anyway, download it from the latest version from their [releases](https://github.com/upx/upx/releases) and place upx.exe in this build folder.