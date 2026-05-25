

use std::env;
use std::io::{Write};
use std::fs;
//use std::io::prelude::*;
use lazy_static::lazy_static;
use chrono::{Local,Timelike,Datelike};
use std::sync::RwLock;



fn _sqlite3dyn_get_environ_var(envname :&str) -> String {
	match env::var(envname) {
		Ok(v) => {
			format!("{}",v)
		},
		Err(_e) => {
			String::from("")
		}
	}
}

#[allow(dead_code)]
struct LogVar {
	level :i32,
	nostderr : bool,
	wfile : Option<fs::File>,
	wfilename :String,
	baklevel :i32,
	baknostderr :bool,
}


fn sqlite3dyn_proc_log_init(prefix :&str) -> LogVar {
	let mut getv :String;
	let mut retv :i32 = 0;
	let mut nostderr :bool = false;
	let mut coptfile :Option<fs::File> = None;
	let mut key :String;
	let mut fname :String = "".to_string();

	key = format!("{}_LEVEL", prefix);
	getv = _sqlite3dyn_get_environ_var(&key);
	if getv.len() > 0 {
		match getv.parse::<i32>() {
			Ok(v) => {
				retv = v;
			},
			Err(e) => {
				retv = 0;
				eprintln!("can not parse [{}] error[{}]", getv,e);
			}
		}
	}

	key = format!("{}_NOSTDERR",prefix);
	getv = _sqlite3dyn_get_environ_var(&key);
	if getv.len() > 0 {
		nostderr = true;
	}



	key = format!("{}_LOGFILE",prefix);
	getv = _sqlite3dyn_get_environ_var(&key);
	if getv.len() > 0 {
		fname = format!("{}",getv);
		let fo = fs::File::create(&getv);
		if fo.is_err() {
			eprintln!("can not open [{}]", getv);
		} else {
			coptfile = Some(fo.unwrap());
		}
	}

	return LogVar {
		level : retv,
		nostderr : nostderr,
		wfile : coptfile,
		wfilename : fname,
		baklevel : 0,
		baknostderr : true,
	};
}


lazy_static! {
	static ref SQLITE3DYN_LOG_LEVEL : RwLock<LogVar> = {
	 	RwLock::new(sqlite3dyn_proc_log_init("SQLITE3DYN"))
	};
}

///  to let not debug output the values
#[allow(dead_code)]
pub fn set_sqlite3dyn_logger_disable() {
	let mut sqlite3dynref = SQLITE3DYN_LOG_LEVEL.write().unwrap();
	sqlite3dynref.baknostderr = sqlite3dynref.nostderr;
	sqlite3dynref.baklevel = sqlite3dynref.level;
	sqlite3dynref.wfile = None;
	sqlite3dynref.level = 0;
	sqlite3dynref.nostderr = true;
	return;
}

#[allow(dead_code)]
pub fn set_sqlite3dyn_logger_enable() {
	let mut sqlite3dynref = SQLITE3DYN_LOG_LEVEL.write().unwrap();
	sqlite3dynref.level = sqlite3dynref.baklevel;
	sqlite3dynref.nostderr = sqlite3dynref.baknostderr;	
	if sqlite3dynref.wfilename.len() > 0 {
		let fo = fs::File::create(&sqlite3dynref.wfilename);
		if fo.is_ok() {
			sqlite3dynref.wfile = Some(fo.unwrap());
		}
	}
	return ;
}


#[allow(dead_code)]
pub (crate)  fn sqlite3dyn_debug_out(level :i32, outs :&str) {
	let sqlite3dynref = SQLITE3DYN_LOG_LEVEL.write().unwrap();
	if sqlite3dynref.level >= level {
		let c = format!("{}\n",outs);
		if !sqlite3dynref.nostderr {
			let _ = std::io::stderr().write_all(c.as_bytes());
		}

		if sqlite3dynref.wfile.is_some() {
			let mut wf = sqlite3dynref.wfile.as_ref().unwrap();
			let _ = wf.write(c.as_bytes());
		}
	}
	return;
}

#[allow(dead_code)]
pub (crate) fn sqlite3dyn_log_get_timestamp() -> String {
	let now = Local::now();
	return format!("{}/{}/{} {}:{}:{}",now.year(),now.month(),now.day(),now.hour(),now.minute(),now.second());
}

/// call when SQLITE3DYN_LEVEL >= 0
/// ```
/// sqlite3dyn_log_error!("exampl error value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_log_error {
	($($arg:tt)+) => {
		let mut c :String= format!("[SQLITE3DYN]<ERROR>{}[{}:{}]  ",sqlite3dyn_log_get_timestamp(),file!(),line!());
		c.push_str(&(format!($($arg)+)[..]));
		sqlite3dyn_debug_out(0,&c);
	}
}

/// call when SQLITE3DYN_LEVEL >= 10
/// ```
/// sqlite3dyn_log_warn!("exampl error value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_log_warn {
	($($arg:tt)+) => {
		let mut c :String= format!("[SQLITE3DYN]<WARN>{}[{}:{}]  ",sqlite3dyn_log_get_timestamp(),file!(),line!());
		c.push_str(&(format!($($arg)+)[..]));
		sqlite3dyn_debug_out(10,&c);
	}
}

/// call when SQLITE3DYN_LEVEL >= 20
/// ```
/// sqlite3dyn_log_info!("exampl error value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_log_info {
	($($arg:tt)+) => {
		let mut c :String= format!("[SQLITE3DYN]<INFO>{}[{}:{}]  ",sqlite3dyn_log_get_timestamp(),file!(),line!());
		c.push_str(&(format!($($arg)+)[..]));
		sqlite3dyn_debug_out(20,&c);
	}
}

