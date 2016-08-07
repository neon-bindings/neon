## How to debug your native extension

You must start by building your native extension in debug mode

    neon build --debug

Then start node using rust-lldb

    rust-lldb node

You need to pass your node command line arguments through the debugger

    process launch -- -e 'require("./")'

To setup breakpoints you can use the regexp breakpoint command
    
    br s -r <your-library>::<your-function>

where <your-library> is  the value of package.name in your Cargo.toml file.

Print variables with

    p <your-var-name>


## OS X specific

In order to be able to print variable information on OS X 
you must have XCode 8.0 or better installed, see the following
issue: https://github.com/rust-lang/rust/issues/33062

You may also need this: 
http://stackoverflow.com/questions/25996484/xcode-wont-start-stuck-on-verifying-xcode/26476988#26476988

After you have installed the beta point the command line tools to it: 

    sudo xcode-select -s /Applications/Xcode-beta.app/Contents/Developer
