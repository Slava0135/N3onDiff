use std::borrow::Cow;

use libafl::{prelude::*, state::State};
use libafl_bolts::{
    current_nanos, rands::StdRand, tuples::{tuple_list, Handle, Handled, MatchNameRef}, Named
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Output {
    status: String,
    errmsg: String,
    estack: Value,
}

#[derive(Clone)]
struct DiffStdOutObjective {
    neogo_stdout_observer: Handle<StdOutObserver>,
    neosharp_stdout_observer: Handle<StdOutObserver>,
}

impl<S> Feedback<S> for DiffStdOutObjective
where
    S: State,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &<S>::Input,
        observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        let neogo_output = observers
            .get(&self.neogo_stdout_observer)
            .unwrap()
            .stdout
            .clone();
        let neosharp_output = observers
            .get(&self.neosharp_stdout_observer)
            .unwrap()
            .stdout
            .clone();
        match (neogo_output, neosharp_output) {
            (Some(neogo_output), Some(neosharp_output)) => {
                let neogo_output: Output = serde_json::from_slice(&neogo_output)
                    .expect("failed to read json output from 'neo-go'");
                let neosharp_output: Output = serde_json::from_slice(&neosharp_output)
                    .expect("failed to read json output from 'neo-sharp'");
                if neogo_output.status != neosharp_output.status {
                    return Ok(true)
                }
                if neogo_output.estack != neosharp_output.estack {
                    return Ok(true)
                }
                Ok(false)
            }
            _ => panic!("no output found"),
        }
    }
}

impl Named for DiffStdOutObjective {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("DiffStdOutObjective")
    }
}

fn main() {
    let neogo_stdout_observer = StdOutObserver::new("neogo-stdout-observer");
    let neosharp_stdout_observer = StdOutObserver::new("neosharp-stdout-observer");

    let mut objective = DiffStdOutObjective{
        neogo_stdout_observer: neogo_stdout_observer.handle(),
        neosharp_stdout_observer: neosharp_stdout_observer.handle(),
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
