#[allow(unused_imports)]
use extargsparse_codegen::{extargs_load_commandline,ArgSet,extargs_map_function};
#[allow(unused_imports)]
use extargsparse_worker::{extargs_error_class,extargs_new_error};
#[allow(unused_imports)]
use extargsparse_worker::namespace::{NameSpaceEx};
#[allow(unused_imports)]
use extargsparse_worker::argset::{ArgSetImpl};
use extargsparse_worker::parser::{ExtArgsParser};
use extargsparse_worker::funccall::{ExtArgsParseFunc};


use std::cell::RefCell;
use std::sync::Arc;
use std::error::Error;
use std::boxed::Box;
#[allow(unused_imports)]
use regex::Regex;
#[allow(unused_imports)]
use std::any::Any;

use lazy_static::lazy_static;
use std::collections::HashMap;

#[allow(unused_imports)]
use extlog::{debug_trace,debug_buffer_trace,format_buffer_log,format_str_log};
#[allow(unused_imports)]
use extlog::loglib::{log_get_timestamp,log_output_function};

use super::logtrans::{init_log};
use sqlite3dyn::*;

extargs_error_class!{SqltstError}

struct SqlRecord {
	cnt :i32,
}

fn call_sql_callback(ptr :*mut std::ffi::c_void,vals :&Vec<String>,cols :&Vec<String>) -> Result<(),Box<dyn Error>> {
	let sqlr :*mut SqlRecord = ptr as *mut SqlRecord;
	let mut idx :usize;

	idx = 0;
	while idx < vals.len() {
		println!("vals.[{}]=[{}]",idx,vals[idx]);
		idx += 1;
	}

	idx = 0;
	while idx < cols.len() {
		println!("cols.[{}]=[{}]",idx,cols[idx]);
		idx += 1;
	}


	unsafe {
		(*sqlr).cnt += 1;
	}
	Ok(())
}

fn sqlexec_handler(ns :NameSpaceEx,_optargset :Option<Arc<RefCell<dyn ArgSetImpl>>>,_ctx :Option<Arc<RefCell<dyn Any>>>) -> Result<(),Box<dyn Error>> {
	let sarr :Vec<String>;
	let sqldll :String;
	let mut idx:usize;
	let sqlr :SqlRecord = SqlRecord{cnt:0};

	init_log(ns.clone())?;

	sarr = ns.get_array("subnargs");

	if sarr.len() < 3 {
		extargs_new_error!{SqltstError,"need dbfile sqls..."}
	}

	sqldll = ns.get_string("sqldll");

	let sqldrv :sqlite3dyn::Sqlite3Dll = Sqlite3Dll::load(&sqldll)?;
	let mut sqlconn :sqlite3dyn::Sqlite3Conn = Sqlite3Conn::connect(&sqldrv,&sarr[0])?;

	idx = 1;
	while idx < sarr.len() {
		let _ = sqlconn.exec(&sarr[idx],((&sqlr) as *const SqlRecord) as *mut std::ffi::c_void,Some(call_sql_callback))?;
		idx += 1;
	}


	println!("exec on [{}] succ",sarr[0]);

	Ok(())
}


#[extargs_map_function(sqlexec_handler)]
pub fn load_sql_handler(parser :ExtArgsParser) -> Result<(),Box<dyn Error>> {
	let mut sqldll :String = "".to_string();
	let reg :regex::Regex = regex::Regex::new("\\\\")?;
	let cmdline :String;

	println!("os [{}]",std::env::consts::OS);
	sqldll = ".\\sqlite3.dll".to_string();
	sqldll = reg.replace_all(&sqldll,"\\\\").to_string();
	cmdline = format!(r#"{{
		"sqldll" : "{}"
		"sqlexec<sqlexec_handler>##dbfile sqlstr ... to execute sql##" : {{
			"$" : "+"
		}}
	}}"#,sqldll);
	extargs_load_commandline!(parser,&cmdline)?;
	Ok(())
}