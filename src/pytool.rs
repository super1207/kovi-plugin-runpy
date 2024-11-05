use std::{fs, io::Write, path::{Path, PathBuf}};

const BASE64_CUSTOM_ENGINE: engine::GeneralPurpose = engine::GeneralPurpose::new(&alphabet::STANDARD, general_purpose::PAD);
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use encoding::Encoding;
use md5::Md5;
use md5::Digest;


fn get_python_cmd_name() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(not(windows))]
    return Ok("python3".to_owned());

    #[cfg(windows)]
    return Ok("python".to_owned());
}
fn get_local_python_uid() -> Result<String,Box<dyn std::error::Error>> {


    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut command = std::process::Command::new(get_python_cmd_name().unwrap());

    #[cfg(windows)]
    let output = command.creation_flags(0x08000000).arg("-c").arg("import sys; print(sys.version)").output()?;

    #[cfg(not(windows))]
    let output = command.arg("-c").arg("import sys; print(sys.version)").output()?;

    let version = 
    if cfg!(target_os = "windows") {
        encoding::all::GBK.decode(&output.stdout, encoding::DecoderTrap::Ignore)?
    }else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    
    let mut hasher = Md5::new();
    hasher.update(version.trim().to_string().as_bytes());
    let result = hasher.finalize();
    let mut content = String::new();
    for ch in result {
        content.push_str(&format!("{:02x}",ch));
    }
    return Ok(content);
}


    // py解析red变量
 const G_RED_PY_DECODE:&str = r#"
def __red_py_decode(input:str):
    return input
def __to_red_type(input):
    if isinstance(input,str):
        return input
    return str(input)
"#;


pub fn run_local_python(code:&str,input:&str,app_dir:PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let code1 = r#"
import os
import sysconfig
import sys

def myprint(*args,**kwargs):
    pass

red_print = sys.stdout.write

sys.stdout.write = myprint

def red_in():
    import base64
    inn = input()
    sw = base64.b64decode(inn).decode()
    return __red_py_decode(sw)

def red_out(sw):
    import base64
    en = base64.b64encode(__to_red_type(sw).encode()).decode()
    red_print(en)
"#;
        
        let input_b64 = BASE64_CUSTOM_ENGINE.encode(input);

        let pip_in = std::process::Stdio::piped();

        let red_py_decode = G_RED_PY_DECODE.to_owned();

        fs::create_dir_all(app_dir.clone())?;


        #[cfg(windows)]
        use std::os::windows::process::CommandExt;

        #[cfg(not(windows))]
        let mut p = std::process::Command::new(get_python_cmd_name().unwrap())
        .stdin(pip_in)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(app_dir)
        .arg("-c")
        .arg(format!("{red_py_decode}{code1}{code}"))
        .spawn()?;


        #[cfg(windows)]
        let mut p = std::process::Command::new(get_python_cmd_name().unwrap()).creation_flags(0x08000000)
        .stdin(pip_in)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(app_dir)
        .arg("-c")
        .arg(format!("{red_py_decode}{code1}{code}"))
        .spawn()?;

        let s = p.stdin.take();
        if s.is_none() {
            p.kill()?;
        }else {
            s.unwrap().write_all(input_b64.as_bytes())?;
        }
        let output = p.wait_with_output()?;
        let out = String::from_utf8_lossy(&output.stdout).to_string();
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        if err.contains("Traceback (most recent call last):") {
            return Err(err.into());
        }
        let content_rst = base64::Engine::decode(&base64::engine::GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            base64::engine::general_purpose::PAD), &out);
        if content_rst.is_err() {
            return Err(out.into());
        }
        Ok(String::from_utf8(content_rst.unwrap())?)
}

