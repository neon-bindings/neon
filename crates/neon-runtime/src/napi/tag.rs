use raw::Local;

pub unsafe extern "C" fn is_undefined(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_null(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_number(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_boolean(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_string(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_object(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_array(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_function(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_error(_val: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_buffer(_obj: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_arraybuffer(_obj: Local) -> bool { unimplemented!() }