/// check whether expr == true ,if not so call panic
/// ```
/// sqlite3dyn_assert!(1 == 1 , "can not call 1 == {}", 0);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_assert {
	($v:expr , $($arg:tt)+) => {
		if !($v) {
			let mut _c :String= format!("[SQLITE3DYN][{}:{}] ",file!(),line!());
			_c.push_str(&(format!($($arg)+)[..]));
			panic!("{}", _c);
		}
	}
}


/// format buffer log
/// ```
/// let buf :Vec<u8> = vec![3,2,4];
/// sqlite3dyn_format_buffer_log!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_format_buffer_log {
	($buf:expr,$len:expr,$info:tt,$iv:expr,$($arg:tt)+) => {
		let mut c :String = format!("[SQLITE3DYN][{}:{}]",file!(),line!());
		c.push_str(&format!("{} ",$info));
		c.push_str(&sqlite3dyn_log_get_timestamp());
		c.push_str(": ");
		c.push_str(&(format!($($arg)+)[..]));
		let _ptr :*const u8 = $buf as *const u8;
		let  mut _ci :usize;
		let _totallen: usize = $len as usize;
		let mut _lasti :usize = 0;
		let mut _nb :u8;
		c.push_str(&format!(" buffer [{:?}][{}]",_ptr,_totallen));
		_ci = 0;
		while _ci < _totallen {
			if (_ci % 16) == 0 {
				if _ci > 0 {
					c.push_str("    ");
					while _lasti < _ci {
						unsafe{
							_nb = *_ptr.offset(_lasti as isize);	
						}
						
						if _nb >= 0x20 && _nb <= 0x7e {
							c.push(_nb as char);
						} else {
							c.push_str(".");
						}
						_lasti += 1;
					}
				}
				c.push_str(&format!("\n0x{:08x}:", _ci));
			}
			unsafe {_nb = *_ptr.offset(_ci as isize);}			
			c.push_str(&format!(" 0x{:02x}",_nb));
			_ci += 1;
		}

		if _lasti < _ci {
			while (_ci % 16) != 0 {
				c.push_str("     ");
				_ci += 1;
			}

			c.push_str("    ");

			while _lasti < _totallen {
				unsafe {_nb = *_ptr.offset(_lasti as isize);}				
				if _nb >= 0x20 && _nb <= 0x7e {
					c.push(_nb as char);
				} else {
					c.push_str(".");
				}
				_lasti += 1;
			}
			//c.push_str("\n");
		}
		sqlite3dyn_debug_out($iv,&c);
	}
}

/// call debug buffer when SQLITE3DYN_LEVEL >= 0
/// ```
/// let buf :Vec<u8> = vec![3,2,4];
/// sqlite3dyn_format_buffer_error!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_debug_buffer_error {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		sqlite3dyn_format_buffer_log!($buf,$len,"<ERROR>",0,$($arg)+);
	}
}

/// call debug buffer when SQLITE3DYN_LEVEL >= 10
/// ```
/// let buf :Vec<u8> = vec![3,2,4];
/// sqlite3dyn_format_buffer_warn!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_debug_buffer_warn {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		sqlite3dyn_format_buffer_log!($buf,$len,"<WARN>",10,$($arg)+);
	}
}

/// call debug buffer when SQLITE3DYN_LEVEL >= 20
/// ```
/// let buf :Vec<u8> = vec![3,2,4];
/// sqlite3dyn_format_buffer_info!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_debug_buffer_info {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		sqlite3dyn_format_buffer_log!($buf,$len,"<INFO>",20,$($arg)+);
	}
}

/// call debug buffer when SQLITE3DYN_LEVEL >= 30
/// ```
/// let buf :Vec<u8> = vec![3,2,4];
/// sqlite3dyn_format_buffer_debug!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_debug_buffer_debug {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		sqlite3dyn_format_buffer_log!($buf,$len,"<DEBUG>",30,$($arg)+);
	}
}



/// call debug buffer when SQLITE3DYN_LEVEL >= 40
/// ```
/// sqlite3dyn_log_trace!("call value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_log_trace {
	($($arg:tt)+) => {
		let mut _c :String= format!("[SQLITE3DYN]<TRACE>{}[{}:{}]  ",sqlite3dyn_log_get_timestamp(),file!(),line!());
		_c.push_str(&(format!($($arg)+)[..]));
		sqlite3dyn_debug_out(40, &_c);
	}
}

/// call debug buffer when SQLITE3DYN_LEVEL >= 40
/// ```
/// sqlite3dyn_log_trace!("call value {}",3);
/// ```
// #[macro_export]
// #[allow(unused_macros)]
// #[cfg(not(feature="debug_mode"))]
// macro_rules! sqlite3dyn_log_trace {
// 	($($arg:tt)+) => {}
// }


/// call debug buffer when SQLITE3DYN_LEVEL >= 40
/// ```
/// let buf :Vec<u8> = vec![3,2,4];
/// sqlite3dyn_format_buffer_trace!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
/// ```
#[macro_export]
#[allow(unused_macros)]
macro_rules! sqlite3dyn_debug_buffer_trace {
	($buf:expr,$len:expr,$($arg:tt)+) => {
		sqlite3dyn_format_buffer_log!($buf,$len,"<TRACE>",40,$($arg)+);
	}
}

// call debug buffer when SQLITE3DYN_LEVEL >= 40
// ```
// let buf :Vec<u8> = vec![3,2,4];
// sqlite3dyn_format_buffer_trace!(buf.as_ptr(),buf.len(),"buffer output value {}",3);
// ```
// #[macro_export]
// #[allow(unused_macros)]
// #[cfg(not(feature="debug_mode"))]
// macro_rules! sqlite3dyn_debug_buffer_trace {
// 	($buf:expr,$len:expr,$($arg:tt)+) => {}
// }
