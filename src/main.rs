mod objective;

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

    let mut feedback = ();

    let neogo_executor = CommandExecutor::builder()
        .program("./harness/neo-go")
        .arg_input_arg()
        .stdout_observer(neogo_stdout_observer.handle())
        .build(tuple_list!(neogo_stdout_observer))
        .unwrap();

    let neosharp_executor = CommandExecutor::builder()
        .program("./harness/neo-sharp")
        .arg_input_arg()
        .stdout_observer(neosharp_stdout_observer.handle())
        .build(tuple_list!(neosharp_stdout_observer))
        .unwrap();

    let mut executor = DiffExecutor::new(neogo_executor, neosharp_executor, ());

    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        InMemoryCorpus::new(),
        InMemoryCorpus::new(),
        &mut feedback,
        &mut objective,
    )
    .unwrap();

    state
        .corpus_mut()
        .add(Testcase::new(BytesInput::from(
            "DAxIZWxsbyB3b3JsZCE=".as_bytes().to_vec(),
        )))
        .unwrap();

    let scheduler = QueueScheduler::new();
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);
    let mut manager = NopEventManager::new();

    let mut stages = tuple_list!(StdMutationalStage::new(NopMutator::new(
        MutationResult::Mutated
    )));

    let corpus_id = fuzzer
        .fuzz_one(&mut stages, &mut executor, &mut state, &mut manager)
        .unwrap();

    println!("last corpus: {}", corpus_id.0)
}
