mod feedback;
mod objective;
mod output;

use std::path::PathBuf;

use feedback::TypeStateFeedback;
use libafl::prelude::*;
use libafl_bolts::{
    current_nanos,
    rands::StdRand,
    tuples::{tuple_list, Handled},
};

fn main() {
    let neogo_stdout_observer = StdOutObserver::new("neogo-stdout-observer");
    let neosharp_stdout_observer = StdOutObserver::new("neosharp-stdout-observer");

    let mut objective = objective::DiffStdOutObjective {
        fst_stdout_observer: neogo_stdout_observer.handle(),
        snd_stdout_observer: neosharp_stdout_observer.handle(),
    };

    let mut feedback = TypeStateFeedback::new(vec![
        neogo_stdout_observer.handle(),
        neosharp_stdout_observer.handle(),
    ]); // <-- you need this!!!

    let neogo_executor = CommandExecutor::builder()
        .program("./harness/neo-go")
        .arg_input_arg()
        .arg("DUMMY")
        .stdout_observer(neogo_stdout_observer.handle())
        .build(tuple_list!(neogo_stdout_observer))
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

    state
        .corpus_mut()
        .add(Testcase::new(BytesInput::from(
            "DAxIZWxsbyB3b3JsZCE=".as_bytes().to_vec(),
        )))
        .unwrap();

    let scheduler = QueueScheduler::new();
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut stages = tuple_list!(StdMutationalStage::new(NopMutator::new(
        MutationResult::Mutated
    )));

    let corpus_id = fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut manager)
        .unwrap();
}
