//! GPUI's release build uses this small FXC-compatible adapter when the SDK is absent.
//! It invokes Windows' D3DCompileFromFile (the same compiler used by GPUI in debug),
//! with standard #include support, shader model 4.1 and optimization level 3.
//! Only GPUI's /T /E /Fh /Vn /O3 arguments are accepted; this is not a general FXC replacement.
use std::{
    ffi::{c_void, CString},
    os::windows::ffi::OsStrExt,
    path::PathBuf,
    ptr,
};

#[repr(C)]
struct Blob {
    vtable: *const BlobVtable,
}
#[repr(C)]
struct BlobVtable {
    query: *const c_void,
    add_ref: *const c_void,
    release: unsafe extern "system" fn(*mut Blob) -> u32,
    pointer: unsafe extern "system" fn(*mut Blob) -> *const u8,
    size: unsafe extern "system" fn(*mut Blob) -> usize,
}
#[link(name = "d3dcompiler_47", kind = "raw-dylib")]
extern "system" {
    fn D3DCompileFromFile(
        file: *const u16,
        macros: *const c_void,
        include: *const c_void,
        entry: *const u8,
        target: *const u8,
        flags: u32,
        effect_flags: u32,
        code: *mut *mut Blob,
        errors: *mut *mut Blob,
    ) -> i32;
}

unsafe fn bytes<'a>(blob: *mut Blob) -> &'a [u8] {
    std::slice::from_raw_parts(
        ((*(*blob).vtable).pointer)(blob),
        ((*(*blob).vtable).size)(blob),
    )
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (mut source, mut target, mut entry, mut output, mut variable) =
        (None, None, None, None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "/T" => target = args.next(),
            "/E" => entry = args.next(),
            "/Fh" => output = args.next(),
            "/Vn" => variable = args.next(),
            "/O3" => (),
            _ if !arg.starts_with('/') && source.is_none() => source = Some(PathBuf::from(arg)),
            _ => return Err(format!("unsupported shader compiler argument: {arg}").into()),
        }
    }
    let source = source.ok_or("missing shader source")?;
    let target = CString::new(target.ok_or("missing /T")?)?;
    let entry = CString::new(entry.ok_or("missing /E")?)?;
    let output = output.ok_or("missing /Fh")?;
    let variable = variable.ok_or("missing /Vn")?;
    let file: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let mut code = ptr::null_mut();
        let mut errors = ptr::null_mut();
        let result = D3DCompileFromFile(
            file.as_ptr(),
            ptr::null(),
            1usize as *const c_void,
            entry.as_ptr().cast(),
            target.as_ptr().cast(),
            1 << 15,
            0,
            &mut code,
            &mut errors,
        );
        if !errors.is_null() {
            if result < 0 {
                eprintln!("{}", String::from_utf8_lossy(bytes(errors)));
            }
            ((*(*errors).vtable).release)(errors);
        }
        if result < 0 || code.is_null() {
            return Err(format!("D3DCompileFromFile failed: 0x{result:08X}").into());
        }
        let data = bytes(code);
        let mut header = format!("const BYTE {variable}[] = {{\n");
        for chunk in data.chunks(24) {
            for byte in chunk {
                header.push_str(&format!("{byte},"));
            }
            header.push('\n');
        }
        header.push_str("};\n");
        ((*(*code).vtable).release)(code);
        std::fs::write(output, header)?;
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("shader compiler: {error}");
        std::process::exit(1);
    }
}
