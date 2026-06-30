use std::os::raw::{c_char, c_int};

unsafe extern "C" {
    fn run_cpp_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() {
    let args: Vec<std::ffi::CString> = std::env::args()
        .map(|arg| std::ffi::CString::new(arg).unwrap())
        .collect();
    let c_args: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();

    let exit_code = unsafe { run_cpp_main(c_args.len() as c_int, c_args.as_ptr()) };
    std::process::exit(exit_code);
}
