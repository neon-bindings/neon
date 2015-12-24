use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Mem_SameHandle"]
    pub fn same_handle(h1: Local, h2: Local) -> bool;

}
