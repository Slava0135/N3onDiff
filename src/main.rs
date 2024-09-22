use std::process::Command;
use serde_json::{Result, Value};

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

    let neo_go_output = String::from_utf8(neo_go_output.stdout).unwrap();
    let neo_sharp_output = String::from_utf8(neo_sharp_output.stdout).unwrap();
    println!("NeoGo output: {}", neo_go_output);
    println!("NeoSharp output: {}", neo_sharp_output);

    let neo_go_value: Value = serde_json::from_str(&neo_go_output).unwrap();
    let neo_sharp_value: Value = serde_json::from_str(&neo_sharp_output).unwrap();
    if neo_go_value == neo_sharp_value {
        println!("-- outputs are equal --");
    } else {
        println!("-- outputs are different --")
    }
}
