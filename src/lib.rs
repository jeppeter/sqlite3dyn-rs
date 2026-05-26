
use libloading;
use std::error::Error;


mod errors;
mod logger;

use crate::logger::*;

sqlite3dyn_error_class!{Sqlite3ConnError}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct SQLITE_RESULT(std::ffi::c_int);

impl PartialEq for SQLITE_RESULT {
    fn eq(&self,others :&SQLITE_RESULT) -> bool {
        return self.0 == others.0;
    }

    fn ne(&self,others :&SQLITE_RESULT) -> bool {
        return !self.eq(others);
    }
}


impl core::fmt::Debug for SQLITE_RESULT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl core::fmt::Display for SQLITE_RESULT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0,f)
    }
}

impl core::fmt::Binary for SQLITE_RESULT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Binary::fmt(&self.0,f)     
    }
}

impl core::fmt::Octal for SQLITE_RESULT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Octal::fmt(&self.0,f)      
    }
}

impl core::fmt::LowerHex for SQLITE_RESULT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.0,f)
    }
}

impl core::fmt::UpperHex for SQLITE_RESULT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.0,f)       
    }
}


const SQLITE3_OK :SQLITE_RESULT = SQLITE_RESULT(0);


pub struct Sqlite3Dll {
    sqlite3_dll_hdl :libloading::Library,
    sqlite3_open_v2 :Option<unsafe extern "C" fn(fname :* const std::ffi::c_char,ppdb :*const *mut std::ffi::c_void,flags :std::ffi::c_int,connstr :*const std::ffi::c_char) -> SQLITE_RESULT>,
    sqlite3_close : Option<unsafe extern "C" fn(pdb:*mut std::ffi::c_void) -> std::ffi::c_void>,
    sqlite3_exec : Option<unsafe extern "C" fn(pdb :*mut std::ffi::c_void,execstr :*const std::ffi::c_char,unsafe extern "C" fn(arg :*mut std::ffi::c_void,argc :std::ffi::c_int,argv :*const *const std::ffi::c_char, *const *const std::ffi::c_char) -> std::ffi::c_int,arg :*mut std::ffi::c_void,pperrmsg:*const *mut std::ffi::c_char) -> SQLITE_RESULT>,
    sqlite3_free : Option<unsafe extern "C" fn(errmsg :*mut std::ffi::c_char) -> std::ffi::c_void>,
}

impl Sqlite3Dll {
    #[allow(static_mut_refs)]
    pub fn load(dllname :& str) -> Result<Sqlite3Dll,Box<dyn Error>> {
        let hdl :libloading::Library;
        hdl = unsafe {libloading::Library::new(dllname)?};
        let mut retv :Self = Self {
            sqlite3_dll_hdl : hdl,
            sqlite3_open_v2 : None,
            sqlite3_close : None,
            sqlite3_exec: None,
            sqlite3_free : None,
        };

        let openv2 :libloading::Symbol<unsafe extern "C" fn(fname :* const std::ffi::c_char,ppdb :*const *mut std::ffi::c_void,flags :std::ffi::c_int,connstr :*const std::ffi::c_char) -> SQLITE_RESULT>;
        let closefn :libloading::Symbol<unsafe extern "C" fn(pdb:*mut std::ffi::c_void) -> std::ffi::c_void>;
        let execfn :libloading::Symbol<unsafe extern "C" fn(pdb :*mut std::ffi::c_void,execstr :*const std::ffi::c_char,unsafe extern "C" fn(arg :*mut std::ffi::c_void,argc :std::ffi::c_int,argv :*const *const std::ffi::c_char, *const *const std::ffi::c_char) -> std::ffi::c_int,arg :*mut std::ffi::c_void,pperrmsg:*const *mut std::ffi::c_char) -> SQLITE_RESULT>;
        let freefn :libloading::Symbol<unsafe extern "C" fn(errmsg :*mut std::ffi::c_char) -> std::ffi::c_void>;
        openv2 = unsafe {retv.sqlite3_dll_hdl.get("sqlite3_open_v2")?};
        retv.sqlite3_open_v2 = Some(*openv2);
        closefn = unsafe {retv.sqlite3_dll_hdl.get("sqlite3_close")?};
        retv.sqlite3_close = Some(*closefn);
        execfn = unsafe {retv.sqlite3_dll_hdl.get("sqlite3_exec")?};
        retv.sqlite3_exec = Some(*execfn);
        freefn = unsafe {retv.sqlite3_dll_hdl.get("sqlite3_free")?};
        retv.sqlite3_free = Some(*freefn);


        Ok(retv)
    }
}

