Electron 4 and windows
======================

Electron 4 uses delayed loading to circumvent the need to name the exectutable node.exe. See https://electronjs.org/docs/tutorial/using-native-node-modules#a-note-about-win_delay_load_hook

The library build with neon therefore must be build with delayed loading. This slightly complicates the build process.

The hook used is the same as node-gyp uses. It is build with the neon runtime. To add the hook to the resulting dynamic library a `.cargo/config` file must be made with the following contents to make the linker jump through the right hoops.

```
[build]
rustflags = ["-C", "link-args=/DELAYLOAD:node.exe /INCLUDE:load_exe_hook /INCLUDE:__pfnDliNotifyHook2 delayimp.lib"]
```

**neon cli should be patched to do this**

A few far from obvious (to me) things happen here. The (patched) neon-runtime library contains the hook, but for the linker to find the symbols and use them for creating the hook they must be included. The library with the hook `delayimp.lib` must be included as well, but it must be included AFTER the library with the hook. Thus we simply add it to the linker flags, adding it sooner, eg with `cargo:rustc-link-lib=delayimp` makes it add a default hook that does nothing and results in a duplicate symbol warning for our `_pfnDliNotifyHook2`.

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

Have fun using neon with electron 4 (and hopefully up) in electron!