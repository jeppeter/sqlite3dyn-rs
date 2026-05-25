
use libloading;
use std::error::Error;

pub struct SQLITE_RESULT(std::ffi::c_int);

pub struct Sqlite3Dll {
    sqlite3_dll_hdl :libloading::Library,
    sqlite3_open_v2 :libloading::Symbol<'static,unsafe extern "C" fn(fname :* const std::ffi::c_char,ppdb :*const *mut std::ffi::c_void,flags :std::ffi::c_uint,connstr :*const std::ffi::c_char) -> SQLITE_RESULT>,
    sqlite3_close :libloading::Symbol<'static,unsafe extern "C" fn(pdb:*mut std::ffi::c_void) -> std::ffi::c_void>,
    sqlite3_exec :libloading::Symbol<'static,unsafe extern "C" fn(pdb :*mut std::ffi::c_void,execstr :*const std::ffi::c_char,unsafe extern "C" fn(arg :*mut std::ffi::c_void,argc :std::ffi::c_int,argv :*const *const std::ffi::c_char) -> SQLITE_RESULT,arg :*mut std::ffi::c_void,pperrmsg:*const *mut std::ffi::c_char) -> SQLITE_RESULT>,
    sqlite3_free :libloading::Symbol<'static,unsafe extern "C" fn(errmsg :*mut std::ffi::c_char) -> std::ffi::c_void>,
}

impl Sqlite3Dll {
    fn load(dllname :&str) -> Result<Sqlite3Dll,Box<dyn Error>> {
        let mut hdl :libloading::Library;
        let mut openv2 :Option<libloading::Symbol<unsafe extern "C" fn(fname :* const std::ffi::c_char,ppdb :*const *mut std::ffi::c_void,flags :std::ffi::c_uint,connstr :*const std::ffi::c_char) -> SQLITE_RESULT>> = None;
        let mut closefn :Option< libloading::Symbol<unsafe extern "C" fn(pdb:*mut std::ffi::c_void) -> std::ffi::c_void> > = None;
        let mut execfn :Option< libloading::Symbol<unsafe extern "C" fn(pdb :*mut std::ffi::c_void,execstr :*const std::ffi::c_char,unsafe extern "C" fn(arg :*mut std::ffi::c_void,argc :std::ffi::c_int,argv :*const *const std::ffi::c_char) -> SQLITE_RESULT,arg :*mut std::ffi::c_void,pperrmsg:*const *mut std::ffi::c_char) -> SQLITE_RESULT> > = None;
        let mut freefn : Option<libloading::Symbol<unsafe extern "C" fn(errmsg :*mut std::ffi::c_char) -> std::ffi::c_void>> = None;

        hdl = unsafe { libloading::Library::new(dllname)?};
        unsafe {
            openv2 = Some(hdl.get("sqlite3_open_v2\0")?);
            closefn = Some(hdl.get("sqlite3_close\0")?);
            execfn = Some(hdl.get("sqlite3_exec\0")?);
            freefn = Some(hdl.get("sqlite3_free\0")?);
        }

        Ok(Sqlite3Dll {
            sqlite3_dll_hdl : hdl,
            sqlite3_open_v2 : openv2.unwrap().clone(),
            sqlite3_close : closefn.unwrap().clone(),
            sqlite3_exec : execfn.unwrap().clone(),
            sqlite3_free : freefn.unwrap().clone(),
        })
    }
}

