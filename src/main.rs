use std::process::Command;

#[derive(Debug)]
#[repr(i32)]
enum ExitCode {
    VmHaltedCode = 0,
    WrongArgCode = 1,
    WrongStrCode = 2,
    RunErrorCode = 3,
    VmFailedCode = 4,
}

impl From<i32> for ExitCode {
    fn from(x: i32) -> Self {
        match x {
            x if x == ExitCode::VmHaltedCode as i32 => ExitCode::VmHaltedCode,
            x if x == ExitCode::WrongArgCode as i32 => ExitCode::WrongArgCode,
            x if x == ExitCode::WrongStrCode as i32 => ExitCode::WrongStrCode,
            x if x == ExitCode::RunErrorCode as i32 => ExitCode::RunErrorCode,
            x if x == ExitCode::VmFailedCode as i32 => ExitCode::VmFailedCode,
            _ => panic!("unknown exit code")
        }
    }
}

fn main() {
    let input = "DAxIZWxsbyB3b3JsZCE=";
    let neo_go_output = Command::new("./harness/neo-go")
        .args([input])
        .output()
        .expect("failed to execute process");
    let neo_sharp_output = Command::new("./harness/neo-sharp")
        .args([input])
        .output()
        .expect("failed to execute process");
    let neo_go_exit_code: ExitCode = neo_go_output.status.code().unwrap_or_default().into();
    let neo_sharp_exit_code: ExitCode = neo_sharp_output.status.code().unwrap_or_default().into();
    println!("NeoGo exit code: {:?}", neo_go_exit_code);
    println!("NeoSharp exit code: {:?}", neo_sharp_exit_code);
}