const SQLITE_OPEN_READWRITE :std::ffi::c_int = 0x2;
const SQLITE_OPEN_CREATE :std::ffi::c_int = 0x4;

pub struct Sqlite3Conn<'a> {
    hdl :&'a Sqlite3Dll,    
    db :*mut std::ffi::c_void,
    nextarg :*mut std::ffi::c_void,
    callback :Option<fn(*mut std::ffi::c_void,vals :&Vec<String>,cols :&Vec<String>) -> Result<(),Box<dyn Error>> >,
}

impl<'a> Drop for Sqlite3Conn<'a> {
    #[allow(static_mut_refs)]
    fn drop(&mut self) {
        self.close();
        self.nextarg = std::ptr::null_mut();
        self.callback = None;
        return;
    }
}

unsafe extern "C" fn next_sqlite3_conn_exec_callback(ptr :*mut std::ffi::c_void,argc :std::ffi::c_int,vals :*const *const std::ffi::c_char,cols :*const *const std::ffi::c_char) -> std::ffi::c_int {
    let  conn :*mut Sqlite3Conn = ptr as *mut Sqlite3Conn;
    let mut valvecs :Vec<String> = vec![];
    let mut colvecs :Vec<String> = vec![];
    let mut i :std::ffi::c_int;
    let mut ores:Result<&str,std::str::Utf8Error>;

    let mut curptr :*const std::ffi::c_char;
    unsafe {
        if (*conn).callback.is_none() {
            return 0;
        }

        i = 0;
        while i < argc {
            curptr = *(vals.wrapping_add(i as usize));
            ores = std::ffi::CStr::from_ptr(curptr).to_str();
            if ores.is_err() {
                return -1;
            }
            valvecs.push(ores.unwrap().to_string());
            i += 1;
        }

        i = 0;
        while i < argc {
            curptr = *(cols.wrapping_add(i as usize));
            ores = std::ffi::CStr::from_ptr(curptr).to_str();
            if ores.is_err() {
                return -1;
            }
            colvecs.push(ores.unwrap().to_string());
            i += 1;
        }
    }

    unsafe {
        let ores2 = (*conn).callback.as_ref().unwrap()((*conn).nextarg,&valvecs,&colvecs);
        if ores2.is_err() {
            return -1;
        }
    }

    return 0;
}

