mod feedback;
mod input;
mod objective;
mod observer;
mod output;

use std::{env, path::PathBuf};

use base64::prelude::*;
use clap::Parser;
use feedback::{go_cover::GoCoverFeedback, type_state::TypeStateFeedback};
use input::ByteCodeInput;
use libafl::prelude::*;
use libafl_bolts::{
    current_nanos,
    rands::StdRand,
    tuples::{tuple_list, Handled},
};
use observer::GoCoverObserver;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Report error if VMs exit with different status. Can result in many false positives!",
        name = "DETECT_STATUS_DIFFERENCE",
        default_value_t = false,
    )]
    detect_status_diff: bool,
}

fn main() {
    let args = Args::parse();

    let neogo_stdout_observer = StdOutObserver::new("neogo-stdout-observer");
    let neosharp_stdout_observer = StdOutObserver::new("neosharp-stdout-observer");

    let mut objective = objective::DiffStdOutObjective::new(
        neogo_stdout_observer.handle(),
        neosharp_stdout_observer.handle(),
        args.detect_status_diff,
    );

    let temp_dir = env::temp_dir().join("N3onDiff");
    std::fs::create_dir(temp_dir.as_path()).unwrap_or(());

    let go_cover_dir = temp_dir.join("go-cover");
    std::fs::create_dir(go_cover_dir.as_path()).unwrap_or(());
    let go_cover_observer = GoCoverObserver::new(go_cover_dir.clone().into_boxed_path());

    let mut feedback = feedback_or!(
        TypeStateFeedback::new(vec![
            neogo_stdout_observer.handle(),
            neosharp_stdout_observer.handle(),
        ]),
        GoCoverFeedback::new(go_cover_observer.handle())
    );

    let neogo_executor = CommandExecutor::builder()
        .program("./harness/neo-go")
        .env("GOCOVERDIR", go_cover_dir.as_path())
        .arg_input_arg()
        .arg("DUMMY")
        .stdout_observer(neogo_stdout_observer.handle())
        .build(tuple_list!(neogo_stdout_observer, go_cover_observer))
        .unwrap();

    let neosharp_executor = CommandExecutor::builder()
        .program("./harness/neo-sharp")
        .arg_input_arg()
        .arg("DUMMY")
        .stdout_observer(neosharp_stdout_observer.handle())
        .build(tuple_list!(neosharp_stdout_observer))
        .unwrap();

    let mut executor = DiffExecutor::new(neogo_executor, neosharp_executor, ());

    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        InMemoryCorpus::new(),
        OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
        &mut feedback,
        &mut objective,
    )
    .unwrap();

    let monitor = SimpleMonitor::new(|s| println!("{s}"));
    let mut manager = SimpleEventManager::new(monitor);

    let corpus = state.corpus_mut();

    corpus
        .add(Testcase::new(ByteCodeInput {
            opcodes: BASE64_STANDARD.decode("DAxIZWxsbyB3b3JsZCE=").unwrap(),
        }))
        .unwrap();

    let scheduler = QueueScheduler::new();
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    loop {
        println!(
            "{:?}",
            fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut manager)
        )
    }
}
