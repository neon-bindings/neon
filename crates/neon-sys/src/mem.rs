use raw::Local;

extern "system" {

    #[link_name = "NeonSys_Mem_SameHandle"]
    pub fn SameHandle(h1: Local, h2: Local) -> bool;

}