impl<'a> Sqlite3Conn<'a> {
    //#[allow(unused_mut)]
    pub fn connect(_drv :&'a Sqlite3Dll,dsn :&str) -> Result<Sqlite3Conn<'a>,Box<dyn Error>> {
        let retv :Sqlite3Conn = Sqlite3Conn {
            hdl : _drv,
            db : std::ptr::null_mut(),
            nextarg : std::ptr::null_mut(),
            callback : None,
        };
        let mut sqlres : SQLITE_RESULT;
        let connstr :String;

        if retv.hdl.sqlite3_open_v2.is_none() {
            sqlite3dyn_new_error!{Sqlite3ConnError,"not init sqlite3_open_v2 yet"}
        }

        if retv.hdl.sqlite3_exec.is_none() {
            sqlite3dyn_new_error!{Sqlite3ConnError,"not init sqlite3_exec yet"}
        }

        if retv.hdl.sqlite3_close.is_none() {
            sqlite3dyn_new_error!{Sqlite3ConnError,"not init sqlite3_close yet"}
        }

        if retv.hdl.sqlite3_free.is_none() {
            sqlite3dyn_new_error!{Sqlite3ConnError,"not init sqlite3_free yet"}
        }


        connstr = format!("{}\0",dsn);
        unsafe {

            let ppdb :*const *mut std::ffi::c_void = &retv.db as *const *mut std::ffi::c_void;
            sqlres = retv.hdl.sqlite3_open_v2.as_ref().unwrap()(connstr.as_ptr() as *const std::ffi::c_char,ppdb,SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,std::ptr::null());
            if sqlres != SQLITE3_OK {
                //sqlres = sqlite3_open_v2.as_ref().unwrap()(connstr.as_ptr() as *const std::ffi::c_char,ppdb,SQLITE_OPEN_READWRITE,std::ptr::null());
                sqlres = retv.hdl.sqlite3_open_v2.as_ref().unwrap()(connstr.as_ptr() as *const std::ffi::c_char,ppdb,SQLITE_OPEN_READWRITE,std::ptr::null());
                if sqlres != SQLITE3_OK {
                    sqlite3dyn_new_error!{Sqlite3ConnError,"connect {} error {}", dsn,sqlres}
                }
            }
        }

        Ok(retv)
    }

    #[allow(unused_assignments)]
    pub fn exec(&mut self,sqlstr :&str,ptr :*mut std::ffi::c_void, callback :Option< fn(*mut std::ffi::c_void,vals :&Vec<String>,cols :&Vec<String>) -> Result<(),Box<dyn Error>> >) -> Result<(),Box<dyn Error>> {
        let sqlres :SQLITE_RESULT;
        let mut errmsg :*mut std::ffi::c_char = std::ptr::null_mut();
        let mut retores :Result<(),Box<dyn Error>> = Ok(());
        let sqlc :String = format!("{}\0",sqlstr);
        if self.db == std::ptr::null_mut() {
            sqlite3dyn_new_error!{Sqlite3ConnError,"not opened yet"}
        }

        self.nextarg = ptr;
        if callback.is_none() {
            self.callback = None;    
        } else {
            self.callback = Some(callback.as_ref().unwrap().clone());
        }

        unsafe {

            let pperrmsg :*const *mut std::ffi::c_char = &errmsg as *const *mut std::ffi::c_char;
            //sqlres = sqlite3_exec.as_ref().unwrap()(self.db, sqlc.as_ptr() as *const std::ffi::c_char,next_sqlite3_conn_exec_callback, self as *mut Sqlite3Conn as *mut std::ffi::c_void,pperrmsg);
            sqlres = self.hdl.sqlite3_exec.as_ref().unwrap()(self.db, sqlc.as_ptr() as *const std::ffi::c_char,next_sqlite3_conn_exec_callback, self as *mut Sqlite3Conn as *mut std::ffi::c_void,pperrmsg);
            if errmsg != std::ptr::null_mut() {
                let errsores = std::ffi::CStr::from_ptr(errmsg).to_str();
                if errsores.is_ok() {
                    let nerrs = errsores.unwrap().to_string();
                    sqlite3dyn_log_trace!("nerrs [{}]",nerrs);
                    retores = Err(sqlite3dyn_error_create!{Sqlite3ConnError,"exec {}\nstr {}",sqlstr,nerrs})    
                } else {
                    retores = Err(sqlite3dyn_error_create!{Sqlite3ConnError,"exec {}\nerror {}",sqlstr,sqlres})
                }

                self.hdl.sqlite3_free.as_ref().unwrap()(errmsg);
                errmsg = std::ptr::null_mut();
            } else if sqlres != SQLITE3_OK {
                retores = Err(sqlite3dyn_error_create!{Sqlite3ConnError,"exec {}\nerror {}",sqlstr,sqlres});
            }
        }

        return retores;
    }

    pub fn close(&mut self)  {
        let closefn :libloading::Symbol<unsafe extern "C" fn(pdb:*mut std::ffi::c_void) -> std::ffi::c_void>;
        if self.db != std::ptr::null_mut() {
            unsafe {
                let ores = self.hdl.sqlite3_dll_hdl.get("sqlite3_close");
                if ores.is_ok() {
                    closefn = ores.unwrap();
                    closefn(self.db);
                }
            }
            self.db = std::ptr::null_mut();
        }
        return;
    }

}

