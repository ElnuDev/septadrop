# Building

In order to have the required development dependencies (C bindings for SFML), install them using your system's package manager. This is `libcsfml-dev` on Debian-based systems, `csfml` on Arch-based systems, and `csfml` on Homebrew if you have the misfortune of running macOS. The package must be at version 2.5. While septadrop hasn't been packaged for macOS, it has been confirmed to compile and run correctly on Intel Macs.

## Packaging

### Debian

For packaging, `upx` is required to optimize executable file size. It is available under the `upx` package.

> ⚠️ **For users of Ubuntu-derivatives >20.04 and Arch using UPX 3.96:** the default version of UPX in the `apt` repositories is 3.96, which will fail to pack septadrop, giving the error `CantPackException: bad DT_GNU_HASH n_bucket=0x6  n_bitmask=0x1  len=0x40`. To avoid this, you have to downgrade to UPX 3.95. On Debian-based distributions, you can install a specific package version using the following command:
> 
> ```SH
> sudo apt install upx-ucl=3.95-2build1
> ```
>
> Interestingly, this issue doesn't affect packing on Windows*. 

To package:
```SH
./debian.sh
```

### Windows

```SH
windows.bat
```

Then compress the `windows` directory. If you haven't configured your system for packaging, it will give guidance on setup.

*In [windows.bat](windows.bat#L5) there is an option to enable packing with UPX by setting `SET USE_UPX="y"`. This drastically reduces the file size of the resulting executable by more than half. However, this option is off by default because Windows tends to flag executables packed with UPX as viruses. If you want to enable UPX anyway, download it from the latest version from their [releases](https://github.com/upx/upx/releases) and place upx.exe in this build folder.
