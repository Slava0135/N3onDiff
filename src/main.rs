use std::process::Command;

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
    println!("NeoGo output: {}", String::from_utf8(neo_go_output.stdout).unwrap());
    println!("NeoSharp output: {}", String::from_utf8(neo_sharp_output.stdout).unwrap());
}
