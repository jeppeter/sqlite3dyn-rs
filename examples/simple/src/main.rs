use sqlite3dyn;
use std::error::Error;

struct SqlRecord {
    cnt :i32,
}

fn call_sql_callback(ptr :*mut std::ffi::c_void,vals :&Vec<String>,cols :&Vec<String>) -> Result<(),Box<dyn Error>> {
    let sqlr :*mut SqlRecord = ptr as *mut SqlRecord;
    let mut idx :usize;
    let mut width :usize = 1;

    idx = 0;
    while idx < vals.len() {
        if vals[idx].len() >= width {
            width = vals[idx].len() + 1;
        }
        idx += 1;
    }

    idx = 0;
    while idx < cols.len() {
        if cols[idx].len() >= width {
            width = cols[idx].len() + 1;
        }
        idx += 1;
    }


    idx = 0;
    while idx < vals.len() {
        print!("{:width$}",vals[idx]);
        idx += 1;
    }

    print!("\n");

    idx = 0;
    while idx < cols.len() {
        print!("{:width$}",cols[idx]);
        idx += 1;
    }
    print!("\n");


    unsafe {
        (*sqlr).cnt += 1;
    }
    Ok(())
}


fn main() -> Result<(),Box<dyn Error>> {
    let sqldrv :sqlite3dyn::Sqlite3Dll = sqlite3dyn::Sqlite3Dll::load("./sqlite3.dll")?;
    let mut sqlconn :sqlite3dyn::Sqlite3Conn = sqlite3dyn::Sqlite3Conn::connect(&sqldrv,"./test.db")?;
    let sqlstr = format!("select * from cctable;");
    let sqlr :SqlRecord = SqlRecord{cnt : 0};


    let _ = sqlconn.exec(&sqlstr,((&sqlr) as *const SqlRecord) as *mut std::ffi::c_void,Some(call_sql_callback))?;
    println!("exec on [{}] succ cnt {}",sqlstr,sqlr.cnt);

    Ok(())
}