pub fn run_virtual_python(code:&str,input:&str,app_dir:PathBuf,tmp_dir:PathBuf,app_flag:&str) -> Result<String, Box<dyn std::error::Error>> {
    let code1 = r#"
import os
import sysconfig
import sys

def myprint(*args,**kwargs):
    pass

red_print = sys.stdout.write

sys.stdout.write = myprint

def red_install(pkg_name):
    '''install a pkg'''
    from pip._internal.cli import main
    ret = main.main(['install', pkg_name, '-i',
                    'https://pypi.tuna.tsinghua.edu.cn/simple', "--no-warn-script-location"])
    if ret != 0:
        err = "pip install {} failed".format(pkg_name)
        raise Exception(err)

def red_in():
    import base64
    inn = input()
    sw = base64.b64decode(inn).decode('utf-8')
    return __red_py_decode(sw)

def red_out(sw):
    import base64
    en = base64.b64encode(__to_red_type(sw).encode('utf-8')).decode('utf-8')
    red_print(en)
"#;
    let input_b64 = BASE64_CUSTOM_ENGINE.encode(input);

    let python_id = get_local_python_uid()?;

    let python_dir = tmp_dir.join(format!("pymain_{app_flag}_{python_id}"));

    fs::create_dir_all(app_dir.clone())?;
    fs::create_dir_all(&python_dir)?;

    let python_env_is_create;
    if Path::new(&python_dir).join("redpymainok").is_file(){
        python_env_is_create = true;
    }else{
        python_env_is_create = false;
    }

    #[cfg(windows)]
    use std::os::windows::process::CommandExt;


    if !python_env_is_create {
        

        #[cfg(windows)]
        let foo = std::process::Command::new(get_python_cmd_name().unwrap()).creation_flags(0x08000000).current_dir(python_dir.clone()).arg("-m").arg("venv").arg("pymain").status();
        
        #[cfg(not(windows))]
        let foo = std::process::Command::new(get_python_cmd_name().unwrap()).current_dir(python_dir.clone()).arg("-m").arg("venv").arg("pymain").status();

        if foo.is_err() {
            return Err(format!("create python env err:{:?}",foo.err()).into());
        } else {
            let f = foo.unwrap();
            let is_ok = f.clone().success();
            if !is_ok {
                return Err(format!("create python env err:{:?}",f).into());
            } else {
                let mut f = fs::File::create(Path::new(&python_dir).join("redpymainok"))?;
                std::io::Write::write_all(&mut f, &[])?;
            }
        }
    }

    let curr_env = std::env::var("PATH").unwrap_or_default();

    let new_env = if cfg!(target_os = "windows") {
        format!("{}pymain/Scripts;{}",python_dir.to_string_lossy().to_string(),curr_env)
    } else {
        format!("{}pymain/bin:{}",python_dir.to_string_lossy().to_string(),curr_env)
    };
    let pip_in = std::process::Stdio::piped();

    let red_py_decode = G_RED_PY_DECODE.to_owned();

    #[cfg(windows)]
    let mut p = std::process::Command::new(get_python_cmd_name().unwrap()).creation_flags(0x08000000)
    .stdin(pip_in)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .current_dir(app_dir)
    .env("PATH", new_env)
    .arg("-c")
    .arg(format!("{red_py_decode}{code1}{code}"))
    .spawn()?;


    #[cfg(not(windows))]
    let mut p = std::process::Command::new(get_python_cmd_name().unwrap())
    .stdin(pip_in)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .current_dir(app_dir)
    .env("PATH", new_env)
    .arg("-c")
    .arg(format!("{red_py_decode}{code1}{code}"))
    .spawn()?;

    let s = p.stdin.take();
    if s.is_none() {
        p.kill()?;
    }else {
        s.unwrap().write_all(input_b64.as_bytes())?;
    }
    let output = p.wait_with_output()?;
    let out = String::from_utf8_lossy(&output.stdout).to_string();
    let err = String::from_utf8_lossy(&output.stderr).to_string();
    if err.contains("Traceback (most recent call last):") {
        return Err(err.into());
    }
    let content_rst = base64::Engine::decode(&base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD), &out);
    if content_rst.is_err() {
        return Err(out.into());
    }
    Ok(String::from_utf8(content_rst.unwrap())?)
}