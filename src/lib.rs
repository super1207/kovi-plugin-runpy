mod pytool;

use std::ffi::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use std::path::PathBuf;

use kovi::bot::runtimebot::kovi_api::KoviApi;
use kovi::serde_json::json;
use kovi::PluginBuilder;
use pytool::run_local_python;
use pytool::run_virtual_python;

#[macro_use]
extern crate lazy_static; 

lazy_static! {
    static ref G_PATH:std::sync::RwLock<Option<PathBuf>>  = std::sync::RwLock::new(None);
}



#[no_mangle]
pub extern "system" fn a9d0d1038bfd4e2b9543d2ef67101731_run_local_python(code: *const c_char,input: *const c_char,app_dir: *const c_char) -> *mut c_char {

    let code_t = unsafe { CStr::from_ptr(code).to_str().unwrap() };
    let input_t = unsafe { CStr::from_ptr(input).to_str().unwrap() };
    let app_dir_t = unsafe { CStr::from_ptr(app_dir).to_str().unwrap() };
    let app_dir_t2 = PathBuf::from(app_dir_t);
    let ret_rst = run_local_python(code_t,input_t,app_dir_t2);
    let ret = match ret_rst {
        Ok(ret) => {
            json!({
                "retcode":0,
                "data":ret,
            })
           
        },
        Err(err) => {
            json!({
                "retcode":-1,
                "data":err.to_string(),
            })
        },
    };
    let ret_str = ret.to_string();
    
    let ret_cstr = CString::new(ret_str).unwrap();
    return ret_cstr.into_raw();
}


#[no_mangle]
pub extern "system" fn a9d0d1038bfd4e2b9543d2ef67101731_run_virtual_python(code: *const c_char,input: *const c_char,app_dir: *const c_char,app_flag: *const c_char) -> *mut c_char {

    let code_t = unsafe { CStr::from_ptr(code).to_str().unwrap() };
    let input_t = unsafe { CStr::from_ptr(input).to_str().unwrap() };
    let app_dir_t = unsafe { CStr::from_ptr(app_dir).to_str().unwrap() };
    let app_dir_t2 = PathBuf::from(app_dir_t);
    
    
    let tmp_dir_t2;
    {
        let lk = G_PATH.read().unwrap();
        if let Some(tmp_dir_t) = &*lk {
            tmp_dir_t2 = tmp_dir_t.clone();
        }else{
            let ret = json!({
                "retcode":-2,
                "data":"python plugin not init"
            });
            let ret_str = ret.to_string();
            let ret_cstr = CString::new(ret_str).unwrap();
            return ret_cstr.into_raw();
        }
    }    
    let app_flag_t = unsafe { CStr::from_ptr(app_flag).to_str().unwrap() };
    
    let ret_rst = run_virtual_python(code_t,input_t,app_dir_t2,tmp_dir_t2,app_flag_t);
    let ret = match ret_rst {
        Ok(ret) => {
            json!({
                "retcode":0,
                "data":ret,
            })
           
        },
        Err(err) => {
            json!({
                "retcode":-1,
                "data":err.to_string(),
            })
        },
    };
    let ret_str = ret.to_string();
    
    let ret_cstr = CString::new(ret_str).unwrap();
    return ret_cstr.into_raw();
}

#[no_mangle]
pub extern "system" fn a9d0d1038bfd4e2b9543d2ef67101731_free(ptr: *mut c_char) {
    unsafe { let _ = CString::from_raw(ptr); };
}

#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    let tmp_dir_t2 = bot.get_data_path();
    {
        let mut lk = G_PATH.write().unwrap();
        *lk = Some(tmp_dir_t2);
    }
}
