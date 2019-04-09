Electron 4 and windows
======================


*** only tested with hello world, not yet with a real life project, though it should be no different, it's just a hook ***
----------------------------------------------------------------------

Electron 4 uses delayed loading to circumvent the need to name the executable node.exe. See https://electronjs.org/docs/tutorial/using-native-node-modules#a-note-about-win_delay_load_hook

Therefore the library build with neon must be build with delayed loading. This slightly complicates the build process.

The hook used is the same as node-gyp uses. It is build with the neon runtime. To add the hook to the resulting dynamic library a `.cargo/config` file must be made with the following contents to make the linker jump through the right hoops.

```
[target.'cfg(windows)']
rustflags = ["-C", "link-args=/DELAYLOAD:node.exe /INCLUDE:__load_exe_hook /INCLUDE:__pfnDliNotifyHook2 delayimp.lib"]
```

**the cli has been patched to do this**

A few far from obvious (to me) things happen here. The (patched) neon-runtime library contains the hook. For the linker to find the related symbols and use those in creating the hook they must be explicitly included. The windows library with the helpers, `delayimp.lib`, must be included as well, but it must be included AFTER the library with the hook. Thus we simply add it to the linker flags, adding it sooner, eg with `cargo:rustc-link-lib=delayimp` makes it add a default hook that does nothing and results in a duplicate symbol warning for our `_pfnDliNotifyHook2`.

If you're using version 0.2.0 of neon that doesn't have this patch, it is possible to build the hook in your own project (first create the .cargo/config file as mentioned above):

Asjust `build.rs` so it contains the following

```
extern crate neon_build;
extern crate cc;

fn main() {
   neon_build::setup(); // must be called in build.rs
   cc::Build::new()
        .cpp(true)
        .static_crt(true)
        .file("src/win_delay_load_hook.cc")
        .compile("hook");
}
```

Add the file [crates/neon-runtime/src/win_delay_load_hook.cc](https://github.com/jrd-rocks/neon/blob/electron_delay_hook/crates/neon-runtime/src/win_delay_load_hook.cc) to your project's `native/src`

Finally add the cc crate to your build dependencies in `native/Cargo.toml` as follows:

```
[build-dependencies]
neon-build = "0.2.0"
cc = "1.0"
```

Have fun using neon with electron 4 (and hopefully up) in windows!