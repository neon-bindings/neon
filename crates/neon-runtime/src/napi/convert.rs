use raw::Local;

pub unsafe extern "C" fn to_object(_out: &mut Local, _value: &Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn to_string(_out: &mut Local, _value: Local) -> bool { unimplemented!() }
