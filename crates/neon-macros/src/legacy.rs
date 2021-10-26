pub(crate) fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn_mid::ItemFn);

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let name = &sig.ident;

    quote::quote!(
        #(#attrs) *
        #vis #sig {
            // Mark this function as a global constructor (like C++).
            #[allow(improper_ctypes)]
            #[cfg_attr(target_os = "linux", link_section = ".ctors")]
            #[cfg_attr(target_os = "android", link_section = ".ctors")]
            #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
            #[cfg_attr(target_os = "ios", link_section = "__DATA,__mod_init_func")]
            #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
            #[used]
            static __LOAD_NEON_MODULE: extern "C" fn() = {
                extern "C" fn __load_neon_module() {
                    // Put everything else in the ctor fn so the user fn can't see it.
                    #[repr(C)]
                    struct __NodeModule {
                        version: i32,
                        flags: u32,
                        dso_handle: *mut u8,
                        filename: *const u8,
                        register_func: Option<extern "C" fn(
                            ::neon::handle::Handle<::neon::types::JsObject>, *mut u8, *mut u8)>,
                        context_register_func: Option<extern "C" fn(
                            ::neon::handle::Handle<::neon::types::JsObject>, *mut u8, *mut u8, *mut u8)>,
                        modname: *const u8,
                        priv_data: *mut u8,
                        link: *mut __NodeModule
                    }

                    // Mark as used during tests to suppress warnings
                    #[cfg_attr(test, used)]
                    static mut __NODE_MODULE: __NodeModule = __NodeModule {
                        version: 0,
                        flags: 0,
                        dso_handle: 0 as *mut _,
                        filename: b"neon_source.rs\0" as *const u8,
                        register_func: Some(__register_neon_module),
                        context_register_func: None,
                        modname: b"neon_module\0" as *const u8,
                        priv_data: 0 as *mut _,
                        link: 0 as *mut _
                    };

                    extern "C" fn __register_neon_module(
                            m: ::neon::handle::Handle<::neon::types::JsObject>, _: *mut u8, _: *mut u8) {
                        ::neon::macro_internal::initialize_module(m, #name);
                    }

                    extern "C" {
                        fn node_module_register(module: *mut __NodeModule);
                    }

                    // During tests, node is not available. Skip module registration.
                    #[cfg(not(test))]
                    unsafe {
                        // Set the ABI version based on the NODE_MODULE_VERSION constant provided by the current node headers.
                        __NODE_MODULE.version = ::neon::macro_internal::runtime::module::get_version();
                        node_module_register(&mut __NODE_MODULE);
                    }
                }

                __load_neon_module
            };

            #block
        }
    )
    .into()
}
